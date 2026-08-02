use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpListener;
use serde_json::{Value, json};

pub(crate) fn run_lsp_command(args: &[String]) -> Result<ExitCode, CliError> {
    let port = parse_port_flag(args);
    if let Some(port) = port {
        let listener = TcpListener::bind(format!("127.0.0.1:{port}"))
            .map_err(|e| failure(format!("failed to bind LSP TCP socket on port {port}: {e}")))?;
        eprintln!("ods lsp: listening for JSON-RPC connections on 127.0.0.1:{port}");
        for stream in listener.incoming().flatten() {
            let reader = stream.try_clone().map_err(|e| failure(e.to_string()))?;
            let writer = stream;
            let mut session = LspSession::new(BufReader::new(reader), writer);
            let _ = session.run_loop();
        }
    } else {
        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut session = LspSession::new(BufReader::new(stdin.lock()), stdout.lock());
        let _ = session.run_loop();
    }
    Ok(ExitCode::from(0))
}

fn parse_port_flag(args: &[String]) -> Option<u16> {
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--port" {
            if let Some(val) = args.get(i + 1) {
                return val.parse().ok();
            }
        }
        i += 1;
    }
    None
}

struct LspSession<R, W> {
    reader: R,
    writer: W,
    workspace_root: Option<PathBuf>,
    documents: HashMap<String, String>,
}

impl<R: BufRead, W: Write> LspSession<R, W> {
    fn new(reader: R, writer: W) -> Self {
        Self {
            reader,
            writer,
            workspace_root: None,
            documents: HashMap::new(),
        }
    }

    fn run_loop(&mut self) -> Result<(), io::Error> {
        while let Some(req) = self.read_message()? {
            let id = req.get("id").cloned();
            let method = req.get("method").and_then(Value::as_str).unwrap_or("");

            match method {
                "initialize" => {
                    if let Some(params) = req.get("params") {
                        if let Some(root_uri) = params.get("rootUri").and_then(Value::as_str) {
                            self.workspace_root = uri_to_path(root_uri);
                        } else if let Some(root_path) = params.get("rootPath").and_then(Value::as_str) {
                            self.workspace_root = Some(PathBuf::from(root_path));
                        }
                    }
                    if let Some(id) = id {
                        self.send_response(
                            &id,
                            json!({
                                "capabilities": {
                                    "textDocumentSync": 1,
                                    "hoverProvider": true,
                                    "definitionProvider": true,
                                    "completionProvider": {
                                        "triggerCharacters": [":", " ", "/"]
                                    }
                                }
                            }),
                        )?;
                    }
                }
                "initialized" => {}
                "textDocument/didOpen" => {
                    if let Some(params) = req.get("params") {
                        if let Some(doc) = params.get("textDocument") {
                            let uri = doc.get("uri").and_then(Value::as_str).unwrap_or("");
                            let text = doc.get("text").and_then(Value::as_str).unwrap_or("");
                            self.documents.insert(uri.to_string(), text.to_string());
                            self.publish_diagnostics_for_uri(uri)?;
                        }
                    }
                }
                "textDocument/didChange" => {
                    if let Some(params) = req.get("params") {
                        let uri = params.get("textDocument").and_then(|t| t.get("uri")).and_then(Value::as_str).unwrap_or("");
                        if let Some(changes) = params.get("contentChanges").and_then(Value::as_array) {
                            if let Some(last) = changes.last() {
                                if let Some(text) = last.get("text").and_then(Value::as_str) {
                                    self.documents.insert(uri.to_string(), text.to_string());
                                    self.publish_diagnostics_for_uri(uri)?;
                                }
                            }
                        }
                    }
                }
                "textDocument/didSave" => {
                    if let Some(params) = req.get("params") {
                        let uri = params.get("textDocument").and_then(|t| t.get("uri")).and_then(Value::as_str).unwrap_or("");
                        self.publish_diagnostics_for_uri(uri)?;
                    }
                }
                "textDocument/hover" => {
                    if let Some(id) = id {
                        let result = self.handle_hover(req.get("params"));
                        self.send_response(&id, result)?;
                    }
                }
                "textDocument/definition" => {
                    if let Some(id) = id {
                        let result = self.handle_definition(req.get("params"));
                        self.send_response(&id, result)?;
                    }
                }
                "textDocument/completion" => {
                    if let Some(id) = id {
                        let result = self.handle_completion(req.get("params"));
                        self.send_response(&id, result)?;
                    }
                }
                "shutdown" => {
                    if let Some(id) = id {
                        self.send_response(&id, Value::Null)?;
                    }
                }
                "exit" => {
                    break;
                }
                _ => {
                    if let Some(id) = id {
                        self.send_response(&id, Value::Null)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn read_message(&mut self) -> io::Result<Option<Value>> {
        let mut content_length: Option<usize> = None;
        loop {
            let mut line = String::new();
            let n = self.reader.read_line(&mut line)?;
            if n == 0 {
                return Ok(None);
            }
            let trimmed = line.trim();
            if trimmed.is_empty() {
                break;
            }
            if let Some(val) = trimmed.strip_prefix("Content-Length:") {
                content_length = val.trim().parse().ok();
            }
        }

        let Some(len) = content_length else {
            return Ok(None);
        };

        let mut buf = vec![0u8; len];
        self.reader.read_exact(&mut buf)?;
        let json_val: Value = serde_json::from_slice(&buf)?;
        Ok(Some(json_val))
    }

    fn send_response(&mut self, id: &Value, result: Value) -> io::Result<()> {
        let resp = json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": result
        });
        self.write_message(&resp)
    }

    fn send_notification(&mut self, method: &str, params: Value) -> io::Result<()> {
        let notif = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });
        self.write_message(&notif)
    }

    fn write_message(&mut self, val: &Value) -> io::Result<()> {
        let payload = serde_json::to_string(val)?;
        let header = format!("Content-Length: {}\r\n\r\n", payload.len());
        self.writer.write_all(header.as_bytes())?;
        self.writer.write_all(payload.as_bytes())?;
        self.writer.flush()
    }

    fn publish_diagnostics_for_uri(&mut self, uri: &str) -> io::Result<()> {
        let Some(path) = uri_to_path(uri) else {
            return Ok(());
        };

        let root = self.workspace_root.clone().unwrap_or_else(|| {
            path.parent().map(Path::to_path_buf).unwrap_or_else(|| PathBuf::from("."))
        });

        let mut lsp_diagnostics = Vec::new();

        if let Ok(ws) = load_workspace(&root) {
            let diags = lint_workspace_with_level(&ws, LintLevel::Strict);
            for diag in diags {
                if diag.path == path {
                    lsp_diagnostics.push(json!({
                        "range": {
                            "start": { "line": 0, "character": 0 },
                            "end": { "line": 0, "character": 100 }
                        },
                        "severity": match diag.severity {
                            Severity::Error => 1,
                            Severity::Warning => 2,
                        },
                        "source": "ods",
                        "message": diag.message
                    }));
                }
            }
        }

        self.send_notification(
            "textDocument/publishDiagnostics",
            json!({
                "uri": uri,
                "diagnostics": lsp_diagnostics
            }),
        )
    }

    fn handle_hover(&self, params: Option<&Value>) -> Value {
        let Some(params) = params else { return Value::Null };
        let uri = params.get("textDocument").and_then(|t| t.get("uri")).and_then(Value::as_str).unwrap_or("");
        let line_num = params.get("position").and_then(|p| p.get("line")).and_then(Value::as_u64).unwrap_or(0) as usize;

        let Some(text) = self.documents.get(uri) else { return Value::Null };
        let lines: Vec<&str> = text.lines().collect();
        let Some(line) = lines.get(line_num) else { return Value::Null };

        let hover_text = if line.contains("status:") {
            "**ods.status**: Document lifecycle state (`draft`, `stable`, `archived`)."
        } else if line.contains("profile:") {
            "**ods.profile**: Document profile template (`index`, `rfc`, `api`, `note`)."
        } else if line.contains("depends:") {
            "**ods.depends**: Hard graph dependency documents required by this document."
        } else if line.contains("related:") {
            "**ods.related**: Soft contextual relation documents associated with this document."
        } else if line.contains("share:") {
            "**ods.share**: Visibility filter (`public`, `org`, `private`)."
        } else if line.contains("custom-profiles:") {
            "**custom-profiles**: Workspace-wide array of custom profile schema definition paths."
        } else if line.contains("ods:") {
            "**ods**: Open Document Spec nested engine key block."
        } else {
            return Value::Null;
        };

        json!({
            "contents": {
                "kind": "markdown",
                "value": hover_text
            }
        })
    }

    fn handle_definition(&self, params: Option<&Value>) -> Value {
        let Some(params) = params else { return Value::Null };
        let uri = params.get("textDocument").and_then(|t| t.get("uri")).and_then(Value::as_str).unwrap_or("");
        let line_num = params.get("position").and_then(|p| p.get("line")).and_then(Value::as_u64).unwrap_or(0) as usize;

        let Some(text) = self.documents.get(uri) else { return Value::Null };
        let lines: Vec<&str> = text.lines().collect();
        let Some(line) = lines.get(line_num) else { return Value::Null };

        let doc_path = uri_to_path(uri);
        let doc_dir = doc_path.as_ref().and_then(|p| p.parent()).unwrap_or_else(|| Path::new("."));

        if let Some(target) = extract_path_from_line(line) {
            let target_path = doc_dir.join(target);
            if target_path.exists() {
                let target_uri = format!("file://{}", target_path.canonicalize().unwrap_or(target_path).display());
                return json!({
                    "uri": target_uri,
                    "range": {
                        "start": { "line": 0, "character": 0 },
                        "end": { "line": 0, "character": 0 }
                    }
                });
            }
        }

        Value::Null
    }

    fn handle_completion(&self, _params: Option<&Value>) -> Value {
        json!([
            { "label": "ods", "kind": 14, "detail": "ODS Engine Key Block" },
            { "label": "profile: rfc", "kind": 12, "detail": "RFC Specification Profile" },
            { "label": "profile: api", "kind": 12, "detail": "API Specification Profile" },
            { "label": "profile: note", "kind": 12, "detail": "General Note Profile" },
            { "label": "status: draft", "kind": 12, "detail": "Draft status" },
            { "label": "status: stable", "kind": 12, "detail": "Stable status" },
            { "label": "status: archived", "kind": 12, "detail": "Archived status" },
            { "label": "depends:", "kind": 14, "detail": "Graph dependencies array" },
            { "label": "related:", "kind": 14, "detail": "Related documents array" },
            { "label": "share: public", "kind": 12, "detail": "Public visibility" },
            { "label": "share: org", "kind": 12, "detail": "Organization visibility" },
            { "label": "share: private", "kind": 12, "detail": "Private visibility" }
        ])
    }
}

fn uri_to_path(uri: &str) -> Option<PathBuf> {
    if let Some(stripped) = uri.strip_prefix("file://") {
        Some(PathBuf::from(stripped))
    } else {
        Some(PathBuf::from(uri))
    }
}

fn extract_path_from_line(line: &str) -> Option<String> {
    if let Some(start) = line.find('(') {
        if let Some(end) = line[start..].find(')') {
            let target = &line[start + 1..start + end];
            if target.ends_with(".md") {
                return Some(target.to_string());
            }
        }
    }
    for word in line.split_whitespace() {
        let clean = word.trim_matches(|c| c == '-' || c == '"' || c == '\'' || c == '`');
        if clean.ends_with(".md") {
            return Some(clean.to_string());
        }
    }
    None
}

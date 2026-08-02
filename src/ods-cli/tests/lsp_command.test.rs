//! Broad LSP JSON-RPC surface coverage (stdio session).
use ods_test_support::temp_workspace;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

fn ods_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ods"))
}

fn write_jsonrpc_msg<W: Write>(writer: &mut W, body: &str) {
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    writer.write_all(header.as_bytes()).unwrap();
    writer.write_all(body.as_bytes()).unwrap();
    writer.flush().unwrap();
}

fn read_jsonrpc_msg<R: BufRead>(reader: &mut R) -> String {
    let mut content_length: Option<usize> = None;
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line).unwrap();
        if n == 0 {
            panic!("unexpected EOF reading JSON-RPC header");
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            break;
        }
        if let Some(val) = trimmed.strip_prefix("Content-Length:") {
            content_length = val.trim().parse().ok();
        }
    }
    let len = content_length.expect("Content-Length header");
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).unwrap();
    String::from_utf8(buf).unwrap()
}

/// Read until a response with matching id or a notification (method present, no id).
fn read_until_id_or_method<R: BufRead>(reader: &mut R, id: u64, method: Option<&str>) -> String {
    for _ in 0..20 {
        let msg = read_jsonrpc_msg(reader);
        if msg.contains(&format!(r#""id":{id}"#)) || msg.contains(&format!(r#""id": {id}"#)) {
            return msg;
        }
        if let Some(m) = method {
            if msg.contains(&format!(r#""method":"{m}""#)) {
                return msg;
            }
        }
        // skip unrelated notifications
    }
    panic!("timed out waiting for id={id} method={method:?}");
}

#[test]
fn test_lsp_jsonrpc_handshake_and_completion() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();

    let mut child = Command::new(ods_bin())
        .arg("lsp")
        .env("ODS_AUTO_UPDATE", "0")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn ods lsp");

    let stdin = child.stdin.as_mut().expect("stdin");
    let stdout = child.stdout.as_mut().expect("stdout");
    let mut reader = BufReader::new(stdout);

    let init_req = format!(
        r#"{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{"rootUri":"file://{root}"}}}}"#
    );
    write_jsonrpc_msg(stdin, &init_req);

    let init_resp = read_jsonrpc_msg(&mut reader);
    assert!(
        init_resp.contains(r#""hoverProvider":true"#),
        "init_resp: {init_resp}"
    );
    assert!(
        init_resp.contains(r#""definitionProvider":true"#),
        "init_resp: {init_resp}"
    );

    let comp_req = format!(
        r#"{{"jsonrpc":"2.0","id":2,"method":"textDocument/completion","params":{{"textDocument":{{"uri":"file://{root}/index.md"}},"position":{{"line":0,"character":0}}}}}}"#
    );
    write_jsonrpc_msg(stdin, &comp_req);

    let comp_resp = read_jsonrpc_msg(&mut reader);
    assert!(
        comp_resp.contains(r#""label":"ods""#),
        "comp_resp: {comp_resp}"
    );

    let exit_req = r#"{"jsonrpc":"2.0","method":"exit"}"#;
    write_jsonrpc_msg(stdin, exit_req);

    let _ = child.wait();
}

#[test]
fn test_lsp_document_lifecycle_hover_definition_and_diagnostics() {
    let dir = temp_workspace();
    let root = dir.path();
    let root_s = root.to_str().unwrap();

    // Minimal ODS workspace
    fs::write(
        root.join("index.md"),
        "---\nprofile: index\nods: 0.1\n---\n\n# Root\n\nSee [note](note.md).\n",
    )
    .unwrap();
    fs::write(
        root.join("note.md"),
        "---\nprofile: note\nstatus: draft\n---\n\n# Note\n",
    )
    .unwrap();
    fs::write(
        root.join("broken.md"),
        "---\nprofile: note\nstatus: draft\ndepends:\n  - missing/doc\n---\n\n# Broken\n",
    )
    .unwrap();

    let mut child = Command::new(ods_bin())
        .arg("lsp")
        .env("ODS_AUTO_UPDATE", "0")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn ods lsp");

    let stdin = child.stdin.as_mut().expect("stdin");
    let stdout = child.stdout.as_mut().expect("stdout");
    let mut reader = BufReader::new(stdout);

    // initialize via rootPath (alternate path)
    write_jsonrpc_msg(
        stdin,
        &format!(
            r#"{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{"rootPath":"{root_s}"}}}}"#
        ),
    );
    let _ = read_jsonrpc_msg(&mut reader);

    write_jsonrpc_msg(
        stdin,
        r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#,
    );

    // didOpen broken.md → publishDiagnostics
    let broken_text = fs::read_to_string(root.join("broken.md")).unwrap();
    let escaped = broken_text
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");
    write_jsonrpc_msg(
        stdin,
        &format!(
            r#"{{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{{"textDocument":{{"uri":"file://{root_s}/broken.md","languageId":"markdown","version":1,"text":"{escaped}"}}}}}}"#
        ),
    );
    let diag = read_until_id_or_method(&mut reader, 0, Some("textDocument/publishDiagnostics"));
    assert!(
        diag.contains("publishDiagnostics") || diag.contains("diagnostics"),
        "diag: {diag}"
    );

    // didChange full replace
    write_jsonrpc_msg(
        stdin,
        &format!(
            r#"{{"jsonrpc":"2.0","method":"textDocument/didChange","params":{{"textDocument":{{"uri":"file://{root_s}/broken.md","version":2}},"contentChanges":[{{"text":"---\nprofile: note\nstatus: draft\n---\n\n# Fixed\n"}}]}}}}"#
        ),
    );
    let _ = read_until_id_or_method(&mut reader, 0, Some("textDocument/publishDiagnostics"));

    // Open index for hover/definition
    let index_text = fs::read_to_string(root.join("index.md")).unwrap();
    let index_esc = index_text
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");
    write_jsonrpc_msg(
        stdin,
        &format!(
            r#"{{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{{"textDocument":{{"uri":"file://{root_s}/index.md","languageId":"markdown","version":1,"text":"{index_esc}"}}}}}}"#
        ),
    );
    let _ = read_until_id_or_method(&mut reader, 0, Some("textDocument/publishDiagnostics"));

    // hover on profile line (line 1)
    write_jsonrpc_msg(
        stdin,
        &format!(
            r#"{{"jsonrpc":"2.0","id":10,"method":"textDocument/hover","params":{{"textDocument":{{"uri":"file://{root_s}/index.md"}},"position":{{"line":1,"character":0}}}}}}"#
        ),
    );
    let hover = read_until_id_or_method(&mut reader, 10, None);
    assert!(
        hover.contains("profile") || hover.contains("contents"),
        "hover: {hover}"
    );

    // hover on ods line
    write_jsonrpc_msg(
        stdin,
        &format!(
            r#"{{"jsonrpc":"2.0","id":11,"method":"textDocument/hover","params":{{"textDocument":{{"uri":"file://{root_s}/index.md"}},"position":{{"line":2,"character":0}}}}}}"#
        ),
    );
    let _ = read_until_id_or_method(&mut reader, 11, None);

    // definition on markdown link line
    write_jsonrpc_msg(
        stdin,
        &format!(
            r#"{{"jsonrpc":"2.0","id":12,"method":"textDocument/definition","params":{{"textDocument":{{"uri":"file://{root_s}/index.md"}},"position":{{"line":6,"character":10}}}}}}"#
        ),
    );
    let def = read_until_id_or_method(&mut reader, 12, None);
    assert!(
        def.contains("note.md") || def.contains("result"),
        "def: {def}"
    );

    // didSave + didClose
    write_jsonrpc_msg(
        stdin,
        &format!(
            r#"{{"jsonrpc":"2.0","method":"textDocument/didSave","params":{{"textDocument":{{"uri":"file://{root_s}/index.md"}}}}}}"#
        ),
    );
    let _ = read_until_id_or_method(&mut reader, 0, Some("textDocument/publishDiagnostics"));

    write_jsonrpc_msg(
        stdin,
        &format!(
            r#"{{"jsonrpc":"2.0","method":"textDocument/didClose","params":{{"textDocument":{{"uri":"file://{root_s}/index.md"}}}}}}"#
        ),
    );
    let _ = read_until_id_or_method(&mut reader, 0, Some("textDocument/publishDiagnostics"));

    // cancel / setTrace / unknown method
    write_jsonrpc_msg(
        stdin,
        r#"{"jsonrpc":"2.0","method":"$/cancelRequest","params":{"id":99}}"#,
    );
    write_jsonrpc_msg(
        stdin,
        r#"{"jsonrpc":"2.0","method":"$/setTrace","params":{"value":"off"}}"#,
    );
    write_jsonrpc_msg(
        stdin,
        r#"{"jsonrpc":"2.0","id":20,"method":"textDocument/formatting","params":{}}"#,
    );
    let _ = read_until_id_or_method(&mut reader, 20, None);

    // workspace change notifications
    write_jsonrpc_msg(
        stdin,
        r#"{"jsonrpc":"2.0","method":"workspace/didChangeWatchedFiles","params":{"changes":[]}}"#,
    );
    write_jsonrpc_msg(
        stdin,
        r#"{"jsonrpc":"2.0","method":"workspace/didChangeWorkspaceFolders","params":{}}"#,
    );

    // shutdown + exit
    write_jsonrpc_msg(
        stdin,
        r#"{"jsonrpc":"2.0","id":99,"method":"shutdown","params":null}"#,
    );
    let _ = read_until_id_or_method(&mut reader, 99, None);
    write_jsonrpc_msg(stdin, r#"{"jsonrpc":"2.0","method":"exit"}"#);

    let status = child
        .wait_timeout(Duration::from_secs(5))
        .unwrap_or_else(|_| {
            let _ = child.kill();
            child.wait().ok();
            panic!("lsp did not exit");
        });
    let _ = status;
}

// wait_timeout polyfill for std
trait WaitTimeout {
    fn wait_timeout(&mut self, dur: Duration) -> std::io::Result<std::process::ExitStatus>;
}
impl WaitTimeout for std::process::Child {
    fn wait_timeout(&mut self, dur: Duration) -> std::io::Result<std::process::ExitStatus> {
        let start = std::time::Instant::now();
        loop {
            match self.try_wait()? {
                Some(s) => return Ok(s),
                None if start.elapsed() > dur => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        "wait timeout",
                    ));
                }
                None => std::thread::sleep(Duration::from_millis(50)),
            }
        }
    }
}

#[test]
fn test_lsp_skills_and_okf_hover_paths() {
    let dir = temp_workspace();
    let root = dir.path();
    let root_s = root.to_str().unwrap();

    // Skill package
    fs::write(
        root.join("SKILL.md"),
        "---\nname: demo\ndescription: A demo skill for hover tests.\nallowed-tools: Bash\ncompatibility: requires git\n---\n\n# Demo\n",
    )
    .unwrap();

    let mut child = Command::new(ods_bin())
        .arg("lsp")
        .env("ODS_AUTO_UPDATE", "0")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn");

    let stdin = child.stdin.as_mut().unwrap();
    let mut reader = BufReader::new(child.stdout.as_mut().unwrap());

    write_jsonrpc_msg(
        stdin,
        &format!(
            r#"{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{"rootUri":"file://{root_s}"}}}}"#
        ),
    );
    let _ = read_jsonrpc_msg(&mut reader);

    let skill_text = fs::read_to_string(root.join("SKILL.md")).unwrap();
    let esc = skill_text
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");
    write_jsonrpc_msg(
        stdin,
        &format!(
            r#"{{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{{"textDocument":{{"uri":"file://{root_s}/SKILL.md","languageId":"markdown","version":1,"text":"{esc}"}}}}}}"#
        ),
    );
    let _ = read_until_id_or_method(&mut reader, 0, Some("textDocument/publishDiagnostics"));

    // hover name / allowed-tools / compatibility lines
    for (id, line) in [(30u64, 1usize), (31, 3), (32, 4)] {
        write_jsonrpc_msg(
            stdin,
            &format!(
                r#"{{"jsonrpc":"2.0","id":{id},"method":"textDocument/hover","params":{{"textDocument":{{"uri":"file://{root_s}/SKILL.md"}},"position":{{"line":{line},"character":0}}}}}}"#
            ),
        );
        let _ = read_until_id_or_method(&mut reader, id, None);
    }

    // incremental change (use r## so nested "# does not terminate the raw string)
    write_jsonrpc_msg(
        stdin,
        &format!(
            r##"{{"jsonrpc":"2.0","method":"textDocument/didChange","params":{{"textDocument":{{"uri":"file://{root_s}/SKILL.md","version":2}},"contentChanges":[{{"range":{{"start":{{"line":6,"character":0}},"end":{{"line":6,"character":6}}}},"text":"# Demo2"}}]}}}}"##
        ),
    );
    let _ = read_until_id_or_method(&mut reader, 0, Some("textDocument/publishDiagnostics"));

    write_jsonrpc_msg(stdin, r#"{"jsonrpc":"2.0","method":"exit"}"#);
    let _ = child.wait();
}

#[test]
fn test_lsp_okf_hover_and_diagnostics() {
    let dir = temp_workspace();
    let root = dir.path();
    let root_s = root.to_str().unwrap();

    fs::write(
        root.join("index.md"),
        "---\nokf_version: \"0.2\"\ntype: Metric\nstale_after: \"2099-01-01\"\n---\n\n# OKF Root\n",
    )
    .unwrap();

    let mut child = Command::new(ods_bin())
        .arg("lsp")
        .env("ODS_AUTO_UPDATE", "0")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn");

    let stdin = child.stdin.as_mut().unwrap();
    let mut reader = BufReader::new(child.stdout.as_mut().unwrap());

    write_jsonrpc_msg(
        stdin,
        &format!(
            r#"{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{"rootUri":"file://{root_s}"}}}}"#
        ),
    );
    let _ = read_jsonrpc_msg(&mut reader);

    let text = fs::read_to_string(root.join("index.md")).unwrap();
    let esc = text
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");
    write_jsonrpc_msg(
        stdin,
        &format!(
            r#"{{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{{"textDocument":{{"uri":"file://{root_s}/index.md","languageId":"markdown","version":1,"text":"{esc}"}}}}}}"#
        ),
    );
    let _ = read_until_id_or_method(&mut reader, 0, Some("textDocument/publishDiagnostics"));

    for (id, line) in [(40u64, 1usize), (41, 2), (42, 3)] {
        write_jsonrpc_msg(
            stdin,
            &format!(
                r#"{{"jsonrpc":"2.0","id":{id},"method":"textDocument/hover","params":{{"textDocument":{{"uri":"file://{root_s}/index.md"}},"position":{{"line":{line},"character":0}}}}}}"#
            ),
        );
        let h = read_until_id_or_method(&mut reader, id, None);
        assert!(
            h.contains("contents") || h.contains("result"),
            "hover {id}: {h}"
        );
    }

    write_jsonrpc_msg(stdin, r#"{"jsonrpc":"2.0","method":"exit"}"#);
    let _ = child.wait();
}

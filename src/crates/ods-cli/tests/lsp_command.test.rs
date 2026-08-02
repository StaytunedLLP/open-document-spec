use ods_test_support::temp_workspace;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

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

#[test]
fn test_lsp_jsonrpc_handshake_and_completion() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();

    let mut child = Command::new(ods_bin())
        .arg("lsp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn ods lsp");

    let stdin = child.stdin.as_mut().expect("stdin");
    let stdout = child.stdout.as_mut().expect("stdout");
    let mut reader = BufReader::new(stdout);

    // 1. Send initialize
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

    // 2. Send completion
    let comp_req = format!(
        r#"{{"jsonrpc":"2.0","id":2,"method":"textDocument/completion","params":{{"textDocument":{{"uri":"file://{root}/index.md"}},"position":{{"line":0,"character":0}}}}}}"#
    );
    write_jsonrpc_msg(stdin, &comp_req);

    let comp_resp = read_jsonrpc_msg(&mut reader);
    assert!(
        comp_resp.contains(r#""label":"ods""#),
        "comp_resp: {comp_resp}"
    );

    // 3. Send exit
    let exit_req = r#"{"jsonrpc":"2.0","method":"exit"}"#;
    write_jsonrpc_msg(stdin, exit_req);

    let _ = child.wait();
}

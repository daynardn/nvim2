use std::{io::{BufRead, BufReader, Read, Write}, process::{self, Command, Stdio}, str::FromStr};
use lsp_types::*;
use serde_json::{to_string, value::to_raw_value};

pub fn run_lsp(directory: String) -> std::io::Result<()> {
    let mut clangd = Command::new("clangd")
        .arg("--log=verbose")
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdin = clangd.stdin.as_mut().expect("Failed to open stdin");
    let stdout = clangd.stdout.take().expect("Failed to open stdout");
    let stderr = clangd.stderr.take().expect("Failed to open stderr");

    #[allow(deprecated)]
    let initialize_params = InitializeParams {
        process_id: Some(process::id()),
        workspace_folders: None,
        // Some(vec![
            // WorkspaceFolder { 
            //     uri: Uri::from_str("file://./main.c").unwrap(),
            //     name: "".to_string(),
            // }]),
        initialization_options: None,
        capabilities: ClientCapabilities::default(),
        trace: Some(TraceValue::Verbose),
        client_info: None,
        locale: None,
        work_done_progress_params: WorkDoneProgressParams::default(),
        root_path: None,
        root_uri: Some(Uri::from_str(&("file://".to_owned() + &directory)).unwrap()),
    };

    let to_value = serde_json::to_value(initialize_params);
    
    let raw_value = to_raw_value(&to_value.unwrap()).unwrap();
    let initialize_request = jsonrpc::Request {
        jsonrpc: Some("2.0"),
        id: 1.into(),
        method: "initialize",
        params: Some(&raw_value),
        // params:
    };

    // Serialize the request and send it to clangd
    let request_json = to_string(&initialize_request)?;
    let request_message = format!("Content-Length: {}\r\n\r\n{}", request_json.len(), request_json);
    println!("request: {}", request_json);

    // write init message
    stdin.write_all(request_message.as_bytes())?;
    stdin.flush()?;

    let file_path = "./main.c";
    let file_content = std::fs::read_to_string(file_path)?;
    let did_open_request = format!(
        r#"{{
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {{
                "textDocument": {{
                    "uri": "file://{0}",
                    "languageId": "c",
                    "version": 1,
                    "text": "{1}"
                }}
            }}
        }}"#,
        file_path,
        file_content.replace('\n', "\\n").replace('"', "\\\"")
    );
    let did_open_message = format!(
        "Content-Length: {}\r\n\r\n{}",
        did_open_request.len(),
        did_open_request
    );
    println!("Sending didOpen request:\n{}", did_open_request);
    stdin.write_all(did_open_message.as_bytes())?;
    stdin.flush()?;

    // Read and process clangd responses
    let mut reader = BufReader::new(stdout);
    let mut response = String::new();

    // Monitor stderr for clangd logs
    let mut stderr_reader = BufReader::new(stderr);
    std::thread::spawn(move || {
        let mut log = String::new();
        while stderr_reader.read_line(&mut log).is_ok() {
            if !log.trim().is_empty() {
                println!("[CLANGD LOG] {:#}", log.trim());
            }
            log.clear();
        }
    });

    println!("Waiting for diagnostics...");
    while reader.read_line(&mut response)? > 0 {
        if response.contains("textDocument/publishDiagnostics") {
            println!("Diagnostics received:\n{}", response);
        }
        // println!("FIAG {}", response);
        response.clear();
    }

    clangd.wait()?;

    Ok(())
}
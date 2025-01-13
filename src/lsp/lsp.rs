use std::{
    io::{BufRead, BufReader, Read, Write},
    process::{self, Command, Stdio},
    str::FromStr,
};
use lsp_types::{
    *,
};
use serde_json::{from_str, json, to_string, value::to_raw_value};

fn read_message<R: Read>(reader: &mut BufReader<R>) -> std::io::Result<Option<String>> {
    let mut header = String::new();
    let mut content_length: usize = 0;

    // Read headers
    loop {
        header.clear();
        if reader.read_line(&mut header)? == 0 {
            return Ok(None);
        }
        
        let header = header.trim();
        if header.is_empty() {
            break;
        }
        
        if let Some(length_str) = header.strip_prefix("Content-Length: ") {
            content_length = length_str.parse().unwrap_or(0);
        }
    }

    // Read content
    let mut content = vec![0; content_length];
    reader.read_exact(&mut content)?;
    
    Ok(Some(String::from_utf8_lossy(&content).into_owned()))
}

fn get_init(directory: String) -> Option<String> {
    #[allow(deprecated)]
    let initialize_params = InitializeParams {
        process_id: Some(process::id()),
        workspace_folders: None,
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
    };

    let request_json = to_string(&initialize_request).ok()?;
    let request_message = format!("Content-Length: {}\r\n\r\n{}", request_json.len(), request_json);
    Some(request_message)
}

pub fn run_lsp(directory: String) -> std::io::Result<()> {
    let mut lsp = Command::new("rust-analyzer")
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdin = lsp.stdin.as_mut().expect("Failed to open stdin");
    let stdout = lsp.stdout.take().expect("Failed to open stdout");
    let stderr = lsp.stderr.take().expect("Failed to open stderr");

    let init_request = get_init(directory).unwrap();
    println!("{}", init_request);
    stdin.write_all(init_request.as_bytes())?;
    stdin.flush()?;

    // Handle stdout messages
    let mut stdout_reader = BufReader::new(stdout);
    if read_message(&mut stdout_reader).is_ok() {
        // Notify that we have initalized
        let initialized_notification = json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {}
        }).to_string();

        let message = format!("Content-Length: {}\r\n\r\n{}", initialized_notification.len(), initialized_notification);
        stdin.write_all(message.as_bytes())?;
        stdin.flush()?;

        let file_path = "./main.rs";
        let file_content = std::fs::read_to_string(file_path)?;
        let did_open_request = format!(
            r#"{{
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {{
                    "textDocument": {{
                        "uri": "file://{0}",
                        "languageId": "rs",
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

        stdin.write_all(did_open_message.as_bytes())?;
        stdin.flush()?;
    }

    // Monitor stderr for lsp logs
    let mut stderr_reader = BufReader::new(stderr);
    std::thread::spawn(move || {
        let mut log = String::new();
        while stderr_reader.read_line(&mut log).is_ok() {
            if !log.trim().is_empty() {
                println!("[LSP LOG] {:#}", log.trim());
            }
            log.clear();
        }
    });

    println!("Waiting for diagnostics...");
    while let Ok(Some(message)) = read_message(&mut stdout_reader) {
        // Try to parse as PublishDiagnostics notification
        if let Ok(notification) = from_str::<serde_json::Value>(&message) {
            if notification["method"] == "textDocument/publishDiagnostics" {
                println!("Received diagnostics:");
                println!("{}", serde_json::to_string_pretty(&notification).unwrap());
            } else {
                println!("Other message received:");
                println!("{}", serde_json::to_string_pretty(&notification).unwrap());
            }
        }
    }

    lsp.wait()?;
    Ok(())
}
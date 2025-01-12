use std::{alloc::System, io::stdout, process::{Command, Stdio}};
use lsp_types::*;


async fn run_lsp() -> std::io::Result<()> {
    let mut clangd = Command::new("clangd")
        .arg("--log=verbose")
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdin = clangd.stdin.as_mut().expect("Failed to open stdin");
    let stdout = clangd.stdout.take().expect("Failed to open stdout");
    let stderr = clangd.stderr.take().expect("Failed to open stderr");

    let initialize_params = InitializeParams {
        capabilities: 

    };
    Ok(())
}
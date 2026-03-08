use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::thread;

use anyhow::{Context, Result};

use crate::hypr;
use crate::model::{ControlRequest, ControlResponse};

pub fn spawn_control_socket_server() -> Result<()> {
    let socket_path = hypr::control_socket_path()?;

    remove_stale_socket(&socket_path)?;

    let listener = UnixListener::bind(&socket_path)
        .with_context(|| format!("failed to bind control socket at {}", socket_path.display()))?;

    thread::Builder::new()
        .name("hyprview2-ipc".to_string())
        .spawn(move || {
            if let Err(error) = serve(listener) {
                eprintln!("control socket server stopped: {error:#}");
            }
        })
        .context("failed to spawn control socket server thread")?;

    Ok(())
}

fn serve(listener: UnixListener) -> Result<()> {
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(error) = handle_connection(stream) {
                    eprintln!("control socket request failed: {error:#}");
                }
            }
            Err(error) => {
                eprintln!("failed to accept control socket connection: {error:#}");
            }
        }
    }

    Ok(())
}

fn handle_connection(mut stream: UnixStream) -> Result<()> {
    let mut line = String::new();
    {
        let mut reader = BufReader::new(&mut stream);
        let bytes_read = reader
            .read_line(&mut line)
            .context("failed to read control socket request")?;

        if bytes_read == 0 {
            write_response(
                &mut stream,
                &ControlResponse::error("request body is empty"),
            )?;
            return Ok(());
        }
    }

    let response = match process_request_line(&line) {
        Ok(request) => match handle_request(request) {
            Ok(()) => ControlResponse::ok(),
            Err(error) => ControlResponse::error(error),
        },
        Err(error) => ControlResponse::error(error),
    };

    write_response(&mut stream, &response)?;
    Ok(())
}

fn handle_request(request: ControlRequest) -> Result<(), String> {
    match request {
        ControlRequest::MoveWindow {
            window_address,
            target_workspace,
        } => {
            validate_move_window_request(&window_address, target_workspace)?;
            hypr::move_window_to_workspace(&window_address, target_workspace)
                .map_err(|error| format!("{error:#}"))
        }
    }
}

fn process_request_line(line: &str) -> Result<ControlRequest, String> {
    serde_json::from_str::<ControlRequest>(line.trim())
        .map_err(|error| format!("invalid request JSON: {error}"))
}

fn validate_move_window_request(window_address: &str, target_workspace: i32) -> Result<(), String> {
    if window_address.trim().is_empty() {
        return Err("window_address must not be empty".to_string());
    }

    if target_workspace <= 0 {
        return Err("target_workspace must be greater than 0".to_string());
    }

    Ok(())
}

fn write_response(stream: &mut UnixStream, response: &ControlResponse) -> Result<()> {
    let payload =
        serde_json::to_string(response).context("failed to serialize control response")?;
    stream
        .write_all(payload.as_bytes())
        .context("failed to write control socket response")?;
    stream
        .write_all(b"\n")
        .context("failed to terminate control socket response")?;
    Ok(())
}

fn remove_stale_socket(socket_path: &Path) -> Result<()> {
    match fs::remove_file(socket_path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error)
            .with_context(|| format!("failed to remove stale socket at {}", socket_path.display())),
    }
}

#[cfg(test)]
mod tests {
    use super::{process_request_line, validate_move_window_request};
    use crate::model::{ControlRequest, ControlResponse};

    #[test]
    fn parses_move_window_request() {
        let request = process_request_line(
            r#"{"type":"move_window","window_address":"0x123abc","target_workspace":4}"#,
        )
        .unwrap();

        assert_eq!(
            request,
            ControlRequest::MoveWindow {
                window_address: "0x123abc".to_string(),
                target_workspace: 4,
            }
        );
    }

    #[test]
    fn rejects_empty_window_address() {
        let error = validate_move_window_request("  ", 4).unwrap_err();
        assert_eq!(error, "window_address must not be empty");
    }

    #[test]
    fn rejects_non_positive_workspace() {
        let error = validate_move_window_request("0x123abc", 0).unwrap_err();
        assert_eq!(error, "target_workspace must be greater than 0");
    }

    #[test]
    fn serializes_error_response() {
        let response = ControlResponse::error("boom");
        let json = serde_json::to_string(&response).unwrap();

        assert_eq!(json, r#"{"ok":false,"error":"boom"}"#);
    }
}

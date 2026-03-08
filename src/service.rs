use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixStream;

use anyhow::{bail, Context, Result};

use crate::{hypr, ipc, notify};

pub fn run() -> Result<()> {
    ipc::spawn_control_socket_server()?;

    let socket_path = hypr::event_socket_path()?;
    let stream = UnixStream::connect(&socket_path).with_context(|| {
        format!(
            "failed to connect to Hyprland event socket at {}",
            socket_path.display()
        )
    })?;

    let initial_state = hypr::snapshot_workspaces()?;
    let mut last_state_message = notify::format_notification(&initial_state.workspaces);
    let startup_message = notify::format_startup_notification(&initial_state.workspaces);

    println!("{startup_message}");
    notify::send_notification(&startup_message)?;

    let reader = BufReader::new(stream);

    for line in reader.lines() {
        let line = line.context("failed to read from Hyprland event socket")?;

        if !is_relevant_event(&line) {
            continue;
        }

        match hypr::snapshot_workspaces() {
            Ok(state) => {
                let message = notify::format_notification(&state.workspaces);
                if message == last_state_message {
                    continue;
                }

                println!("{message}");

                if let Err(error) = notify::send_notification(&message) {
                    eprintln!("failed to send notification after event '{line}': {error:#}");
                    continue;
                }

                last_state_message = message;
            }
            Err(error) => {
                eprintln!("failed to refresh Hyprland state after event '{line}': {error:#}");
            }
        }
    }

    bail!(
        "Hyprland event socket at {} closed; restart the service to resume watching",
        socket_path.display()
    )
}

fn is_relevant_event(line: &str) -> bool {
    let Some((event, _)) = line.split_once(">>") else {
        return false;
    };

    matches!(
        event,
        "workspace"
            | "workspacev2"
            | "focusedmon"
            | "focusedmonv2"
            | "createworkspace"
            | "createworkspacev2"
            | "destroyworkspace"
            | "destroyworkspacev2"
            | "openwindow"
            | "closewindow"
            | "movewindow"
            | "movewindowv2"
    )
}

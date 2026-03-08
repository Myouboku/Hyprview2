use std::process::Command;

use anyhow::{bail, Context, Result};

use crate::model::WorkspaceState;

const MAX_MESSAGE_LEN: usize = 1800;

pub fn format_notification(workspaces: &[WorkspaceState]) -> String {
    let mut lines = Vec::with_capacity(workspaces.len());

    for workspace in workspaces {
        let line = if workspace.windows.is_empty() {
            format!("WS {}: empty", workspace.id)
        } else {
            let classes = workspace
                .windows
                .iter()
                .map(|window| window.class.as_str())
                .collect::<Vec<_>>()
                .join(", ");

            format!("WS {}: {}", workspace.id, classes)
        };

        lines.push(line);
    }

    let mut message = lines.join("\n");
    if message.len() > MAX_MESSAGE_LEN {
        message.truncate(utf8_boundary(&message, MAX_MESSAGE_LEN.saturating_sub(3)));
        message.push_str("...");
    }

    message
}

fn utf8_boundary(input: &str, max_bytes: usize) -> usize {
    if input.len() <= max_bytes {
        return input.len();
    }

    input
        .char_indices()
        .take_while(|(index, _)| *index <= max_bytes)
        .map(|(index, _)| index)
        .last()
        .unwrap_or(0)
}

pub fn send_notification(message: &str) -> Result<()> {
    let output = Command::new("hyprctl")
        .args(["notify", "-1", "5000", "rgb(7cc7ff)", message])
        .output()
        .context("failed to invoke hyprctl notify")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        bail!(
            "hyprctl notify failed with status {}: {}{}",
            output.status,
            stdout,
            stderr
        );
    }

    Ok(())
}

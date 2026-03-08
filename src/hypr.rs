use std::collections::BTreeMap;
use std::env;
use std::path::PathBuf;

use anyhow::{Context, Result};
use hyprland::data::{Clients, Workspace, Workspaces};
use hyprland::dispatch::{
    Dispatch, DispatchType, WindowIdentifier, WorkspaceIdentifierWithSpecial,
};
use hyprland::prelude::*;
use hyprland::shared::Address;

use crate::model::{WindowInfo, WorkspaceSnapshot, WorkspaceState};

pub fn snapshot_workspaces() -> Result<WorkspaceSnapshot> {
    let workspaces = Workspaces::get()
        .context("failed to query Hyprland workspaces")?
        .to_vec();
    let clients = Clients::get()
        .context("failed to query Hyprland clients")?
        .to_vec();
    let focused_workspace_id = Workspace::get_active()
        .context("failed to query active Hyprland workspace")?
        .id;

    let mut by_id = BTreeMap::<i32, WorkspaceState>::new();

    for workspace in workspaces {
        if workspace.id < 0 {
            continue;
        }

        by_id.insert(
            workspace.id,
            WorkspaceState {
                id: workspace.id,
                name: workspace.name,
                windows: Vec::new(),
            },
        );
    }

    for client in clients {
        let workspace_id = client.workspace.id;
        if workspace_id < 0 {
            continue;
        }

        let entry = by_id.entry(workspace_id).or_insert_with(|| WorkspaceState {
            id: workspace_id,
            name: client.workspace.name.clone(),
            windows: Vec::new(),
        });

        if client.class.trim().is_empty() {
            continue;
        }

        entry.windows.push(WindowInfo {
            address: client.address.to_string(),
            class: client.class,
        });
    }

    let mut state = by_id.into_values().collect::<Vec<_>>();
    for workspace in &mut state {
        workspace.windows.sort_by(|left, right| {
            left.class
                .cmp(&right.class)
                .then(left.address.cmp(&right.address))
        });
    }

    Ok(WorkspaceSnapshot {
        focused_workspace_id: (focused_workspace_id >= 0).then_some(focused_workspace_id),
        workspaces: state,
    })
}

pub fn move_window_to_workspace(window_address: &str, target_workspace: i32) -> Result<()> {
    Dispatch::call(DispatchType::MoveToWorkspaceSilent(
        WorkspaceIdentifierWithSpecial::Id(target_workspace),
        Some(WindowIdentifier::Address(Address::new(window_address))),
    ))
    .with_context(|| {
        format!("failed to move window {window_address} to workspace {target_workspace}")
    })?;

    Ok(())
}

pub fn switch_to_workspace(target_workspace: i32) -> Result<()> {
    Dispatch::call(DispatchType::Workspace(WorkspaceIdentifierWithSpecial::Id(
        target_workspace,
    )))
    .with_context(|| format!("failed to switch to workspace {target_workspace}"))?;

    Ok(())
}

pub fn event_socket_path() -> Result<PathBuf> {
    let runtime_dir = env::var("XDG_RUNTIME_DIR")
        .context("XDG_RUNTIME_DIR is not set; are you running inside a user session?")?;
    let instance_signature = env::var("HYPRLAND_INSTANCE_SIGNATURE")
        .context("HYPRLAND_INSTANCE_SIGNATURE is not set; are you running inside Hyprland?")?;

    Ok(PathBuf::from(runtime_dir)
        .join("hypr")
        .join(instance_signature)
        .join(".socket2.sock"))
}

pub fn control_socket_path() -> Result<PathBuf> {
    let runtime_dir = env::var("XDG_RUNTIME_DIR")
        .context("XDG_RUNTIME_DIR is not set; are you running inside a user session?")?;

    Ok(PathBuf::from(runtime_dir).join("hyprview2.sock"))
}

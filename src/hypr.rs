use std::collections::BTreeMap;

use anyhow::{Context, Result};
use hyprland::data::{Clients, Workspace, Workspaces};
use hyprland::dispatch::{Dispatch, DispatchType, WorkspaceIdentifierWithSpecial};
use hyprland::prelude::*;

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

pub fn switch_to_workspace(target_workspace: i32) -> Result<()> {
    Dispatch::call(DispatchType::Workspace(WorkspaceIdentifierWithSpecial::Id(
        target_workspace,
    )))
    .with_context(|| format!("failed to switch to workspace {target_workspace}"))?;

    Ok(())
}

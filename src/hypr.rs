use std::collections::BTreeMap;

use anyhow::{Context, Result};
use hyprland::data::{Clients, Workspaces};
use hyprland::prelude::*;

use crate::model::{WindowInfo, WorkspaceState};

pub fn snapshot_workspaces() -> Result<Vec<WorkspaceState>> {
    let workspaces = Workspaces::get()
        .context("failed to query Hyprland workspaces")?
        .to_vec();
    let clients = Clients::get()
        .context("failed to query Hyprland clients")?
        .to_vec();

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
            class: client.class,
        });
    }

    let mut state = by_id.into_values().collect::<Vec<_>>();
    for workspace in &mut state {
        workspace
            .windows
            .sort_by(|left, right| left.class.cmp(&right.class));
    }

    Ok(state)
}

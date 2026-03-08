#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowInfo {
    pub address: String,
    pub class: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceState {
    pub id: i32,
    pub name: String,
    pub windows: Vec<WindowInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSnapshot {
    pub focused_workspace_id: Option<i32>,
    pub workspaces: Vec<WorkspaceState>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowInfo {
    pub class: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceState {
    pub id: i32,
    pub name: String,
    pub windows: Vec<WindowInfo>,
}

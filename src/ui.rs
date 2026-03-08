use std::time::Duration;

use iced::alignment::{Horizontal, Vertical};
use iced::task::Task;
use iced::time;
use iced::widget::{column, container, row, scrollable, text};
use iced::{Alignment, Element, Fill, Length, Subscription, Theme, window};

use crate::hypr;
use crate::model::WorkspaceState;

const WINDOW_WIDTH: f32 = 720.0;
const WINDOW_HEIGHT: f32 = 520.0;
const REFRESH_INTERVAL: Duration = Duration::from_millis(500);

pub fn run() -> iced::Result {
    iced::application(title, update, view)
        .theme(theme)
        .subscription(subscription)
        .window(window_settings())
        .run_with(|| (HyprviewApp::default(), initial_task()))
}

#[derive(Debug, Clone)]
enum ViewState {
    Loading,
    Ready(Vec<WorkspaceState>),
    Error(String),
}

impl Default for ViewState {
    fn default() -> Self {
        Self::Loading
    }
}

#[derive(Default)]
struct HyprviewApp {
    view_state: ViewState,
}

#[derive(Debug, Clone)]
enum Message {
    Refresh,
    SnapshotLoaded(Result<Vec<WorkspaceState>, String>),
}

fn title(_app: &HyprviewApp) -> String {
    "Hyprview2".to_string()
}

fn theme(_app: &HyprviewApp) -> Theme {
    Theme::TokyoNight
}

fn update(app: &mut HyprviewApp, message: Message) -> Task<Message> {
    match message {
        Message::Refresh => refresh_task(),
        Message::SnapshotLoaded(result) => {
            app.view_state = match result {
                Ok(workspaces) => ViewState::Ready(workspaces),
                Err(error) => ViewState::Error(error),
            };

            Task::none()
        }
    }
}

fn subscription(_app: &HyprviewApp) -> Subscription<Message> {
    time::every(REFRESH_INTERVAL).map(|_| Message::Refresh)
}

fn view(app: &HyprviewApp) -> Element<'_, Message> {
    let content = match &app.view_state {
        ViewState::Loading => loading_view(),
        ViewState::Ready(workspaces) => workspace_list_view(workspaces),
        ViewState::Error(error) => error_view(error),
    };

    container(content)
        .width(Fill)
        .height(Fill)
        .center_x(Fill)
        .center_y(Fill)
        .into()
}

fn loading_view<'a>() -> Element<'a, Message> {
    let content = column![
        text("Hyprview2").size(28),
        text("Chargement des workspaces...").size(16),
    ]
    .spacing(12)
    .align_x(Alignment::Center)
    .max_width(420);

    panel(content).into()
}

fn error_view<'a>(error: &'a str) -> Element<'a, Message> {
    let content = column![
        text("Hyprview2").size(28),
        text("Impossible de lire l'etat Hyprland.").size(18),
        text(error).size(14),
    ]
    .spacing(12)
    .align_x(Alignment::Center)
    .max_width(520);

    panel(content).into()
}

fn workspace_list_view(workspaces: &[WorkspaceState]) -> Element<'_, Message> {
    let mut items = column![
        row![
            text("Hyprview2").size(28),
            text(format!("{} workspace(s)", workspaces.len())).size(16),
        ]
        .spacing(16)
        .align_y(Alignment::Center)
    ]
    .spacing(18);

    for workspace in workspaces {
        items = items.push(workspace_card(workspace));
    }

    let scroll = scrollable(items).height(Length::Shrink);

    panel(scroll).into()
}

fn workspace_card(workspace: &WorkspaceState) -> Element<'_, Message> {
    let header = row![
        text(format!("Workspace {}", workspace.id)).size(20),
        text(workspace.name.as_str()).size(15),
    ]
    .spacing(12)
    .align_y(Alignment::Center);

    let windows = if workspace.windows.is_empty() {
        column![text("empty").size(15)]
    } else {
        workspace.windows.iter().fold(column![].spacing(8), |column, window| {
            column.push(
                row![
                    text("-"),
                    text(window.class.as_str()).size(16),
                    text(window.address.as_str()).size(13),
                ]
                .spacing(10),
            )
        })
    };

    container(column![header, windows].spacing(12))
        .width(Length::Fill)
        .padding(16)
        .into()
}

fn panel<'a>(content: impl Into<Element<'a, Message>>) -> iced::widget::Container<'a, Message> {
    container(content)
        .width(Length::Fill)
        .max_width(640)
        .padding(24)
        .align_x(Horizontal::Left)
        .align_y(Vertical::Top)
}

fn initial_task() -> Task<Message> {
    refresh_task()
}

fn refresh_task() -> Task<Message> {
    Task::perform(async { hypr::snapshot_workspaces().map_err(|error| format!("{error:#}")) }, Message::SnapshotLoaded)
}

fn window_settings() -> window::Settings {
    window::Settings {
        size: iced::Size::new(WINDOW_WIDTH, WINDOW_HEIGHT),
        position: window::Position::Centered,
        resizable: true,
        decorations: true,
        ..window::Settings::default()
    }
}

use std::time::Duration;

use iced::alignment::{Horizontal, Vertical};
use iced::task::Task;
use iced::time;
use iced::widget::{Space, button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Fill, Length, Subscription, Theme, window};

use crate::hypr;
use crate::model::{WorkspaceSnapshot, WorkspaceState};

const WINDOW_WIDTH: f32 = 720.0;
const WINDOW_HEIGHT: f32 = 520.0;
const REFRESH_INTERVAL: Duration = Duration::from_millis(500);
const MAX_COLUMNS: usize = 5;
const CARD_MIN_HEIGHT: f32 = 170.0;
const CARD_WINDOW_LIST_HEIGHT: f32 = 120.0;

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
    Ready(WorkspaceSnapshot),
    Error(String),
}

impl Default for ViewState {
    fn default() -> Self {
        Self::Loading
    }
}

struct HyprviewApp {
    view_state: ViewState,
    window_width: f32,
}

impl Default for HyprviewApp {
    fn default() -> Self {
        Self {
            view_state: ViewState::Loading,
            window_width: WINDOW_WIDTH,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    Refresh,
    SnapshotLoaded(Result<WorkspaceSnapshot, String>),
    SwitchWorkspace(i32),
    WorkspaceSwitched(Result<(), String>),
    WindowResized(f32),
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
        Message::SwitchWorkspace(workspace_id) => Task::perform(
            async move {
                hypr::switch_to_workspace(workspace_id)
                    .map_err(|error| format!("{error:#}"))
            },
            Message::WorkspaceSwitched,
        ),
        Message::WorkspaceSwitched(result) => match result {
            Ok(()) => refresh_task(),
            Err(error) => {
                app.view_state = ViewState::Error(error);
                Task::none()
            }
        },
        Message::WindowResized(width) => {
            app.window_width = width;
            Task::none()
        }
    }
}

fn subscription(_app: &HyprviewApp) -> Subscription<Message> {
    Subscription::batch([
        time::every(REFRESH_INTERVAL).map(|_| Message::Refresh),
        window::resize_events().map(|(_id, size)| Message::WindowResized(size.width)),
    ])
}

fn view(app: &HyprviewApp) -> Element<'_, Message> {
    let content = match &app.view_state {
        ViewState::Loading => loading_view(),
        ViewState::Ready(snapshot) => workspace_list_view(snapshot, app.window_width),
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

fn workspace_list_view(snapshot: &WorkspaceSnapshot, window_width: f32) -> Element<'_, Message> {
    let workspaces = snapshot.workspaces.as_slice();
    let columns = columns_for_width(window_width);
    let mut items = column![
        row![
            text("Hyprview2").size(28),
            text(format!("{} workspace(s)", workspaces.len())).size(16),
        ]
        .spacing(16)
        .align_y(Alignment::Center)
    ]
    .spacing(18);

    for row_workspaces in workspaces.chunks(columns) {
        let row = workspace_row(row_workspaces, columns, snapshot.focused_workspace_id);
        items = items.push(row);
    }

    let scroll = scrollable(items).height(Length::Fill);

    panel(scroll).into()
}

fn workspace_row<'a>(
    workspaces: &'a [WorkspaceState],
    columns: usize,
    focused_workspace_id: Option<i32>,
) -> Element<'a, Message> {
    let mut cards = row![].spacing(16).width(Length::Fill);

    for workspace in workspaces {
        cards = cards.push(workspace_card(workspace, focused_workspace_id));
    }

    for _ in workspaces.len()..columns {
        cards = cards.push(Space::with_width(Length::FillPortion(1)));
    }

    cards.into()
}

fn workspace_card(workspace: &WorkspaceState, focused_workspace_id: Option<i32>) -> Element<'_, Message> {
    let title = if focused_workspace_id == Some(workspace.id) {
        format!("Workspace {} *", workspace.id)
    } else {
        format!("Workspace {}", workspace.id)
    };

    let header = row![
        button(text(title).size(20))
            .padding(0)
            .on_press(Message::SwitchWorkspace(workspace.id)),
        text(workspace.name.as_str()).size(15),
    ]
    .spacing(12)
    .align_y(Alignment::Center);

    let windows = if workspace.windows.is_empty() {
        container(text("empty").size(15))
            .height(CARD_WINDOW_LIST_HEIGHT)
            .align_y(Vertical::Center)
    } else {
        let windows = workspace
            .windows
            .iter()
            .fold(column![].spacing(8), |column, window| {
                column.push(
                    row![
                        text("-"),
                        text(window.class.as_str()).size(16),
                        text(window.address.as_str()).size(13),
                    ]
                    .spacing(10),
                )
            });

        container(scrollable(windows).height(CARD_WINDOW_LIST_HEIGHT))
    };

    container(column![header, windows].spacing(12))
        .width(Length::FillPortion(1))
        .height(CARD_MIN_HEIGHT)
        .padding(16)
        .into()
}

fn panel<'a>(content: impl Into<Element<'a, Message>>) -> iced::widget::Container<'a, Message> {
    container(content)
        .width(Length::Fill)
        .max_width(1440)
        .padding(24)
        .align_x(Horizontal::Left)
        .align_y(Vertical::Top)
}

fn columns_for_width(window_width: f32) -> usize {
    if window_width < 500.0 {
        1
    } else if window_width < 760.0 {
        2
    } else if window_width < 1020.0 {
        3
    } else if window_width < 1280.0 {
        4
    } else {
        MAX_COLUMNS
    }
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

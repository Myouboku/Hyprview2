mod hypr;
mod model;
mod notify;

use anyhow::Result;

fn main() -> Result<()> {
    let state = hypr::snapshot_workspaces()?;
    let message = notify::format_notification(&state);

    println!("{message}");
    notify::send_notification(&message)?;

    Ok(())
}

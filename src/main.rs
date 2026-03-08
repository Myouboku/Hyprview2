mod hypr;
mod model;
mod notify;
mod service;

use anyhow::Result;

fn main() -> Result<()> {
    service::run()
}

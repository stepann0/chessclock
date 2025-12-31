use anyhow::Result;

use crate::app::App;

mod app;
mod clock;
mod event;
mod tabs;

#[tokio::main]
async fn main() -> Result<()> {
    let terminal = ratatui::init();
    let result = App::new().run(terminal).await;
    ratatui::restore();
    result
}

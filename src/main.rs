use std::io;

use crate::app::App;

pub mod app;
pub mod event;
pub mod tabs;

#[tokio::main]
async fn main() -> io::Result<()> {
    let terminal = ratatui::init();
    let result = App::new().run(terminal).await;
    ratatui::restore();
    result
}

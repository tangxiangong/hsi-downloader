mod app;
mod event;
pub mod theme;
pub mod widgets;
mod ui;

pub use app::App;
pub use event::{Event, EventHandler};

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

pub async fn run() -> Result<()> {
    // 设置终端
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 创建应用
    let mut app = App::new().await?;
    let mut event_handler = EventHandler::new(250);

    // 运行主循环
    let result = run_app(&mut terminal, &mut app, &mut event_handler).await;

    // 恢复终端
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    event_handler: &mut EventHandler,
) -> Result<()>
where
    <B as ratatui::backend::Backend>::Error: Send + Sync + 'static,
{
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if let Some(event) = event_handler.next().await {
            match event {
                Event::Key(key) => {
                    if !app.handle_key(key).await? {
                        app.persist_on_exit().await?;
                        return Ok(());
                    }
                }
                Event::Tick => {
                    app.on_tick().await?;
                }
            }
        }
    }
}

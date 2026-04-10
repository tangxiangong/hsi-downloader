use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum Event {
    Key(KeyEvent),
    Tick,
}

pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<Event>,
    _tx: mpsc::UnboundedSender<Event>,
}

impl EventHandler {
    pub fn new(tick_rate: u64) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let event_tx = tx.clone();

        tokio::spawn(async move {
            let mut tick_interval = tokio::time::interval(Duration::from_millis(tick_rate));
            loop {
                let tick_delay = tick_interval.tick();
                let crossterm_event = event::poll(Duration::from_millis(tick_rate));

                tokio::select! {
                    _ = tick_delay => {
                        if event_tx.send(Event::Tick).is_err() {
                            break;
                        }
                    }
                    _ = tokio::task::spawn_blocking(move || crossterm_event) => {
                        if event::poll(Duration::from_secs(0)).unwrap_or(false)
                            && let Ok(CrosstermEvent::Key(key)) = event::read()
                                && event_tx.send(Event::Key(key)).is_err() {
                                    break;
                                }
                    }
                }
            }
        });

        Self { rx, _tx: tx }
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }
}

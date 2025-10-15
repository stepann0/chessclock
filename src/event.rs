use futures::{FutureExt, StreamExt};
use ratatui::crossterm::event::Event as CrosstermEvent;
use std::{io, time::Duration};
use tokio::sync::mpsc;

/// The frequency at which tick events are emitted.
const TICK_FPS: f64 = 60.0;
/// Timer tick event
pub(crate) const TIMER_TICK: u64 = 10;

/// Representation of all possible events.
#[derive(Clone, Debug)]
pub enum Event {
    Tick,
    /// An event that represents timer tick. Each tick is TIMER_TICK milliseconds long
    ///
    /// Use this event to decrement player's timer
    TimerTick,
    /// Crossterm events.
    /// These events are emitted by the terminal.
    Crossterm(CrosstermEvent),
    /// Application events.
    ///
    /// Use this event to emit custom events that are specific to your application.
    App(AppEvent),
}

#[derive(Clone, Debug)]
pub enum AppEvent {
    Timeout,
    HitClock,
    /// Quit the application.
    Quit,
}

/// Terminal event handler.
#[derive(Debug)]
pub struct EventHandler {
    /// Event sender channel.
    sender: mpsc::UnboundedSender<Event>,
    /// Event receiver channel.
    receiver: mpsc::UnboundedReceiver<Event>,
}

impl EventHandler {
    /// Constructs a new instance of [`EventHandler`] and spawns a new thread to handle events.
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let actor = EventTask::new(sender.clone());
        tokio::spawn(async { actor.run().await });
        Self { sender, receiver }
    }

    pub async fn next(&mut self) -> io::Result<Event> {
        self.receiver.recv().await.ok_or(io::Error::new(
            io::ErrorKind::Other,
            "could not recieve event",
        ))
    }

    /// Queue an app event to be sent to the event receiver.
    ///
    /// This is useful for sending events to the event handler which will be processed by the next
    /// iteration of the application's event loop.
    pub fn send(&mut self, app_event: AppEvent) {
        // Ignore the result as the reciever cannot be dropped while this struct still has a
        // reference to it
        let _ = self.sender.send(Event::App(app_event));
    }
}

/// A thread that handles reading crossterm events and emitting tick events on a regular schedule.
struct EventTask {
    sender: mpsc::UnboundedSender<Event>,
}

impl EventTask {
    fn new(sender: mpsc::UnboundedSender<Event>) -> Self {
        Self { sender }
    }

    /// Runs the event thread.
    ///
    /// This function emits tick events at a fixed rate and polls for crossterm events in between.
    async fn run(self) -> io::Result<()> {
        let mut reader = crossterm::event::EventStream::new();

        let tick_rate = Duration::from_secs_f64(1.0 / TICK_FPS);
        let mut tick = tokio::time::interval(tick_rate);

        let millisec = Duration::from_millis(TIMER_TICK);
        let mut clock_tick = tokio::time::interval(millisec);
        loop {
            let clock_tick_delay = clock_tick.tick();
            let tick_delay = tick.tick();
            let crossterm_event = reader.next().fuse();
            tokio::select! {
              _ = self.sender.closed() => {
                break;
              }
              _ = tick_delay => {
                self.send(Event::Tick);
              }
              _ = clock_tick_delay => {
                self.send(Event::TimerTick);
              }
              Some(Ok(evt)) = crossterm_event => {
                  self.send(Event::Crossterm(evt)); // evt: crossterm::event::Event
              }
            };
        }
        Ok(())
    }

    /// Sends an event to the receiver.
    fn send(&self, event: Event) {
        // Ignores the result because shutting down the app drops the receiver, which causes the send
        // operation to fail. This is expected behavior and should not panic.
        let _ = self.sender.send(event);
    }
}

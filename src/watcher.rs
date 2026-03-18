use std::fmt;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};

use notify::{Event as NotifyEvent, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::events::Event;

// ---------------------------------------------------------------------------
// WatcherError
// ---------------------------------------------------------------------------

/// Errors that can occur while setting up or running the file watcher.
#[derive(Debug)]
pub enum WatcherError {
    /// An I/O error (e.g. failing to create the events directory).
    Io(std::io::Error),
    /// An error from the `notify` crate.
    Notify(notify::Error),
}

impl fmt::Display for WatcherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WatcherError::Io(e) => write!(f, "I/O error in file watcher: {e}"),
            WatcherError::Notify(e) => write!(f, "notify watcher error: {e}"),
        }
    }
}

impl std::error::Error for WatcherError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            WatcherError::Io(e) => Some(e),
            WatcherError::Notify(e) => Some(e),
        }
    }
}

impl From<std::io::Error> for WatcherError {
    fn from(e: std::io::Error) -> Self {
        WatcherError::Io(e)
    }
}

impl From<notify::Error> for WatcherError {
    fn from(e: notify::Error) -> Self {
        WatcherError::Notify(e)
    }
}

// ---------------------------------------------------------------------------
// EventWatcher
// ---------------------------------------------------------------------------

/// Watches `~/.config/gnoem/events/` for new event files and delivers parsed
/// [`Event`] values over an in-process channel.
///
/// # Lifecycle
///
/// 1. [`EventWatcher::new`] — creates the directory if absent, sets up the
///    channel, but does **not** start the OS-level watch yet.
/// 2. [`EventWatcher::start`] — attaches the OS watcher to the directory.
/// 3. [`EventWatcher::try_recv`] — call in your main loop to drain events.
/// 4. [`EventWatcher::stop`] — detaches the OS watcher.
pub struct EventWatcher {
    /// Directory being monitored.
    events_dir: PathBuf,
    /// Sends parsed events from the watcher callback to the main thread.
    tx: Sender<Event>,
    /// Receives parsed events on the main thread side.
    rx: Receiver<Event>,
    /// The underlying `notify` watcher — `Some` while watching, `None` otherwise.
    inner: Option<RecommendedWatcher>,
}

impl EventWatcher {
    /// Create a new `EventWatcher` for `events_dir`.
    ///
    /// The events directory is created if it does not already exist.
    /// The OS watcher is **not** started; call [`start`](Self::start) when ready.
    pub fn new(events_dir: PathBuf) -> Result<Self, WatcherError> {
        std::fs::create_dir_all(&events_dir).map_err(WatcherError::Io)?;

        let (tx, rx) = mpsc::channel::<Event>();

        Ok(Self {
            events_dir,
            tx,
            rx,
            inner: None,
        })
    }

    /// Start watching the events directory for new files.
    ///
    /// Each time a file is created the watcher will:
    /// 1. Parse it with [`Event::from_file`].
    /// 2. On success — send the event through the channel, then delete the file.
    /// 3. On failure — log a warning with `eprintln!`, then delete the file.
    pub fn start(&mut self) -> Result<(), WatcherError> {
        let tx = self.tx.clone();
        let events_dir = self.events_dir.clone();

        let mut watcher = RecommendedWatcher::new(
            move |result: notify::Result<NotifyEvent>| {
                let notify_event = match result {
                    Ok(e) => e,
                    Err(err) => {
                        eprintln!("[gnoem watcher] notify error: {err}");
                        return;
                    }
                };

                // We only care about file-creation events.
                if !matches!(notify_event.kind, EventKind::Create(_)) {
                    return;
                }

                for path in &notify_event.paths {
                    // Skip if this isn't a file directly inside events_dir
                    // (notify can occasionally fire for the directory itself).
                    if path.parent() != Some(events_dir.as_path()) {
                        continue;
                    }

                    match Event::from_file(path) {
                        Ok(event) => {
                            // Deliver the event before deleting so the receiver
                            // always sees it even if deletion fails.
                            if let Err(send_err) = tx.send(event) {
                                eprintln!("[gnoem watcher] channel closed, dropping event: {send_err}");
                            }
                            if let Err(del_err) = std::fs::remove_file(path) {
                                eprintln!(
                                    "[gnoem watcher] failed to delete event file {}: {del_err}",
                                    path.display()
                                );
                            }
                        }
                        Err(parse_err) => {
                            eprintln!(
                                "[gnoem watcher] failed to parse event file {}: {parse_err}",
                                path.display()
                            );
                            if let Err(del_err) = std::fs::remove_file(path) {
                                eprintln!(
                                    "[gnoem watcher] failed to delete unparseable file {}: {del_err}",
                                    path.display()
                                );
                            }
                        }
                    }
                }
            },
            notify::Config::default(),
        )
        .map_err(WatcherError::Notify)?;

        watcher
            .watch(&self.events_dir, RecursiveMode::NonRecursive)
            .map_err(WatcherError::Notify)?;

        self.inner = Some(watcher);
        Ok(())
    }

    /// Non-blocking poll for the next parsed event.
    ///
    /// Returns `None` when there are no events waiting in the channel.
    pub fn try_recv(&self) -> Option<Event> {
        self.rx.try_recv().ok()
    }

    /// Stop watching the directory.
    ///
    /// Drops the underlying OS watcher; any events already in the channel can
    /// still be drained with [`try_recv`](Self::try_recv).
    pub fn stop(&mut self) {
        self.inner = None;
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::time::Duration;
    use tempfile::TempDir;

    // ------------------------------------------------------------------
    // Helpers
    // ------------------------------------------------------------------

    /// Creates a temporary directory that acts as the events directory.
    fn temp_events_dir() -> TempDir {
        tempfile::tempdir().expect("create tempdir")
    }

    /// Returns a minimal valid JSON string for a `SessionStart` event.
    fn session_start_json(session_id: &str) -> String {
        format!(
            r#"{{"event":"session_start","session_id":"{session_id}","cwd":"/tmp","timestamp":1000}}"#
        )
    }

    /// Writes `content` to a new `.json` file inside `dir` and returns its path.
    fn write_event_file(dir: &std::path::Path, filename: &str, content: &str) -> PathBuf {
        let path = dir.join(filename);
        let mut f = std::fs::File::create(&path).expect("create event file");
        f.write_all(content.as_bytes()).expect("write event file");
        // Flush and sync so the OS watcher sees a complete file.
        f.flush().expect("flush");
        f.sync_all().expect("sync");
        path
    }

    // ------------------------------------------------------------------
    // new() creates the events directory if missing
    // ------------------------------------------------------------------

    /// Given a path that does not yet exist, `EventWatcher::new` creates it.
    #[test]
    fn new_creates_events_directory_when_missing() {
        let base = tempfile::tempdir().expect("create base tempdir");
        let events_path = base.path().join("events");

        assert!(
            !events_path.exists(),
            "precondition: events dir should not exist yet"
        );

        EventWatcher::new(events_path.clone()).expect("EventWatcher::new failed");

        assert!(
            events_path.exists(),
            "EventWatcher::new should have created the events directory"
        );
    }

    /// When the directory already exists, `new` succeeds without error.
    #[test]
    fn new_succeeds_when_events_directory_already_exists() {
        let tmp = temp_events_dir();
        EventWatcher::new(tmp.path().to_path_buf()).expect("EventWatcher::new failed for existing dir");
    }

    // ------------------------------------------------------------------
    // try_recv() returns None when no events are pending
    // ------------------------------------------------------------------

    /// A freshly created watcher with no files written has nothing to receive.
    #[test]
    fn try_recv_returns_none_when_no_events_pending() {
        let tmp = temp_events_dir();
        let watcher = EventWatcher::new(tmp.path().to_path_buf())
            .expect("EventWatcher::new failed");

        assert!(
            watcher.try_recv().is_none(),
            "expected None from try_recv when no events have been written"
        );
    }

    /// `try_recv` also returns None on a started-but-idle watcher.
    #[test]
    fn try_recv_returns_none_on_idle_started_watcher() {
        let tmp = temp_events_dir();
        let mut watcher = EventWatcher::new(tmp.path().to_path_buf())
            .expect("EventWatcher::new failed");
        watcher.start().expect("start failed");

        assert!(
            watcher.try_recv().is_none(),
            "expected None from idle started watcher"
        );

        watcher.stop();
    }

    // ------------------------------------------------------------------
    // Full flow: write file → watcher fires → try_recv returns event
    // ------------------------------------------------------------------

    /// When a valid event file appears in the watched directory, `try_recv`
    /// eventually returns the parsed event.
    #[test]
    fn full_flow_write_event_file_delivers_parsed_event() {
        let tmp = temp_events_dir();
        let mut watcher = EventWatcher::new(tmp.path().to_path_buf())
            .expect("EventWatcher::new failed");
        watcher.start().expect("start failed");

        write_event_file(tmp.path(), "evt1.json", &session_start_json("abc-123"));

        // Poll with backoff — the OS watcher runs on a background thread.
        let event = poll_for_event(&watcher, Duration::from_secs(5));

        watcher.stop();

        let event = event.expect("expected an event but got None after timeout");
        assert_eq!(
            event,
            Event::SessionStart {
                session_id: "abc-123".into(),
                cwd: "/tmp".into(),
                timestamp: 1000,
            }
        );
    }

    // ------------------------------------------------------------------
    // Event files are deleted after successful processing
    // ------------------------------------------------------------------

    /// After `try_recv` returns an event the source file must be gone.
    #[test]
    fn event_file_is_deleted_after_processing() {
        let tmp = temp_events_dir();
        let mut watcher = EventWatcher::new(tmp.path().to_path_buf())
            .expect("EventWatcher::new failed");
        watcher.start().expect("start failed");

        let file_path = write_event_file(tmp.path(), "evt2.json", &session_start_json("del-test"));

        // Wait until the event arrives (i.e. the watcher has processed the file).
        let event = poll_for_event(&watcher, Duration::from_secs(5));
        watcher.stop();

        assert!(event.is_some(), "expected event to be delivered");
        assert!(
            !file_path.exists(),
            "event file should have been deleted after processing"
        );
    }

    // ------------------------------------------------------------------
    // Invalid file is deleted after a parse failure
    // ------------------------------------------------------------------

    /// An unparseable file should be deleted and no event should be delivered.
    #[test]
    fn invalid_event_file_is_deleted_and_no_event_delivered() {
        let tmp = temp_events_dir();
        let mut watcher = EventWatcher::new(tmp.path().to_path_buf())
            .expect("EventWatcher::new failed");
        watcher.start().expect("start failed");

        let file_path = write_event_file(tmp.path(), "bad.json", "this is not valid json {{{");

        // Give the watcher time to process the bad file.
        std::thread::sleep(Duration::from_millis(500));

        let event = watcher.try_recv();
        watcher.stop();

        assert!(
            event.is_none(),
            "no event should be delivered for an invalid file"
        );
        assert!(
            !file_path.exists(),
            "invalid event file should have been deleted"
        );
    }

    // ------------------------------------------------------------------
    // stop() can be called when already stopped
    // ------------------------------------------------------------------

    /// Calling `stop` on a watcher that was never started (or already stopped)
    /// is a no-op and does not panic.
    #[test]
    fn stop_is_idempotent() {
        let tmp = temp_events_dir();
        let mut watcher = EventWatcher::new(tmp.path().to_path_buf())
            .expect("EventWatcher::new failed");
        watcher.stop(); // never started — should be fine
        watcher.stop(); // second call — still fine
    }

    // ------------------------------------------------------------------
    // Internal helper
    // ------------------------------------------------------------------

    /// Polls `try_recv` repeatedly until an event arrives or the timeout elapses.
    fn poll_for_event(watcher: &EventWatcher, timeout: Duration) -> Option<Event> {
        let start = std::time::Instant::now();
        let poll_interval = Duration::from_millis(50);

        loop {
            if let Some(event) = watcher.try_recv() {
                return Some(event);
            }
            if start.elapsed() >= timeout {
                return None;
            }
            std::thread::sleep(poll_interval);
        }
    }
}

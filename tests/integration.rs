/// End-to-end integration tests for the full pipeline:
/// event file → EventWatcher → Office → GnoemState.
///
/// These tests exercise the real filesystem and the notify-based watcher, so
/// they sleep briefly to let the OS watcher deliver events.  All I/O is
/// isolated to temporary directories that are removed when each test ends.
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use gnoem::config::{AnimationConfig, Config, DisplayConfig, PathsConfig};
use gnoem::events::Event;
use gnoem::office::{GnoemState, Office};
use gnoem::persistence::GnoemRegistry;
use gnoem::watcher::EventWatcher;

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Build a `Config` whose `events_dir` points at `events_dir`.
fn config_for(events_dir: PathBuf) -> Config {
    Config {
        display: DisplayConfig::default(),
        animation: AnimationConfig::default(),
        paths: PathsConfig { events_dir },
    }
}

/// Build an in-memory `Office` (no registry file on disk).
fn make_office(events_dir: PathBuf) -> Office {
    let registry = GnoemRegistry::new(PathBuf::from("/dev/null"));
    let config = config_for(events_dir);
    Office::new(registry, config)
}

/// Atomically write `content` to `dir/<filename>` and flush to disk so the OS
/// watcher sees a complete file before we start polling.
fn write_event_file(dir: &Path, filename: &str, content: &str) -> PathBuf {
    let path = dir.join(filename);
    let mut f = std::fs::File::create(&path).expect("create event file");
    f.write_all(content.as_bytes()).expect("write event file");
    f.flush().expect("flush event file");
    f.sync_all().expect("sync event file");
    path
}

/// Poll `watcher.try_recv()` until an event arrives or `timeout` elapses.
fn poll_for_event(watcher: &EventWatcher, timeout: Duration) -> Option<Event> {
    let deadline = std::time::Instant::now() + timeout;
    let interval = Duration::from_millis(50);
    loop {
        if let Some(event) = watcher.try_recv() {
            return Some(event);
        }
        if std::time::Instant::now() >= deadline {
            return None;
        }
        std::thread::sleep(interval);
    }
}

/// JSON for a `SessionStart` event.
fn session_start_json(session_id: &str, cwd: &str, timestamp: u64) -> String {
    format!(
        r#"{{"event":"session_start","session_id":"{session_id}","cwd":"{cwd}","timestamp":{timestamp}}}"#
    )
}

/// JSON for a `PostToolUse` event.
fn post_tool_use_json(session_id: &str, cwd: &str, timestamp: u64, tool_name: &str) -> String {
    format!(
        r#"{{"event":"post_tool_use","session_id":"{session_id}","cwd":"{cwd}","timestamp":{timestamp},"tool_name":"{tool_name}"}}"#
    )
}

/// JSON for a `SessionEnd` event.
fn session_end_json(session_id: &str, cwd: &str, timestamp: u64) -> String {
    format!(
        r#"{{"event":"session_end","session_id":"{session_id}","cwd":"{cwd}","timestamp":{timestamp}}}"#
    )
}

// ---------------------------------------------------------------------------
// Test: single session full lifecycle
// ---------------------------------------------------------------------------

/// The main lifecycle test:
/// SessionStart → Entering
/// PostToolUse  → Typing, paper on desk
/// SessionEnd   → PackingUp
/// tick enough  → Gnoem removed (Gone)
#[test]
fn full_lifecycle_session_start_tool_use_session_end_then_gone() {
    let tmp = tempfile::tempdir().expect("create tempdir");
    let events_dir = tmp.path().to_path_buf();

    let mut watcher = EventWatcher::new(events_dir.clone()).expect("EventWatcher::new");
    watcher.start().expect("watcher start");

    let mut office = make_office(events_dir.clone());

    // -- Step 1: SessionStart ------------------------------------------------
    write_event_file(
        &events_dir,
        "01_session_start.json",
        &session_start_json("sess-1", "/project/alpha", 1000),
    );

    let event = poll_for_event(&watcher, Duration::from_secs(5))
        .expect("timed out waiting for SessionStart event");
    office.process_event(event);

    assert_eq!(office.gnoems().len(), 1, "expected 1 gnoem after SessionStart");
    let gnoem = &office.gnoems()[0];
    assert_eq!(gnoem.session_id, "sess-1");
    assert_eq!(
        gnoem.state_machine.state,
        GnoemState::Entering,
        "gnoem should start in Entering state"
    );

    // -- Step 2: PostToolUse -------------------------------------------------
    write_event_file(
        &events_dir,
        "02_post_tool_use.json",
        &post_tool_use_json("sess-1", "/project/alpha", 1001, "Bash"),
    );

    let event = poll_for_event(&watcher, Duration::from_secs(5))
        .expect("timed out waiting for PostToolUse event");
    office.process_event(event);

    let gnoem = &office.gnoems()[0];
    assert_eq!(
        gnoem.state_machine.state,
        GnoemState::Typing,
        "gnoem should be Typing after PostToolUse"
    );
    assert!(
        gnoem.desk.paper_stack > 0,
        "desk should have papers after PostToolUse"
    );
    assert!(gnoem.desk.monitor_typing, "monitor should show typing indicator");

    // -- Step 3: SessionEnd --------------------------------------------------
    write_event_file(
        &events_dir,
        "03_session_end.json",
        &session_end_json("sess-1", "/project/alpha", 2000),
    );

    let event = poll_for_event(&watcher, Duration::from_secs(5))
        .expect("timed out waiting for SessionEnd event");
    office.process_event(event);

    let gnoem = &office.gnoems()[0];
    assert_eq!(
        gnoem.state_machine.state,
        GnoemState::PackingUp,
        "gnoem should be PackingUp after SessionEnd"
    );

    // -- Step 4: Tick until Gone and removed ---------------------------------
    // PackingUp (2 s) → Leaving (2 s) → Gone.
    // The state machine works in seconds, so advance 10 s past the timestamp
    // used in SessionEnd (2000) to ensure both transitions fire.
    office.tick(2010);
    // After the first tick at t=2010 the gnoem has been in PackingUp since
    // t=2000 (10 s elapsed >= 2 s threshold), so it transitions to Leaving.
    // Tick again a further 3 s ahead to trigger Leaving → Gone.
    office.tick(2013);
    // tick() also retains only non-Gone gnoems, so after Gone is reached on
    // this second tick the gnoem is purged.
    office.tick(2014); // ensure retain runs after Gone is set

    assert!(
        office.gnoems().is_empty(),
        "gnoem should be removed from the office after reaching Gone"
    );

    watcher.stop();
}

// ---------------------------------------------------------------------------
// Test: watcher deletes consumed event files
// ---------------------------------------------------------------------------

/// After the watcher delivers an event the source file must no longer exist.
#[test]
fn watcher_deletes_event_file_after_delivering_event() {
    let tmp = tempfile::tempdir().expect("create tempdir");
    let events_dir = tmp.path().to_path_buf();

    let mut watcher = EventWatcher::new(events_dir.clone()).expect("EventWatcher::new");
    watcher.start().expect("watcher start");

    let file_path = write_event_file(
        &events_dir,
        "delete_me.json",
        &session_start_json("del-sess", "/project/delete", 5000),
    );

    // Wait until the event is delivered (i.e. the file has been processed).
    let event = poll_for_event(&watcher, Duration::from_secs(5));
    watcher.stop();

    assert!(event.is_some(), "event should have been delivered by the watcher");
    assert!(
        !file_path.exists(),
        "the source event file should be deleted after the watcher processes it"
    );
}

// ---------------------------------------------------------------------------
// Test: multiple concurrent sessions
// ---------------------------------------------------------------------------

/// Two sessions can run concurrently inside the same Office without
/// interfering with each other.
#[test]
fn multiple_concurrent_sessions_are_tracked_independently() {
    let tmp = tempfile::tempdir().expect("create tempdir");
    let events_dir = tmp.path().to_path_buf();

    let mut watcher = EventWatcher::new(events_dir.clone()).expect("EventWatcher::new");
    watcher.start().expect("watcher start");

    let mut office = make_office(events_dir.clone());

    // Start two sessions concurrently.
    write_event_file(
        &events_dir,
        "alpha_start.json",
        &session_start_json("alpha", "/project/alpha", 1000),
    );
    let ev = poll_for_event(&watcher, Duration::from_secs(5))
        .expect("timed out waiting for alpha SessionStart");
    office.process_event(ev);

    write_event_file(
        &events_dir,
        "beta_start.json",
        &session_start_json("beta", "/project/beta", 1001),
    );
    let ev = poll_for_event(&watcher, Duration::from_secs(5))
        .expect("timed out waiting for beta SessionStart");
    office.process_event(ev);

    assert_eq!(office.gnoems().len(), 2, "expected 2 concurrent gnoems");

    // PostToolUse for alpha only — beta should remain in Entering.
    write_event_file(
        &events_dir,
        "alpha_tool.json",
        &post_tool_use_json("alpha", "/project/alpha", 1002, "Read"),
    );
    let ev = poll_for_event(&watcher, Duration::from_secs(5))
        .expect("timed out waiting for alpha PostToolUse");
    office.process_event(ev);

    let alpha = office.gnoems().iter().find(|g| g.session_id == "alpha")
        .expect("alpha gnoem not found");
    assert_eq!(alpha.state_machine.state, GnoemState::Typing,
        "alpha should be Typing after PostToolUse");

    let beta = office.gnoems().iter().find(|g| g.session_id == "beta")
        .expect("beta gnoem not found");
    assert_eq!(beta.state_machine.state, GnoemState::Entering,
        "beta should still be Entering — it received no events");

    // End alpha's session; beta should remain unaffected.
    write_event_file(
        &events_dir,
        "alpha_end.json",
        &session_end_json("alpha", "/project/alpha", 2000),
    );
    let ev = poll_for_event(&watcher, Duration::from_secs(5))
        .expect("timed out waiting for alpha SessionEnd");
    office.process_event(ev);

    let alpha = office.gnoems().iter().find(|g| g.session_id == "alpha")
        .expect("alpha gnoem not found");
    assert_eq!(alpha.state_machine.state, GnoemState::PackingUp,
        "alpha should be PackingUp after SessionEnd");

    let beta = office.gnoems().iter().find(|g| g.session_id == "beta")
        .expect("beta gnoem not found");
    assert_eq!(beta.state_machine.state, GnoemState::Entering,
        "beta should still be Entering after alpha ends");

    assert_eq!(office.gnoems().len(), 2,
        "both gnoems still present — alpha packing up, beta still running");

    watcher.stop();
}

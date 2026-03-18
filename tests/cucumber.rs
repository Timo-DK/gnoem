use cucumber::{given, then, when, World};
use gnoem::config::Config;
use gnoem::events::Event;
use gnoem::office::{GnoemState, Office};
use gnoem::persistence::{GnoemIdentity, GnoemRegistry};
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// World
// ---------------------------------------------------------------------------

/// Shared test state passed through every step in a scenario.
#[derive(Debug, Default, World)]
pub struct GnoemWorld {
    office: Option<Office>,
}

impl GnoemWorld {
    /// Return a mutable reference to the inner `Office`, panicking if it has
    /// not been initialised yet (i.e. a `Given` step was missed).
    fn office_mut(&mut self) -> &mut Office {
        self.office
            .as_mut()
            .expect("office not initialised — did you include a 'Given the office is empty' step?")
    }

    /// Return a shared reference to the inner `Office`.
    fn office(&self) -> &Office {
        self.office
            .as_ref()
            .expect("office not initialised — did you include a 'Given the office is empty' step?")
    }

    /// Find a gnoem by session id, panicking with a helpful message if absent.
    fn find_gnoem(&self, session_id: &str) -> &gnoem::office::Gnoem {
        self.office()
            .gnoems()
            .iter()
            .find(|g| g.session_id == session_id)
            .unwrap_or_else(|| panic!("no gnoem with session id '{session_id}' in the office"))
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_registry() -> GnoemRegistry {
    GnoemRegistry::new(PathBuf::from("/dev/null"))
}

fn make_office() -> Office {
    Office::new(make_registry(), Config::default())
}

/// Construct a `SessionStart` event at a fixed timestamp.
fn session_start_event(session_id: &str, cwd: &str) -> Event {
    Event::SessionStart {
        session_id: session_id.to_owned(),
        cwd: cwd.to_owned(),
        timestamp: 1_000,
    }
}

/// Construct a `PostToolUse` event at a fixed timestamp.
fn tool_use_event(session_id: &str, tool_name: &str) -> Event {
    Event::PostToolUse {
        session_id: session_id.to_owned(),
        cwd: "/proj".to_owned(),
        timestamp: 2_000,
        tool_name: tool_name.to_owned(),
    }
}

/// Construct a `PostToolUseFailure` event at a fixed timestamp.
fn tool_failure_event(session_id: &str, tool_name: &str) -> Event {
    Event::PostToolUseFailure {
        session_id: session_id.to_owned(),
        cwd: "/proj".to_owned(),
        timestamp: 2_000,
        tool_name: tool_name.to_owned(),
        error: "simulated failure".to_owned(),
    }
}

/// Parse a `GnoemState` variant from a string, panicking on unknown names.
fn parse_state(name: &str) -> GnoemState {
    match name {
        "Entering" => GnoemState::Entering,
        "SittingDown" => GnoemState::SittingDown,
        "Idle" => GnoemState::Idle,
        "Typing" => GnoemState::Typing,
        "Jumping" => GnoemState::Jumping,
        "HeadScratching" => GnoemState::HeadScratching,
        "ReceivingLetter" => GnoemState::ReceivingLetter,
        "LeaningBack" => GnoemState::LeaningBack,
        "ShufflingPapers" => GnoemState::ShufflingPapers,
        "PackingUp" => GnoemState::PackingUp,
        "Leaving" => GnoemState::Leaving,
        "Gone" => GnoemState::Gone,
        other => panic!("unknown GnoemState variant: '{other}'"),
    }
}

// ---------------------------------------------------------------------------
// Given steps
// ---------------------------------------------------------------------------

/// `Given the office is empty`
#[given("the office is empty")]
fn given_office_is_empty(world: &mut GnoemWorld) {
    world.office = Some(make_office());
}

/// `Given a gnoem is in the office with session id {string}`
#[given(expr = "a gnoem is in the office with session id {string}")]
fn given_gnoem_in_office(world: &mut GnoemWorld, session_id: String) {
    let mut office = make_office();
    office.process_event(session_start_event(&session_id, "/home/user/project"));
    world.office = Some(office);
}

/// `Given the office has a registered gnoem named {string} for {string}`
///
/// Pre-populates the registry so that the named project directory maps to the
/// given name, then creates a fresh (empty) office backed by that registry.
#[given(expr = "the office has a registered gnoem named {string} for {string}")]
fn given_office_has_registered_gnoem(world: &mut GnoemWorld, name: String, cwd: String) {
    let mut registry = make_registry();
    registry.insert(
        cwd,
        GnoemIdentity {
            name,
            color: "cyan".to_owned(),
        },
    );
    world.office = Some(Office::new(registry, Config::default()));
}

// ---------------------------------------------------------------------------
// When steps
// ---------------------------------------------------------------------------

/// `When a session starts for {string} with id {string}`
#[when(expr = "a session starts for {string} with id {string}")]
fn when_session_starts(world: &mut GnoemWorld, cwd: String, session_id: String) {
    world
        .office_mut()
        .process_event(session_start_event(&session_id, &cwd));
}

/// `When a tool use event arrives for session {string} with tool {string}`
///
/// Also registered as `#[given]` so that `And a tool use event…` lines that
/// appear inside a `Given` block (e.g. the desk compaction scenario) are
/// dispatched correctly.
#[when(expr = "a tool use event arrives for session {string} with tool {string}")]
fn when_tool_use(world: &mut GnoemWorld, session_id: String, tool_name: String) {
    world
        .office_mut()
        .process_event(tool_use_event(&session_id, &tool_name));
}

/// `Given a tool use event arrives for session {string} with tool {string}`
///
/// Cucumber normalises `And`/`But` to the surrounding keyword. When these
/// steps appear as `And` after a `Given` they are dispatched as `Given`.
#[given(expr = "a tool use event arrives for session {string} with tool {string}")]
fn given_tool_use(world: &mut GnoemWorld, session_id: String, tool_name: String) {
    world
        .office_mut()
        .process_event(tool_use_event(&session_id, &tool_name));
}

/// `Given a subagent start event arrives for session {string} with subagent {string}`
///
/// Registered as `#[given]` so that `And a subagent start event…` lines that
/// appear inside a `Given` block (lifecycle subagent ending scenario) work.
#[given(expr = "a subagent start event arrives for session {string} with subagent {string}")]
fn given_subagent_start(world: &mut GnoemWorld, session_id: String, subagent_id: String) {
    world.office_mut().process_event(Event::SubagentStart {
        session_id,
        cwd: "/proj".to_owned(),
        timestamp: 2_000,
        subagent_id,
    });
}

/// `When a tool failure event arrives for session {string} with tool {string}`
#[when(expr = "a tool failure event arrives for session {string} with tool {string}")]
fn when_tool_failure(world: &mut GnoemWorld, session_id: String, tool_name: String) {
    world
        .office_mut()
        .process_event(tool_failure_event(&session_id, &tool_name));
}

/// `When a notification event arrives for session {string}`
#[when(expr = "a notification event arrives for session {string}")]
fn when_notification(world: &mut GnoemWorld, session_id: String) {
    world.office_mut().process_event(Event::Notification {
        session_id,
        cwd: "/proj".to_owned(),
        timestamp: 2_000,
        message: "task complete".to_owned(),
    });
}

/// `When a user prompt event arrives for session {string}`
#[when(expr = "a user prompt event arrives for session {string}")]
fn when_user_prompt(world: &mut GnoemWorld, session_id: String) {
    world.office_mut().process_event(Event::UserPromptSubmit {
        session_id,
        cwd: "/proj".to_owned(),
        timestamp: 2_000,
    });
}

/// `When a session end event arrives for session {string}`
#[when(expr = "a session end event arrives for session {string}")]
fn when_session_end(world: &mut GnoemWorld, session_id: String) {
    world.office_mut().process_event(Event::SessionEnd {
        session_id,
        cwd: "/proj".to_owned(),
        timestamp: 2_000,
    });
}

/// `When a subagent start event arrives for session {string} with subagent {string}`
#[when(expr = "a subagent start event arrives for session {string} with subagent {string}")]
fn when_subagent_start(world: &mut GnoemWorld, session_id: String, subagent_id: String) {
    world.office_mut().process_event(Event::SubagentStart {
        session_id,
        cwd: "/proj".to_owned(),
        timestamp: 2_000,
        subagent_id,
    });
}

/// `When a subagent end event arrives for session {string} with subagent {string}`
#[when(expr = "a subagent end event arrives for session {string} with subagent {string}")]
fn when_subagent_end(world: &mut GnoemWorld, session_id: String, subagent_id: String) {
    world.office_mut().process_event(Event::SubagentEnd {
        session_id,
        cwd: "/proj".to_owned(),
        timestamp: 3_000,
        subagent_id,
    });
}

/// `When a pre-compact event arrives for session {string}`
#[when(expr = "a pre-compact event arrives for session {string}")]
fn when_pre_compact(world: &mut GnoemWorld, session_id: String) {
    world.office_mut().process_event(Event::PreCompact {
        session_id,
        cwd: "/proj".to_owned(),
        timestamp: 3_000,
    });
}

// ---------------------------------------------------------------------------
// Then steps
// ---------------------------------------------------------------------------

/// `Then there should be {int} active gnoem(s)`
///
/// Matches both "0 active gnoems" and "1 active gnoem" (singular/plural).
#[then(expr = "there should be {int} active gnoem(s)")]
fn then_active_count(world: &mut GnoemWorld, expected: usize) {
    let actual = world.office().active_count();
    assert_eq!(
        actual, expected,
        "expected {expected} active gnoem(s), found {actual}"
    );
}

/// `Then the gnoem {string} should be in the {string} state`
#[then(expr = "the gnoem {string} should be in the {string} state")]
fn then_gnoem_state(world: &mut GnoemWorld, session_id: String, state_name: String) {
    let gnoem = world.find_gnoem(&session_id);
    let expected = parse_state(&state_name);
    let actual = &gnoem.state_machine.state;
    assert_eq!(
        actual, &expected,
        "gnoem '{session_id}' expected state {state_name:?}, found {actual:?}"
    );
}

/// `Then the gnoem {string} should be named {string}`
#[then(expr = "the gnoem {string} should be named {string}")]
fn then_gnoem_named(world: &mut GnoemWorld, session_id: String, expected_name: String) {
    let gnoem = world.find_gnoem(&session_id);
    assert_eq!(
        gnoem.name, expected_name,
        "gnoem '{session_id}' expected name '{expected_name}', found '{}'",
        gnoem.name
    );
}

/// `Then the last activity for {string} should be {string}`
#[then(expr = "the last activity for {string} should be {string}")]
fn then_last_activity(world: &mut GnoemWorld, session_id: String, expected: String) {
    let gnoem = world.find_gnoem(&session_id);
    assert_eq!(
        gnoem.last_activity, expected,
        "gnoem '{session_id}' expected last_activity '{expected}', found '{}'",
        gnoem.last_activity
    );
}

/// `Then the desk for {string} should have an envelope`
#[then(expr = "the desk for {string} should have an envelope")]
fn then_desk_has_envelope(world: &mut GnoemWorld, session_id: String) {
    let gnoem = world.find_gnoem(&session_id);
    assert!(
        gnoem.desk.has_envelope,
        "gnoem '{session_id}' desk should have an envelope, but has_envelope is false"
    );
}

/// `Then the desk for {string} should have {int} papers`
#[then(expr = "the desk for {string} should have {int} papers")]
fn then_desk_paper_count(world: &mut GnoemWorld, session_id: String, expected: u32) {
    let gnoem = world.find_gnoem(&session_id);
    assert_eq!(
        gnoem.desk.paper_stack, expected,
        "gnoem '{session_id}' desk expected {expected} papers, found {}",
        gnoem.desk.paper_stack
    );
}

/// `Then the desk for {string} should show a monitor error`
#[then(expr = "the desk for {string} should show a monitor error")]
fn then_desk_monitor_error(world: &mut GnoemWorld, session_id: String) {
    let gnoem = world.find_gnoem(&session_id);
    assert!(
        gnoem.desk.monitor_error,
        "gnoem '{session_id}' desk should show a monitor error, but monitor_error is false"
    );
}

/// `Then the gnoem {string} should have {int} mini gnoem(s)`
///
/// Matches both singular and plural ("1 mini gnoem" / "0 mini gnoems").
#[then(expr = "the gnoem {string} should have {int} mini gnoem(s)")]
fn then_mini_gnoem_count(world: &mut GnoemWorld, session_id: String, expected: usize) {
    let gnoem = world.find_gnoem(&session_id);
    let actual = gnoem.mini_gnoems.len();
    assert_eq!(
        actual, expected,
        "gnoem '{session_id}' expected {expected} mini gnoem(s), found {actual}"
    );
}

// ---------------------------------------------------------------------------
// Also register And-prefixed steps that share implementations with When steps.
// Cucumber automatically handles And/But as aliases for the previous keyword,
// so no extra registration is needed — the macros cover Given/When/Then and
// cucumber treats And/But transparently.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    futures::executor::block_on(GnoemWorld::run("tests/features"));
}

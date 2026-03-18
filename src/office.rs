/// Office model: manages all Gnoems and processes incoming events.
///
/// This module is self-contained. It embeds the state machine (`GnoemStateMachine`,
/// `GnoemState`) and desk environment (`DeskEnvironment`) types that will be
/// extracted into separate crates in a later refactor.  Keeping them here lets
/// the module compile without modifying `src/lib.rs`.
use rand::thread_rng;
use ratatui::style::Color;

use crate::config::Config;
use crate::events::Event;
use crate::names::generate_gnoem_identity;
use crate::persistence::{GnoemIdentity, GnoemRegistry, color_to_string, string_to_color};

// ---------------------------------------------------------------------------
// GnoemState
// ---------------------------------------------------------------------------

/// Every possible visual / behavioural state a Gnoem can be in.
#[derive(Debug, Clone, PartialEq)]
pub enum GnoemState {
    Entering,
    SittingDown,
    Idle,
    Typing,
    Jumping,
    HeadScratching,
    ReceivingLetter,
    LeaningBack,
    ShufflingPapers,
    PackingUp,
    Leaving,
    /// Terminal state — Gnoem has left the office and will be removed from the
    /// active list on the next `Office::tick`.
    Gone,
}

impl GnoemState {
    /// Milliseconds before the state automatically transitions to the next one.
    /// Returns `None` for states that require an external trigger.
    pub fn auto_transition_ms(&self) -> Option<u64> {
        match self {
            GnoemState::Entering => Some(2_000),
            GnoemState::SittingDown => Some(1_000),
            GnoemState::Jumping => Some(1_500),
            GnoemState::HeadScratching => Some(2_000),
            GnoemState::ReceivingLetter => Some(1_500),
            GnoemState::LeaningBack => Some(2_000),
            GnoemState::ShufflingPapers => Some(2_000),
            GnoemState::PackingUp => Some(2_000),
            GnoemState::Leaving => Some(2_000),
            GnoemState::Idle | GnoemState::Typing | GnoemState::Gone => None,
        }
    }

    /// State to transition to once `auto_transition_ms` has elapsed.
    pub fn auto_next(&self) -> Option<GnoemState> {
        match self {
            GnoemState::Entering => Some(GnoemState::SittingDown),
            GnoemState::SittingDown => Some(GnoemState::Idle),
            GnoemState::Jumping => Some(GnoemState::Idle),
            GnoemState::HeadScratching => Some(GnoemState::Idle),
            GnoemState::ReceivingLetter => Some(GnoemState::Idle),
            GnoemState::LeaningBack => Some(GnoemState::Idle),
            GnoemState::ShufflingPapers => Some(GnoemState::Idle),
            GnoemState::PackingUp => Some(GnoemState::Leaving),
            GnoemState::Leaving => Some(GnoemState::Gone),
            _ => None,
        }
    }

    /// Number of animation frames for this state.
    pub fn frame_count(&self) -> usize {
        match self {
            GnoemState::Idle => 2,
            GnoemState::Typing => 4,
            GnoemState::Jumping => 3,
            GnoemState::HeadScratching => 2,
            GnoemState::Entering | GnoemState::Leaving => 4,
            _ => 2,
        }
    }
}

// ---------------------------------------------------------------------------
// GnoemStateMachine
// ---------------------------------------------------------------------------

/// Drives state transitions for a single Gnoem.
///
/// Time is tracked via Unix timestamps (`u64` seconds) so the machine can be
/// tested deterministically without sleeping.
#[derive(Debug, Clone)]
pub struct GnoemStateMachine {
    pub state: GnoemState,
    /// Unix timestamp (seconds) when the current state was entered.
    pub state_entered_at: u64,
    /// Current animation frame index within the active state.
    pub animation_frame: usize,
}

impl GnoemStateMachine {
    /// Create a new state machine starting in `Entering`.
    pub fn new(timestamp: u64) -> Self {
        Self {
            state: GnoemState::Entering,
            state_entered_at: timestamp,
            animation_frame: 0,
        }
    }

    /// Transition to `new_state`, resetting the frame counter and recording
    /// `current_time` as the moment the state was entered.
    pub fn transition_to(&mut self, new_state: GnoemState, current_time: u64) {
        self.state = new_state;
        self.animation_frame = 0;
        self.state_entered_at = current_time;
    }

    /// Advance the animation frame, wrapping at `state.frame_count()`.
    pub fn advance_frame(&mut self) {
        self.animation_frame = (self.animation_frame + 1) % self.state.frame_count();
    }

    /// Apply any pending automatic state transition based on elapsed time.
    /// Returns `true` if a transition occurred.
    pub fn tick(&mut self, current_time: u64) -> bool {
        if let (Some(duration_ms), Some(next)) =
            (self.state.auto_transition_ms(), self.state.auto_next())
        {
            let elapsed_ms = current_time.saturating_sub(self.state_entered_at) * 1_000;
            if elapsed_ms >= duration_ms {
                self.transition_to(next, current_time);
                return true;
            }
        }
        false
    }

    /// Returns `true` when the Gnoem has reached the terminal `Gone` state.
    pub fn is_gone(&self) -> bool {
        self.state == GnoemState::Gone
    }
}

// ---------------------------------------------------------------------------
// DeskEnvironment
// ---------------------------------------------------------------------------

/// Visual state of a Gnoem's desk props.
#[derive(Debug, Clone)]
pub struct DeskEnvironment {
    /// Coffee fill level, `0.0` (empty) to `1.0` (full).
    pub coffee_level: f32,
    /// Number of papers on the desk (capped at 10).
    pub paper_stack: u32,
    /// Plant health, `0.0` (wilted) to `1.0` (thriving).
    pub plant_health: f32,
    /// Monitor is showing an error indicator.
    pub monitor_error: bool,
    /// Monitor is showing a typing / busy indicator.
    pub monitor_typing: bool,
    /// A letter / envelope is sitting on the desk (after `UserPromptSubmit`).
    pub has_envelope: bool,
}

impl Default for DeskEnvironment {
    fn default() -> Self {
        Self {
            coffee_level: 0.5,
            paper_stack: 0,
            plant_health: 1.0,
            monitor_error: false,
            monitor_typing: false,
            has_envelope: false,
        }
    }
}

impl DeskEnvironment {
    pub fn on_tool_use(&mut self) {
        self.paper_stack = (self.paper_stack + 1).min(10);
        self.monitor_typing = true;
        self.monitor_error = false;
        self.has_envelope = false;
        self.recover_plant();
    }

    pub fn on_tool_failure(&mut self) {
        self.monitor_error = true;
        self.monitor_typing = false;
    }

    pub fn on_agent_stop(&mut self) {
        self.coffee_level = (self.coffee_level + 0.25).min(1.0);
    }

    pub fn on_compact(&mut self) {
        self.paper_stack = self.paper_stack.saturating_sub(3);
    }

    pub fn on_prompt_submit(&mut self) {
        self.has_envelope = true;
        self.monitor_typing = false;
        self.monitor_error = false;
    }

    pub fn on_idle_tick(&mut self, wilt_progress: f32) {
        self.plant_health = (1.0 - wilt_progress).max(0.0);
        self.monitor_typing = false;
    }

    fn recover_plant(&mut self) {
        self.plant_health = (self.plant_health + 0.1).min(1.0);
    }
}

// ---------------------------------------------------------------------------
// MiniGnoem
// ---------------------------------------------------------------------------

/// A lightweight representation of a subagent spawned by a parent Gnoem.
#[derive(Debug, Clone, PartialEq)]
pub struct MiniGnoem {
    pub subagent_id: String,
    pub active: bool,
}

// ---------------------------------------------------------------------------
// Gnoem
// ---------------------------------------------------------------------------

/// A single Claude session represented as an ASCII gnome working at a desk.
#[derive(Debug)]
pub struct Gnoem {
    pub name: String,
    pub color: Color,
    pub session_id: String,
    pub cwd: String,
    pub state_machine: GnoemStateMachine,
    pub desk: DeskEnvironment,
    /// Unix timestamp (seconds) when the session started.
    pub started_at: u64,
    /// Human-readable label for the most recent activity.
    pub last_activity: String,
    pub mini_gnoems: Vec<MiniGnoem>,
}

impl Gnoem {
    /// Create a new Gnoem entering the office.
    pub fn new(
        name: String,
        color: Color,
        session_id: String,
        cwd: String,
        timestamp: u64,
    ) -> Self {
        Self {
            name,
            color,
            session_id,
            cwd,
            state_machine: GnoemStateMachine::new(timestamp),
            desk: DeskEnvironment::default(),
            started_at: timestamp,
            last_activity: "session started".into(),
            mini_gnoems: Vec::new(),
        }
    }

    /// Elapsed seconds since the session started.
    pub fn duration_secs(&self, current_time: u64) -> u64 {
        current_time.saturating_sub(self.started_at)
    }
}

// ---------------------------------------------------------------------------
// Office
// ---------------------------------------------------------------------------

/// Owns all active Gnoems, processes events, and drives per-tick updates.
#[derive(Debug)]
pub struct Office {
    gnoems: Vec<Gnoem>,
    registry: GnoemRegistry,
    #[allow(dead_code)]
    config: Config,
}

impl Office {
    /// Create an empty office.
    pub fn new(registry: GnoemRegistry, config: Config) -> Self {
        Self {
            gnoems: Vec::new(),
            registry,
            config,
        }
    }

    // ------------------------------------------------------------------
    // Event processing
    // ------------------------------------------------------------------

    /// Route an incoming event to the appropriate Gnoem (or create one).
    pub fn process_event(&mut self, event: Event) {
        match event {
            // ---- lifecycle ---------------------------------------------------
            Event::SessionStart {
                session_id,
                cwd,
                timestamp,
            } => {
                let (name, color) = self.resolve_identity(&cwd);
                let gnoem = Gnoem::new(name, color, session_id, cwd, timestamp);
                self.gnoems.push(gnoem);
            }

            Event::SessionEnd {
                session_id,
                timestamp,
                ..
            } => {
                if let Some(g) = self.find_gnoem_mut(&session_id) {
                    g.last_activity = "session ended".into();
                    g.state_machine
                        .transition_to(GnoemState::PackingUp, timestamp);
                }
            }

            // ---- tool events -------------------------------------------------
            Event::PostToolUse {
                session_id,
                timestamp,
                tool_name,
                ..
            } => {
                if let Some(g) = self.find_gnoem_mut(&session_id) {
                    g.last_activity = format!("used {tool_name}");
                    g.state_machine.transition_to(GnoemState::Typing, timestamp);
                    g.desk.on_tool_use();
                }
            }

            Event::PostToolUseFailure {
                session_id,
                timestamp,
                tool_name,
                ..
            } => {
                if let Some(g) = self.find_gnoem_mut(&session_id) {
                    g.last_activity = format!("tool failure: {tool_name}");
                    g.state_machine
                        .transition_to(GnoemState::HeadScratching, timestamp);
                    g.desk.on_tool_failure();
                }
            }

            // ---- notifications / prompts ------------------------------------
            Event::Notification { session_id, timestamp, .. } => {
                if let Some(g) = self.find_gnoem_mut(&session_id) {
                    g.last_activity = "notification".into();
                    g.state_machine.transition_to(GnoemState::Jumping, timestamp);
                }
            }

            Event::UserPromptSubmit {
                session_id,
                timestamp,
                ..
            } => {
                if let Some(g) = self.find_gnoem_mut(&session_id) {
                    g.last_activity = "prompt submitted".into();
                    g.desk.on_prompt_submit();
                    g.state_machine
                        .transition_to(GnoemState::ReceivingLetter, timestamp);
                }
            }

            // ---- subagents ---------------------------------------------------
            Event::SubagentStart {
                session_id,
                subagent_id,
                ..
            } => {
                if let Some(g) = self.find_gnoem_mut(&session_id) {
                    g.mini_gnoems.push(MiniGnoem {
                        subagent_id,
                        active: true,
                    });
                }
            }

            Event::SubagentEnd {
                session_id,
                subagent_id,
                ..
            } => {
                if let Some(g) = self.find_gnoem_mut(&session_id) {
                    g.mini_gnoems.retain(|m| m.subagent_id != subagent_id);
                }
            }

            // ---- compaction / stop -------------------------------------------
            Event::PreCompact {
                session_id,
                timestamp,
                ..
            } => {
                if let Some(g) = self.find_gnoem_mut(&session_id) {
                    g.last_activity = "compacting context".into();
                    g.state_machine
                        .transition_to(GnoemState::ShufflingPapers, timestamp);
                    g.desk.on_compact();
                }
            }

            Event::Stop {
                session_id,
                timestamp,
                ..
            } => {
                if let Some(g) = self.find_gnoem_mut(&session_id) {
                    g.last_activity = "stopped".into();
                    g.state_machine
                        .transition_to(GnoemState::LeaningBack, timestamp);
                    g.desk.on_agent_stop();
                }
            }
        }
    }

    // ------------------------------------------------------------------
    // Tick
    // ------------------------------------------------------------------

    /// Advance all state machines, update desks, and purge Gone Gnoems.
    pub fn tick(&mut self, current_time: u64) {
        for gnoem in &mut self.gnoems {
            gnoem.state_machine.tick(current_time);
        }
        self.gnoems.retain(|g| !g.state_machine.is_gone());
    }

    // ------------------------------------------------------------------
    // Accessors
    // ------------------------------------------------------------------

    /// All currently tracked Gnoems (including those packing up / leaving).
    pub fn gnoems(&self) -> &[Gnoem] {
        &self.gnoems
    }

    /// Number of Gnoems that have not yet reached the `Gone` state.
    pub fn active_count(&self) -> usize {
        self.gnoems
            .iter()
            .filter(|g| !g.state_machine.is_gone())
            .count()
    }

    // ------------------------------------------------------------------
    // Private helpers
    // ------------------------------------------------------------------

    fn find_gnoem_mut(&mut self, session_id: &str) -> Option<&mut Gnoem> {
        self.gnoems
            .iter_mut()
            .find(|g| g.session_id == session_id)
    }

    /// Look up the persistent identity for `cwd`, generating and storing a new
    /// one if none exists yet.
    fn resolve_identity(&mut self, cwd: &str) -> (String, Color) {
        if let Some(identity) = self.registry.get(cwd) {
            let name = identity.name.clone();
            let color = string_to_color(&identity.color);
            return (name, color);
        }

        let mut rng = thread_rng();
        let (name, color) = generate_gnoem_identity(&mut rng);
        let identity = GnoemIdentity {
            name: name.clone(),
            color: color_to_string(color),
        };
        self.registry.insert(cwd.to_string(), identity);

        (name, color)
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn make_registry() -> GnoemRegistry {
        GnoemRegistry::new(std::path::PathBuf::from("/dev/null"))
    }

    fn make_office() -> Office {
        Office::new(make_registry(), Config::default())
    }

    fn session_start(session_id: &str, cwd: &str, ts: u64) -> Event {
        Event::SessionStart {
            session_id: session_id.into(),
            cwd: cwd.into(),
            timestamp: ts,
        }
    }

    fn session_end(session_id: &str, ts: u64) -> Event {
        Event::SessionEnd {
            session_id: session_id.into(),
            cwd: "/proj".into(),
            timestamp: ts,
        }
    }

    fn post_tool_use(session_id: &str, tool: &str, ts: u64) -> Event {
        Event::PostToolUse {
            session_id: session_id.into(),
            cwd: "/proj".into(),
            timestamp: ts,
            tool_name: tool.into(),
        }
    }

    fn post_tool_failure(session_id: &str, tool: &str, ts: u64) -> Event {
        Event::PostToolUseFailure {
            session_id: session_id.into(),
            cwd: "/proj".into(),
            timestamp: ts,
            tool_name: tool.into(),
            error: "failed".into(),
        }
    }

    // -----------------------------------------------------------------------
    // Office::new
    // -----------------------------------------------------------------------

    #[test]
    fn new_creates_empty_office() {
        let office = make_office();
        assert!(office.gnoems().is_empty());
        assert_eq!(office.active_count(), 0);
    }

    // -----------------------------------------------------------------------
    // SessionStart
    // -----------------------------------------------------------------------

    #[test]
    fn session_start_creates_a_gnoem() {
        let mut office = make_office();
        office.process_event(session_start("s1", "/home/user/proj", 1000));

        assert_eq!(office.gnoems().len(), 1);
        let g = &office.gnoems()[0];
        assert_eq!(g.session_id, "s1");
        assert_eq!(g.cwd, "/home/user/proj");
        assert_eq!(g.started_at, 1000);
        assert_eq!(g.state_machine.state, GnoemState::Entering);
    }

    #[test]
    fn session_start_with_known_cwd_reuses_name_from_registry() {
        let mut registry = make_registry();
        registry.insert(
            "/known/project".into(),
            GnoemIdentity {
                name: "Grumbold".into(),
                color: "cyan".into(),
            },
        );
        let mut office = Office::new(registry, Config::default());

        office.process_event(session_start("s1", "/known/project", 1000));

        let g = &office.gnoems()[0];
        assert_eq!(g.name, "Grumbold");
        assert_eq!(g.color, Color::Cyan);
    }

    #[test]
    fn session_start_with_unknown_cwd_generates_new_identity() {
        let mut office = make_office();
        office.process_event(session_start("s1", "/brand/new/project", 1000));

        let g = &office.gnoems()[0];
        assert!(!g.name.is_empty(), "name should be generated");
    }

    #[test]
    fn session_start_inserts_new_identity_into_registry() {
        let mut office = make_office();
        office.process_event(session_start("s1", "/new/project", 1000));

        // A second SessionStart for the same cwd should reuse the stored identity.
        let first_name = office.gnoems()[0].name.clone();
        office.process_event(session_start("s2", "/new/project", 1001));

        let second_name = office.gnoems()[1].name.clone();
        assert_eq!(first_name, second_name, "second session should reuse stored name");
    }

    // -----------------------------------------------------------------------
    // SessionEnd
    // -----------------------------------------------------------------------

    #[test]
    fn session_end_transitions_gnoem_to_packing_up() {
        let mut office = make_office();
        office.process_event(session_start("s1", "/proj", 1000));
        office.process_event(session_end("s1", 2000));

        let g = &office.gnoems()[0];
        assert_eq!(g.state_machine.state, GnoemState::PackingUp);
    }

    #[test]
    fn session_end_for_unknown_session_is_a_noop() {
        let mut office = make_office();
        office.process_event(session_start("s1", "/proj", 1000));
        office.process_event(session_end("unknown", 2000));

        // The real gnoem should be untouched.
        assert_eq!(office.gnoems()[0].state_machine.state, GnoemState::Entering);
    }

    // -----------------------------------------------------------------------
    // PostToolUse
    // -----------------------------------------------------------------------

    #[test]
    fn post_tool_use_transitions_to_typing_and_updates_desk() {
        let mut office = make_office();
        office.process_event(session_start("s1", "/proj", 1000));
        office.process_event(post_tool_use("s1", "Bash", 1001));

        let g = &office.gnoems()[0];
        assert_eq!(g.state_machine.state, GnoemState::Typing);
        assert_eq!(g.desk.paper_stack, 1);
        assert!(g.desk.monitor_typing);
        assert!(!g.desk.monitor_error);
        assert_eq!(g.last_activity, "used Bash");
    }

    // -----------------------------------------------------------------------
    // PostToolUseFailure
    // -----------------------------------------------------------------------

    #[test]
    fn post_tool_use_failure_transitions_to_head_scratching_and_updates_desk() {
        let mut office = make_office();
        office.process_event(session_start("s1", "/proj", 1000));
        office.process_event(post_tool_failure("s1", "Read", 1001));

        let g = &office.gnoems()[0];
        assert_eq!(g.state_machine.state, GnoemState::HeadScratching);
        assert!(g.desk.monitor_error);
        assert!(!g.desk.monitor_typing);
        assert_eq!(g.last_activity, "tool failure: Read");
    }

    // -----------------------------------------------------------------------
    // Notification
    // -----------------------------------------------------------------------

    #[test]
    fn notification_transitions_gnoem_to_jumping() {
        let mut office = make_office();
        office.process_event(session_start("s1", "/proj", 1000));
        office.process_event(Event::Notification {
            session_id: "s1".into(),
            cwd: "/proj".into(),
            timestamp: 1001,
            message: "Task done".into(),
        });

        assert_eq!(
            office.gnoems()[0].state_machine.state,
            GnoemState::Jumping
        );
    }

    // -----------------------------------------------------------------------
    // UserPromptSubmit
    // -----------------------------------------------------------------------

    #[test]
    fn user_prompt_submit_updates_desk_envelope() {
        let mut office = make_office();
        office.process_event(session_start("s1", "/proj", 1000));
        office.process_event(Event::UserPromptSubmit {
            session_id: "s1".into(),
            cwd: "/proj".into(),
            timestamp: 1001,
        });

        let g = &office.gnoems()[0];
        assert!(g.desk.has_envelope);
        assert_eq!(g.state_machine.state, GnoemState::ReceivingLetter);
    }

    // -----------------------------------------------------------------------
    // SubagentStart / SubagentEnd
    // -----------------------------------------------------------------------

    #[test]
    fn subagent_start_adds_mini_gnoem() {
        let mut office = make_office();
        office.process_event(session_start("s1", "/proj", 1000));
        office.process_event(Event::SubagentStart {
            session_id: "s1".into(),
            cwd: "/proj".into(),
            timestamp: 1001,
            subagent_id: "sub-1".into(),
        });

        let g = &office.gnoems()[0];
        assert_eq!(g.mini_gnoems.len(), 1);
        assert_eq!(g.mini_gnoems[0].subagent_id, "sub-1");
        assert!(g.mini_gnoems[0].active);
    }

    #[test]
    fn subagent_end_removes_mini_gnoem() {
        let mut office = make_office();
        office.process_event(session_start("s1", "/proj", 1000));
        office.process_event(Event::SubagentStart {
            session_id: "s1".into(),
            cwd: "/proj".into(),
            timestamp: 1001,
            subagent_id: "sub-1".into(),
        });
        office.process_event(Event::SubagentEnd {
            session_id: "s1".into(),
            cwd: "/proj".into(),
            timestamp: 1002,
            subagent_id: "sub-1".into(),
        });

        assert!(office.gnoems()[0].mini_gnoems.is_empty());
    }

    #[test]
    fn subagent_end_for_unknown_id_is_noop() {
        let mut office = make_office();
        office.process_event(session_start("s1", "/proj", 1000));
        office.process_event(Event::SubagentStart {
            session_id: "s1".into(),
            cwd: "/proj".into(),
            timestamp: 1001,
            subagent_id: "sub-1".into(),
        });
        office.process_event(Event::SubagentEnd {
            session_id: "s1".into(),
            cwd: "/proj".into(),
            timestamp: 1002,
            subagent_id: "does-not-exist".into(),
        });

        assert_eq!(office.gnoems()[0].mini_gnoems.len(), 1);
    }

    // -----------------------------------------------------------------------
    // PreCompact
    // -----------------------------------------------------------------------

    #[test]
    fn pre_compact_transitions_to_shuffling_papers_and_reduces_stack() {
        let mut office = make_office();
        office.process_event(session_start("s1", "/proj", 1000));
        // Build up some papers first.
        for ts in 1001..=1005 {
            office.process_event(post_tool_use("s1", "Bash", ts));
        }
        office.process_event(Event::PreCompact {
            session_id: "s1".into(),
            cwd: "/proj".into(),
            timestamp: 1010,
        });

        let g = &office.gnoems()[0];
        assert_eq!(g.state_machine.state, GnoemState::ShufflingPapers);
        assert_eq!(g.desk.paper_stack, 2); // 5 papers - 3 = 2
    }

    // -----------------------------------------------------------------------
    // Stop
    // -----------------------------------------------------------------------

    #[test]
    fn stop_transitions_gnoem_to_leaning_back_and_refills_coffee() {
        let mut office = make_office();
        office.process_event(session_start("s1", "/proj", 1000));
        // Drain the coffee first.
        office.gnoems[0].desk.coffee_level = 0.0;
        office.process_event(Event::Stop {
            session_id: "s1".into(),
            cwd: "/proj".into(),
            timestamp: 1001,
        });

        let g = &office.gnoems()[0];
        assert_eq!(g.state_machine.state, GnoemState::LeaningBack);
        assert_eq!(g.desk.coffee_level, 0.25);
    }

    // -----------------------------------------------------------------------
    // tick — auto-transitions and Gone removal
    // -----------------------------------------------------------------------

    #[test]
    fn tick_removes_gone_gnoems() {
        let mut office = make_office();
        office.process_event(session_start("s1", "/proj", 1000));

        // Force directly into Gone state.
        office.gnoems[0]
            .state_machine
            .transition_to(GnoemState::Gone, 9000);

        assert_eq!(office.gnoems().len(), 1);
        office.tick(9001);
        assert!(office.gnoems().is_empty(), "Gone gnoem should be removed on tick");
    }

    #[test]
    fn tick_does_not_remove_non_gone_gnoems() {
        let mut office = make_office();
        office.process_event(session_start("s1", "/proj", 1000));
        office.tick(1001); // Entering has 2000 ms auto-transition; not enough time
        assert_eq!(office.gnoems().len(), 1);
    }

    #[test]
    fn tick_advances_auto_transitions() {
        let mut office = make_office();
        // Timestamp 0 — Gnoem starts in Entering (auto-transitions to SittingDown after 2 s).
        office.process_event(session_start("s1", "/proj", 0));

        // Tick at t=3 — 3 seconds later, enough to pass the 2 s Entering threshold.
        office.tick(3);
        assert_eq!(
            office.gnoems()[0].state_machine.state,
            GnoemState::SittingDown,
            "Entering should have auto-transitioned to SittingDown"
        );
    }

    // -----------------------------------------------------------------------
    // active_count
    // -----------------------------------------------------------------------

    #[test]
    fn active_count_reflects_current_state() {
        let mut office = make_office();
        assert_eq!(office.active_count(), 0);

        office.process_event(session_start("s1", "/proj", 1000));
        assert_eq!(office.active_count(), 1);

        office.process_event(session_start("s2", "/other", 1001));
        assert_eq!(office.active_count(), 2);

        // Push one to Gone; active_count should drop on next read.
        office.gnoems[0]
            .state_machine
            .transition_to(GnoemState::Gone, 9000);
        // active_count filters Gone without requiring a tick.
        assert_eq!(office.active_count(), 1);

        // After tick the Gone gnoem is removed entirely.
        office.tick(9001);
        assert_eq!(office.active_count(), 1);
    }

    // -----------------------------------------------------------------------
    // Gnoem::duration_secs
    // -----------------------------------------------------------------------

    #[test]
    fn duration_secs_returns_elapsed_seconds() {
        let mut office = make_office();
        office.process_event(session_start("s1", "/proj", 1000));

        let g = &office.gnoems()[0];
        assert_eq!(g.duration_secs(1060), 60);
    }

    #[test]
    fn duration_secs_saturates_at_zero_for_past_timestamps() {
        let mut office = make_office();
        office.process_event(session_start("s1", "/proj", 1000));

        let g = &office.gnoems()[0];
        // current_time < started_at should not underflow.
        assert_eq!(g.duration_secs(500), 0);
    }

    // -----------------------------------------------------------------------
    // GnoemStateMachine — unit-level tests
    // -----------------------------------------------------------------------

    #[test]
    fn state_machine_new_starts_in_entering() {
        let sm = GnoemStateMachine::new(0);
        assert_eq!(sm.state, GnoemState::Entering);
        assert_eq!(sm.animation_frame, 0);
    }

    #[test]
    fn state_machine_transition_resets_frame() {
        let mut sm = GnoemStateMachine::new(0);
        sm.animation_frame = 3;
        sm.transition_to(GnoemState::Idle, 100);
        assert_eq!(sm.animation_frame, 0);
        assert_eq!(sm.state, GnoemState::Idle);
    }

    #[test]
    fn state_machine_tick_auto_transitions_entering_to_sitting_down() {
        let mut sm = GnoemStateMachine::new(0);
        // Entering auto-transitions after 2000 ms, i.e., 2 s.
        // current_time = 3 => elapsed = 3 s >= 2 s.
        let transitioned = sm.tick(3);
        assert!(transitioned);
        assert_eq!(sm.state, GnoemState::SittingDown);
    }

    #[test]
    fn state_machine_tick_does_not_transition_before_timeout() {
        let mut sm = GnoemStateMachine::new(0);
        // Entering auto-transitions after 2 s. Only 1 s has passed.
        let transitioned = sm.tick(1);
        assert!(!transitioned);
        assert_eq!(sm.state, GnoemState::Entering);
    }

    #[test]
    fn state_machine_leaving_transitions_to_gone() {
        let mut sm = GnoemStateMachine::new(0);
        sm.transition_to(GnoemState::Leaving, 0);
        // Leaving auto-transitions after 2 s.
        sm.tick(3);
        assert_eq!(sm.state, GnoemState::Gone);
        assert!(sm.is_gone());
    }

    #[test]
    fn state_machine_idle_does_not_auto_transition() {
        let mut sm = GnoemStateMachine::new(0);
        sm.transition_to(GnoemState::Idle, 0);
        let transitioned = sm.tick(9999);
        assert!(!transitioned);
        assert_eq!(sm.state, GnoemState::Idle);
    }
}

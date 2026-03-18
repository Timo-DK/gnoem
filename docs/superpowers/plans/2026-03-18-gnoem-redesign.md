# Gnoem Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a ratatui TUI that visualizes active Claude Code sessions as ASCII gnome creatures working in a virtual office, driven by file-watched events from Claude hooks.

**Architecture:** Monolithic single-binary with three internal layers: Data (file watcher, event parsing, config), Simulation (state machines, office model), and Rendering (ratatui via a `Renderer` trait). The trait boundary enables future graphical frontends.

**Tech Stack:** Rust, ratatui, crossterm, notify, serde/serde_json, toml, cucumber (BDD), clap, dirs, rand, chrono

**Spec:** `docs/superpowers/specs/2026-03-18-gnoem-redesign-design.md`

---

## File Structure

```
gnoem/
├── Cargo.toml
├── src/
│   ├── main.rs              # entry point, CLI (clap), delegates to app or CLI commands
│   ├── app.rs               # App struct, main tick loop, coordinates layers
│   ├── config.rs            # Config struct, TOML loading, defaults
│   ├── event/
│   │   ├── mod.rs           # re-exports
│   │   ├── schema.rs        # GnoemEvent enum + deserialization
│   │   └── watcher.rs       # EventWatcher using notify, reads+deletes event files
│   ├── simulation/
│   │   ├── mod.rs           # re-exports
│   │   ├── gnoem.rs         # Gnoem struct, GnoemState enum, state machine transitions
│   │   ├── office.rs        # Office struct, Cubicle, processes events → state changes
│   │   ├── desk.rs          # DeskEnvironment: coffee mug, paper stack, plant, monitor
│   │   ├── names.rs         # NameGenerator: prefix+suffix combiner
│   │   └── persistence.rs   # GnoemRegistry: load/save gnoems.toml, cwd→identity mapping
│   └── rendering/
│       ├── mod.rs           # Renderer trait + AppAction enum
│       ├── tui.rs           # RatatuiRenderer: terminal setup, main render fn
│       ├── sprites.rs       # ASCII art frames per GnoemState (2-4 frames each)
│       ├── cubicle.rs       # CubicleWidget: renders one cubicle with gnoem+desk+info
│       └── layout.rs        # grid layout calculator: terminal size → cubicle positions
├── tests/
│   ├── cucumber.rs          # cucumber test runner binary
│   ├── features/
│   │   ├── session_lifecycle.feature
│   │   ├── gnoem_reactions.feature
│   │   ├── subagents.feature
│   │   ├── desk_environment.feature
│   │   └── persistence.feature
│   ├── steps/
│   │   ├── mod.rs           # step definition module
│   │   ├── world.rs         # cucumber World struct
│   │   ├── session_steps.rs # Given/When/Then for session lifecycle
│   │   ├── reaction_steps.rs
│   │   ├── subagent_steps.rs
│   │   ├── desk_steps.rs
│   │   └── persistence_steps.rs
│   └── integration/
│       ├── event_watcher_test.rs
│       └── config_test.rs
├── hooks/
│   ├── gnoem-session-start.sh
│   ├── gnoem-session-end.sh
│   ├── gnoem-post-tool-use.sh
│   ├── gnoem-post-tool-use-failure.sh
│   ├── gnoem-user-prompt-submit.sh
│   ├── gnoem-notification.sh
│   ├── gnoem-subagent-start.sh
│   ├── gnoem-subagent-stop.sh
│   ├── gnoem-pre-compact.sh
│   └── gnoem-stop.sh
└── docs/
```

---

## Task 1: Project Scaffold

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `.gitignore`

- [ ] **Step 1: Initialize Cargo project**

```bash
cd E:/Personal/gnoem
cargo init --name gnoem
```

- [ ] **Step 2: Set up Cargo.toml with all dependencies**

Replace `Cargo.toml` with:

```toml
[package]
name = "gnoem"
version = "0.1.0"
edition = "2021"
description = "Visualize Claude Code sessions as ASCII gnomes in a virtual office"
license = "MIT"

[dependencies]
ratatui = "0.29"
crossterm = "0.28"
notify = "7"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
clap = { version = "4", features = ["derive"] }
dirs = "6"
rand = "0.8"
chrono = "0.4"

[dev-dependencies]
cucumber = "0.21"
tempfile = "3"
assert_fs = "1"

[[test]]
name = "cucumber"
harness = false
```

- [ ] **Step 3: Create minimal main.rs**

```rust
fn main() {
    println!("Gnoem - Claude Code Session Visualizer");
}
```

- [ ] **Step 4: Update .gitignore**

```
/target
.superpowers/
```

- [ ] **Step 5: Verify it compiles**

Run: `cargo build`
Expected: Compiles successfully

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock src/main.rs .gitignore
git commit -m "feat: initialize gnoem project with dependencies"
```

---

## Task 2: Event Schema and Parsing

**Files:**
- Create: `src/event/mod.rs`
- Create: `src/event/schema.rs`

- [ ] **Step 1: Write unit tests for event deserialization**

Create `src/event/schema.rs`:

```rust
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    SessionStart,
    SessionEnd,
    PostToolUse,
    PostToolUseFailure,
    UserPromptSubmit,
    Notification,
    SubagentStart,
    SubagentStop,
    PreCompact,
    Stop,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GnoemEvent {
    pub event: EventType,
    pub session_id: String,
    pub cwd: String,
    pub timestamp: u64,
    #[serde(default)]
    pub data: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_session_start_event() {
        let json = r#"{
            "event": "session_start",
            "session_id": "abc123",
            "cwd": "/home/user/project",
            "timestamp": 1710734400000,
            "data": {"model": "claude-opus-4-6", "source": "startup"}
        }"#;

        let event: GnoemEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.event, EventType::SessionStart);
        assert_eq!(event.session_id, "abc123");
        assert_eq!(event.cwd, "/home/user/project");
    }

    #[test]
    fn deserializes_all_event_types() {
        let types = vec![
            "session_start", "session_end", "post_tool_use",
            "post_tool_use_failure", "user_prompt_submit", "notification",
            "subagent_start", "subagent_stop", "pre_compact", "stop",
        ];
        for event_type in types {
            let json = format!(
                r#"{{"event":"{}","session_id":"s1","cwd":"/tmp","timestamp":0}}"#,
                event_type
            );
            let event: GnoemEvent = serde_json::from_str(&json).unwrap();
            assert_eq!(event.session_id, "s1");
        }
    }

    #[test]
    fn deserializes_event_without_data_field() {
        let json = r#"{
            "event": "session_end",
            "session_id": "abc123",
            "cwd": "/home/user/project",
            "timestamp": 1710734400000
        }"#;

        let event: GnoemEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.event, EventType::SessionEnd);
        assert!(event.data.is_null());
    }
}
```

- [ ] **Step 2: Create event module re-exports**

Create `src/event/mod.rs`:

```rust
pub mod schema;

pub use schema::{EventType, GnoemEvent};
```

- [ ] **Step 3: Wire module into main.rs**

Update `src/main.rs`:

```rust
mod event;

fn main() {
    println!("Gnoem - Claude Code Session Visualizer");
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test event::schema`
Expected: 3 tests pass

- [ ] **Step 5: Commit**

```bash
git add src/event/
git commit -m "feat: add event schema with deserialization and tests"
```

---

## Task 3: Event File Watcher

**Files:**
- Create: `src/event/watcher.rs`
- Create: `tests/integration/event_watcher_test.rs`

- [ ] **Step 1: Write integration test for event watcher**

Create `tests/integration/event_watcher_test.rs`:

```rust
use gnoem::event::{EventWatcher, GnoemEvent};
use std::fs;
use std::time::Duration;
use tempfile::TempDir;

#[test]
fn watcher_picks_up_new_event_files() {
    let dir = TempDir::new().unwrap();
    let mut watcher = EventWatcher::new(dir.path()).unwrap();

    // Write an event file
    let event_json = r#"{
        "event": "session_start",
        "session_id": "test1",
        "cwd": "/tmp/project",
        "timestamp": 1000
    }"#;
    let event_path = dir.path().join("1000-session-start-test1.json");
    fs::write(&event_path, event_json).unwrap();

    // Give watcher time to pick it up
    std::thread::sleep(Duration::from_millis(500));

    let events = watcher.poll_events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].session_id, "test1");

    // File should be deleted after processing
    assert!(!event_path.exists());
}

#[test]
fn watcher_ignores_non_json_files() {
    let dir = TempDir::new().unwrap();
    let mut watcher = EventWatcher::new(dir.path()).unwrap();

    fs::write(dir.path().join("notes.txt"), "not an event").unwrap();
    std::thread::sleep(Duration::from_millis(500));

    let events = watcher.poll_events();
    assert!(events.is_empty());
}

#[test]
fn watcher_handles_malformed_json_gracefully() {
    let dir = TempDir::new().unwrap();
    let mut watcher = EventWatcher::new(dir.path()).unwrap();

    let bad_path = dir.path().join("1000-bad-event.json");
    fs::write(&bad_path, "not valid json{{{").unwrap();
    std::thread::sleep(Duration::from_millis(500));

    let events = watcher.poll_events();
    assert!(events.is_empty());
    // Bad file should still be cleaned up
    assert!(!bad_path.exists());
}
```

- [ ] **Step 2: Run integration tests to verify they fail**

Run: `cargo test --test integration`
Expected: FAIL — `EventWatcher` not found

- [ ] **Step 3: Implement EventWatcher**

Create `src/event/watcher.rs`:

```rust
use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use super::GnoemEvent;

pub struct EventWatcher {
    events_dir: PathBuf,
    pending: VecDeque<GnoemEvent>,
    _watcher: RecommendedWatcher,
    rx: mpsc::Receiver<Result<Event, notify::Error>>,
}

impl EventWatcher {
    pub fn new(events_dir: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        fs::create_dir_all(events_dir)?;

        let (tx, rx) = mpsc::channel();
        let mut watcher = notify::recommended_watcher(move |res| {
            let _ = tx.send(res);
        })?;
        watcher.watch(events_dir, RecursiveMode::NonRecursive)?;

        let mut w = Self {
            events_dir: events_dir.to_path_buf(),
            pending: VecDeque::new(),
            _watcher: watcher,
            rx,
        };

        // Process any existing files on startup
        w.scan_existing_files();

        Ok(w)
    }

    pub fn poll_events(&mut self) -> Vec<GnoemEvent> {
        // Drain notifications from the file watcher
        while let Ok(Ok(event)) = self.rx.try_recv() {
            if matches!(event.kind, EventKind::Create(_)) {
                for path in event.paths {
                    self.try_read_event(&path);
                }
            }
        }

        self.pending.drain(..).collect()
    }

    fn scan_existing_files(&mut self) {
        let Ok(entries) = fs::read_dir(&self.events_dir) else {
            return;
        };

        let mut paths: Vec<PathBuf> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
            .collect();

        paths.sort();

        for path in paths {
            self.try_read_event(&path);
        }
    }

    fn try_read_event(&mut self, path: &Path) {
        if path.extension().is_none_or(|ext| ext != "json") {
            return;
        }

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return,
        };

        // Always delete the file after reading
        let _ = fs::remove_file(path);

        match serde_json::from_str::<GnoemEvent>(&content) {
            Ok(event) => self.pending.push_back(event),
            Err(_) => {} // Malformed JSON — already deleted
        }
    }
}
```

- [ ] **Step 4: Update event/mod.rs with new export**

```rust
pub mod schema;
pub mod watcher;

pub use schema::{EventType, GnoemEvent};
pub use watcher::EventWatcher;
```

- [ ] **Step 5: Make the crate a library too for integration tests**

Create `src/lib.rs`:

```rust
pub mod event;
```

- [ ] **Step 6: Set up integration test module**

Create `tests/integration/mod.rs`:

```rust
mod event_watcher_test;
```

Create `tests/integration.rs`:

```rust
mod integration;
```

- [ ] **Step 7: Run integration tests**

Run: `cargo test --test integration`
Expected: 3 tests pass

- [ ] **Step 8: Commit**

```bash
git add src/event/watcher.rs src/lib.rs tests/
git commit -m "feat: add event file watcher with notify"
```

---

## Task 4: Configuration

**Files:**
- Create: `src/config.rs`

- [ ] **Step 1: Write config tests**

Create `src/config.rs`:

```rust
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    pub display: DisplayConfig,
    pub animation: AnimationConfig,
    pub paths: PathsConfig,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct DisplayConfig {
    pub show_session_name: bool,
    pub show_duration: bool,
    pub show_last_activity: bool,
    pub show_project_branch: bool,
    pub show_status: bool,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct AnimationConfig {
    pub ui_fps: u32,
    pub idle_timeout_secs: u64,
    pub plant_wilt_after_secs: u64,
    pub paused: bool,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct PathsConfig {
    pub events_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            display: DisplayConfig::default(),
            animation: AnimationConfig::default(),
            paths: PathsConfig::default(),
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            show_session_name: true,
            show_duration: true,
            show_last_activity: true,
            show_project_branch: true,
            show_status: true,
        }
    }
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            ui_fps: 16,
            idle_timeout_secs: 30,
            plant_wilt_after_secs: 120,
            paused: false,
        }
    }
}

impl Default for PathsConfig {
    fn default() -> Self {
        let base = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("gnoem");
        Self {
            events_dir: base.join("events"),
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("gnoem")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn default_config_has_all_display_fields_enabled() {
        let config = Config::default();
        assert!(config.display.show_session_name);
        assert!(config.display.show_duration);
        assert!(config.display.show_last_activity);
        assert!(config.display.show_project_branch);
        assert!(config.display.show_status);
    }

    #[test]
    fn default_animation_config_values() {
        let config = Config::default();
        assert_eq!(config.animation.ui_fps, 16);
        assert_eq!(config.animation.idle_timeout_secs, 30);
        assert_eq!(config.animation.plant_wilt_after_secs, 120);
        assert!(!config.animation.paused);
    }

    #[test]
    fn loads_config_from_toml_file() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, r#"
[display]
show_duration = false

[animation]
ui_fps = 30
idle_timeout_secs = 60
"#).unwrap();

        let config = Config::load(file.path()).unwrap();
        assert!(!config.display.show_duration);
        assert!(config.display.show_session_name); // default preserved
        assert_eq!(config.animation.ui_fps, 30);
        assert_eq!(config.animation.idle_timeout_secs, 60);
    }

    #[test]
    fn returns_defaults_when_file_missing() {
        let config = Config::load(Path::new("/nonexistent/config.toml")).unwrap();
        assert_eq!(config.animation.ui_fps, 16);
    }
}
```

- [ ] **Step 2: Wire into lib.rs and main.rs**

Update `src/lib.rs`:

```rust
pub mod config;
pub mod event;
```

Update `src/main.rs`:

```rust
mod config;
mod event;

fn main() {
    println!("Gnoem - Claude Code Session Visualizer");
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test config`
Expected: 4 tests pass

- [ ] **Step 4: Commit**

```bash
git add src/config.rs src/lib.rs
git commit -m "feat: add config loading with TOML and sensible defaults"
```

---

## Task 5: Name Generator

**Files:**
- Create: `src/simulation/mod.rs`
- Create: `src/simulation/names.rs`

- [ ] **Step 1: Write name generator tests**

Create `src/simulation/names.rs`:

```rust
use rand::Rng;

const PREFIXES: &[&str] = &[
    "Grum", "Fiz", "Bram", "Twig", "Nob", "Wort", "Snib", "Drib",
    "Cob", "Flint", "Gar", "Hob", "Pip", "Quill", "Rook", "Stump",
    "Bark", "Moss", "Fern", "Reed", "Burr", "Peat", "Slag", "Wren",
];

const SUFFIXES: &[&str] = &[
    "bold", "wick", "ble", "knot", "sprout", "whistle", "thorn",
    "snap", "root", "spark", "stone", "leaf", "mire", "dust",
    "brook", "shade", "drift", "forge", "glen", "marsh", "vale",
];

pub struct NameGenerator;

impl NameGenerator {
    pub fn generate() -> String {
        let mut rng = rand::thread_rng();
        let prefix = PREFIXES[rng.gen_range(0..PREFIXES.len())];
        let suffix = SUFFIXES[rng.gen_range(0..SUFFIXES.len())];
        format!("{}{}", prefix, suffix)
    }

    pub fn generate_seeded(seed: u64) -> String {
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let prefix = PREFIXES[rng.gen_range(0..PREFIXES.len())];
        let suffix = SUFFIXES[rng.gen_range(0..SUFFIXES.len())];
        format!("{}{}", prefix, suffix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_non_empty_name() {
        let name = NameGenerator::generate();
        assert!(!name.is_empty());
    }

    #[test]
    fn name_starts_with_known_prefix() {
        let name = NameGenerator::generate();
        assert!(
            PREFIXES.iter().any(|p| name.starts_with(p)),
            "Name '{}' doesn't start with a known prefix",
            name
        );
    }

    #[test]
    fn name_ends_with_known_suffix() {
        let name = NameGenerator::generate();
        assert!(
            SUFFIXES.iter().any(|s| name.ends_with(s)),
            "Name '{}' doesn't end with a known suffix",
            name
        );
    }

    #[test]
    fn seeded_generation_is_deterministic() {
        let name1 = NameGenerator::generate_seeded(42);
        let name2 = NameGenerator::generate_seeded(42);
        assert_eq!(name1, name2);
    }

    #[test]
    fn different_seeds_produce_different_names() {
        let name1 = NameGenerator::generate_seeded(1);
        let name2 = NameGenerator::generate_seeded(999);
        // Statistically almost certain to differ
        assert_ne!(name1, name2);
    }
}
```

- [ ] **Step 2: Create simulation module**

Create `src/simulation/mod.rs`:

```rust
pub mod names;

pub use names::NameGenerator;
```

- [ ] **Step 3: Wire into lib.rs and main.rs**

Add `pub mod simulation;` to `src/lib.rs`.
Add `mod simulation;` to `src/main.rs`.

- [ ] **Step 4: Run tests**

Run: `cargo test simulation::names`
Expected: 5 tests pass

- [ ] **Step 5: Commit**

```bash
git add src/simulation/
git commit -m "feat: add gnome name generator with prefix+suffix combiner"
```

---

## Task 6: Gnoem Persistence (gnoems.toml)

**Files:**
- Create: `src/simulation/persistence.rs`

- [ ] **Step 1: Write persistence tests**

Create `src/simulation/persistence.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GnoemIdentity {
    pub name: String,
    pub color: String,
}

#[derive(Debug, Default)]
pub struct GnoemRegistry {
    path: PathBuf,
    identities: HashMap<String, GnoemIdentity>,
}

const COLORS: &[&str] = &[
    "cyan", "magenta", "yellow", "green", "red", "blue",
    "light_cyan", "light_magenta", "light_yellow", "light_green",
    "light_red", "light_blue",
];

impl GnoemRegistry {
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let identities = if path.exists() {
            let content = std::fs::read_to_string(path)?;
            toml::from_str(&content)?
        } else {
            HashMap::new()
        };

        Ok(Self {
            path: path.to_path_buf(),
            identities,
        })
    }

    pub fn get_or_create(&mut self, cwd: &str) -> &GnoemIdentity {
        if !self.identities.contains_key(cwd) {
            let name = super::NameGenerator::generate();
            let color = self.next_unused_color();
            self.identities.insert(
                cwd.to_string(),
                GnoemIdentity { name, color },
            );
        }
        &self.identities[cwd]
    }

    pub fn get(&self, cwd: &str) -> Option<&GnoemIdentity> {
        self.identities.get(cwd)
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(&self.identities)?;
        std::fs::write(&self.path, content)?;
        Ok(())
    }

    fn next_unused_color(&self) -> String {
        let used: Vec<&str> = self.identities.values().map(|i| i.color.as_str()).collect();
        COLORS
            .iter()
            .find(|c| !used.contains(*c))
            .unwrap_or(&COLORS[0])
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn creates_new_identity_for_unknown_cwd() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("gnoems.toml");
        let mut registry = GnoemRegistry::load(&path).unwrap();

        let identity = registry.get_or_create("/home/user/project");
        assert!(!identity.name.is_empty());
        assert!(!identity.color.is_empty());
    }

    #[test]
    fn returns_same_identity_for_same_cwd() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("gnoems.toml");
        let mut registry = GnoemRegistry::load(&path).unwrap();

        let name1 = registry.get_or_create("/home/user/project").name.clone();
        let name2 = registry.get_or_create("/home/user/project").name.clone();
        assert_eq!(name1, name2);
    }

    #[test]
    fn assigns_different_colors_to_different_cwds() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("gnoems.toml");
        let mut registry = GnoemRegistry::load(&path).unwrap();

        let color1 = registry.get_or_create("/project/a").color.clone();
        let color2 = registry.get_or_create("/project/b").color.clone();
        assert_ne!(color1, color2);
    }

    #[test]
    fn save_and_reload_preserves_identities() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("gnoems.toml");

        {
            let mut registry = GnoemRegistry::load(&path).unwrap();
            registry.get_or_create("/home/user/project");
            registry.save().unwrap();
        }

        let registry = GnoemRegistry::load(&path).unwrap();
        let identity = registry.get("/home/user/project");
        assert!(identity.is_some());
    }

    #[test]
    fn returns_defaults_when_file_missing() {
        let registry = GnoemRegistry::load(Path::new("/nonexistent/gnoems.toml")).unwrap();
        assert!(registry.get("/any/path").is_none());
    }
}
```

- [ ] **Step 2: Add to simulation module**

Update `src/simulation/mod.rs`:

```rust
pub mod names;
pub mod persistence;

pub use names::NameGenerator;
pub use persistence::GnoemRegistry;
```

- [ ] **Step 3: Run tests**

Run: `cargo test simulation::persistence`
Expected: 5 tests pass

- [ ] **Step 4: Commit**

```bash
git add src/simulation/persistence.rs src/simulation/mod.rs
git commit -m "feat: add gnoem persistence registry with save/load"
```

---

## Task 7: Gnoem State Machine

**Files:**
- Create: `src/simulation/gnoem.rs`

- [ ] **Step 1: Write state machine tests**

Create `src/simulation/gnoem.rs`:

```rust
use std::time::Instant;

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
}

impl GnoemState {
    /// Duration in milliseconds before auto-transitioning to the next state
    pub fn auto_transition_ms(&self) -> Option<u64> {
        match self {
            GnoemState::Entering => Some(2000),
            GnoemState::SittingDown => Some(1000),
            GnoemState::Jumping => Some(1500),
            GnoemState::HeadScratching => Some(2000),
            GnoemState::ReceivingLetter => Some(1500),
            GnoemState::LeaningBack => Some(2000),
            GnoemState::ShufflingPapers => Some(2000),
            GnoemState::PackingUp => Some(2000),
            GnoemState::Leaving => Some(2000),
            GnoemState::Idle | GnoemState::Typing => None,
        }
    }

    /// The state to auto-transition to after the duration expires
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
            _ => None,
        }
    }

    /// Number of animation frames for this state
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

#[derive(Debug, Clone, PartialEq)]
pub struct MiniGnoem {
    pub agent_id: String,
    pub state: GnoemState,
}

pub struct Gnoem {
    pub name: String,
    pub color: String,
    pub state: GnoemState,
    pub session_id: String,
    pub cwd: String,
    pub animation_frame: usize,
    pub state_entered_at: Instant,
    pub subagents: Vec<MiniGnoem>,
}

impl Gnoem {
    pub fn new(name: String, color: String, session_id: String, cwd: String) -> Self {
        Self {
            name,
            color,
            state: GnoemState::Entering,
            session_id,
            cwd,
            animation_frame: 0,
            state_entered_at: Instant::now(),
            subagents: Vec::new(),
        }
    }

    pub fn transition_to(&mut self, new_state: GnoemState) {
        self.state = new_state;
        self.animation_frame = 0;
        self.state_entered_at = Instant::now();
    }

    pub fn advance_frame(&mut self) {
        self.animation_frame = (self.animation_frame + 1) % self.state.frame_count();
    }

    pub fn check_auto_transition(&mut self) -> bool {
        if let (Some(duration), Some(next)) = (
            self.state.auto_transition_ms(),
            self.state.auto_next(),
        ) {
            if self.state_entered_at.elapsed().as_millis() >= duration as u128 {
                self.transition_to(next);
                return true;
            }
        }
        false
    }

    pub fn is_leaving(&self) -> bool {
        self.state == GnoemState::Leaving
    }

    pub fn has_left(&self) -> bool {
        self.state == GnoemState::Leaving
            && self.state_entered_at.elapsed().as_millis()
                >= self.state.auto_transition_ms().unwrap_or(0) as u128
    }

    pub fn add_subagent(&mut self, agent_id: String) {
        self.subagents.push(MiniGnoem {
            agent_id,
            state: GnoemState::Typing,
        });
    }

    pub fn remove_subagent(&mut self, agent_id: &str) {
        self.subagents.retain(|s| s.agent_id != agent_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_gnoem_starts_in_entering_state() {
        let gnoem = Gnoem::new(
            "Grumbold".into(), "cyan".into(),
            "sess1".into(), "/project".into(),
        );
        assert_eq!(gnoem.state, GnoemState::Entering);
    }

    #[test]
    fn transition_resets_animation_frame() {
        let mut gnoem = Gnoem::new(
            "Grumbold".into(), "cyan".into(),
            "sess1".into(), "/project".into(),
        );
        gnoem.animation_frame = 3;
        gnoem.transition_to(GnoemState::Typing);
        assert_eq!(gnoem.animation_frame, 0);
        assert_eq!(gnoem.state, GnoemState::Typing);
    }

    #[test]
    fn advance_frame_wraps_around() {
        let mut gnoem = Gnoem::new(
            "Grumbold".into(), "cyan".into(),
            "sess1".into(), "/project".into(),
        );
        gnoem.transition_to(GnoemState::Idle); // 2 frames
        gnoem.advance_frame(); // 1
        gnoem.advance_frame(); // wraps to 0
        assert_eq!(gnoem.animation_frame, 0);
    }

    #[test]
    fn entering_auto_transitions_to_sitting_down() {
        assert_eq!(
            GnoemState::Entering.auto_next(),
            Some(GnoemState::SittingDown)
        );
    }

    #[test]
    fn sitting_down_auto_transitions_to_idle() {
        assert_eq!(
            GnoemState::SittingDown.auto_next(),
            Some(GnoemState::Idle)
        );
    }

    #[test]
    fn idle_does_not_auto_transition() {
        assert_eq!(GnoemState::Idle.auto_next(), None);
        assert_eq!(GnoemState::Idle.auto_transition_ms(), None);
    }

    #[test]
    fn packing_up_transitions_to_leaving() {
        assert_eq!(
            GnoemState::PackingUp.auto_next(),
            Some(GnoemState::Leaving)
        );
    }

    #[test]
    fn add_and_remove_subagent() {
        let mut gnoem = Gnoem::new(
            "Grumbold".into(), "cyan".into(),
            "sess1".into(), "/project".into(),
        );
        gnoem.add_subagent("agent1".into());
        assert_eq!(gnoem.subagents.len(), 1);

        gnoem.remove_subagent("agent1");
        assert!(gnoem.subagents.is_empty());
    }

    #[test]
    fn remove_nonexistent_subagent_is_noop() {
        let mut gnoem = Gnoem::new(
            "Grumbold".into(), "cyan".into(),
            "sess1".into(), "/project".into(),
        );
        gnoem.remove_subagent("nonexistent");
        assert!(gnoem.subagents.is_empty());
    }
}
```

- [ ] **Step 2: Add to simulation module**

Update `src/simulation/mod.rs`:

```rust
pub mod gnoem;
pub mod names;
pub mod persistence;

pub use gnoem::{Gnoem, GnoemState, MiniGnoem};
pub use names::NameGenerator;
pub use persistence::GnoemRegistry;
```

- [ ] **Step 3: Run tests**

Run: `cargo test simulation::gnoem`
Expected: 9 tests pass

- [ ] **Step 4: Commit**

```bash
git add src/simulation/gnoem.rs src/simulation/mod.rs
git commit -m "feat: add gnoem state machine with auto-transitions and subagents"
```

---

## Task 8: Desk Environment

**Files:**
- Create: `src/simulation/desk.rs`

- [ ] **Step 1: Write desk environment tests and implementation**

Create `src/simulation/desk.rs`:

```rust
#[derive(Debug, Clone)]
pub struct DeskEnvironment {
    pub coffee_level: f32,       // 0.0 (empty) to 1.0 (full)
    pub paper_stack: u32,        // number of papers
    pub plant_health: f32,       // 0.0 (wilted) to 1.0 (healthy)
    pub monitor_error: bool,     // showing error icon
    pub monitor_typing: bool,    // showing typing indicator
}

impl Default for DeskEnvironment {
    fn default() -> Self {
        Self {
            coffee_level: 0.5,
            paper_stack: 0,
            plant_health: 1.0,
            monitor_error: false,
            monitor_typing: false,
        }
    }
}

impl DeskEnvironment {
    pub fn on_tool_use(&mut self) {
        self.paper_stack = (self.paper_stack + 1).min(10);
        self.monitor_typing = true;
        self.monitor_error = false;
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

    pub fn on_idle_tick(&mut self, wilt_progress: f32) {
        self.plant_health = (1.0 - wilt_progress).max(0.0);
        self.monitor_typing = false;
    }

    fn recover_plant(&mut self) {
        self.plant_health = (self.plant_health + 0.1).min(1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_desk_has_healthy_plant_and_half_coffee() {
        let desk = DeskEnvironment::default();
        assert_eq!(desk.plant_health, 1.0);
        assert_eq!(desk.coffee_level, 0.5);
        assert_eq!(desk.paper_stack, 0);
        assert!(!desk.monitor_error);
    }

    #[test]
    fn tool_use_grows_paper_stack() {
        let mut desk = DeskEnvironment::default();
        desk.on_tool_use();
        assert_eq!(desk.paper_stack, 1);
        desk.on_tool_use();
        assert_eq!(desk.paper_stack, 2);
    }

    #[test]
    fn paper_stack_caps_at_10() {
        let mut desk = DeskEnvironment::default();
        for _ in 0..15 {
            desk.on_tool_use();
        }
        assert_eq!(desk.paper_stack, 10);
    }

    #[test]
    fn tool_failure_shows_error_on_monitor() {
        let mut desk = DeskEnvironment::default();
        desk.on_tool_failure();
        assert!(desk.monitor_error);
        assert!(!desk.monitor_typing);
    }

    #[test]
    fn tool_use_clears_error_and_shows_typing() {
        let mut desk = DeskEnvironment::default();
        desk.on_tool_failure();
        desk.on_tool_use();
        assert!(!desk.monitor_error);
        assert!(desk.monitor_typing);
    }

    #[test]
    fn agent_stop_fills_coffee() {
        let mut desk = DeskEnvironment::default();
        desk.coffee_level = 0.0;
        desk.on_agent_stop();
        assert_eq!(desk.coffee_level, 0.25);
    }

    #[test]
    fn coffee_caps_at_1() {
        let mut desk = DeskEnvironment::default();
        desk.coffee_level = 0.9;
        desk.on_agent_stop();
        assert_eq!(desk.coffee_level, 1.0);
    }

    #[test]
    fn compact_reduces_paper_stack() {
        let mut desk = DeskEnvironment::default();
        desk.paper_stack = 5;
        desk.on_compact();
        assert_eq!(desk.paper_stack, 2);
    }

    #[test]
    fn idle_tick_wilts_plant() {
        let mut desk = DeskEnvironment::default();
        desk.on_idle_tick(0.5);
        assert_eq!(desk.plant_health, 0.5);
    }

    #[test]
    fn tool_use_recovers_plant() {
        let mut desk = DeskEnvironment::default();
        desk.plant_health = 0.5;
        desk.on_tool_use();
        assert_eq!(desk.plant_health, 0.6);
    }
}
```

- [ ] **Step 2: Add to simulation module**

Add `pub mod desk;` and `pub use desk::DeskEnvironment;` to `src/simulation/mod.rs`.

- [ ] **Step 3: Run tests**

Run: `cargo test simulation::desk`
Expected: 10 tests pass

- [ ] **Step 4: Commit**

```bash
git add src/simulation/desk.rs src/simulation/mod.rs
git commit -m "feat: add desk environment with coffee, papers, plant, monitor"
```

---

## Task 9: Office Model (Event Processing)

**Files:**
- Create: `src/simulation/office.rs`

- [ ] **Step 1: Write office tests and implementation**

Create `src/simulation/office.rs`:

```rust
use std::collections::HashMap;
use std::time::Instant;

use crate::event::{EventType, GnoemEvent};
use super::{Gnoem, GnoemRegistry, GnoemState};
use super::desk::DeskEnvironment;

pub struct SessionInfo {
    pub name: String,
    pub status: String,
    pub started_at: Instant,
    pub last_activity: String,
    pub project_branch: String,
}

pub struct Office {
    pub gnoems: HashMap<String, Gnoem>,
    pub session_info: HashMap<String, SessionInfo>,
    pub registry: GnoemRegistry,
    idle_timeout_secs: u64,
    plant_wilt_after_secs: u64,
}

impl Office {
    pub fn new(registry: GnoemRegistry, idle_timeout_secs: u64, plant_wilt_after_secs: u64) -> Self {
        Self {
            gnoems: HashMap::new(),
            session_info: HashMap::new(),
            registry,
            idle_timeout_secs,
            plant_wilt_after_secs,
        }
    }

    pub fn process_event(&mut self, event: GnoemEvent) {
        match event.event {
            EventType::SessionStart => {
                let identity = self.registry.get_or_create(&event.cwd).clone();
                let mut gnoem = Gnoem::new(
                    identity.name.clone(),
                    identity.color.clone(),
                    event.session_id.clone(),
                    event.cwd.clone(),
                );
                gnoem.desk = DeskEnvironment::default();
                self.gnoems.insert(event.session_id.clone(), gnoem);
                self.session_info.insert(event.session_id, SessionInfo {
                    name: identity.name,
                    status: "entering".into(),
                    started_at: Instant::now(),
                    last_activity: "session started".into(),
                    project_branch: event.cwd,
                });
            }
            EventType::SessionEnd => {
                if let Some(gnoem) = self.gnoems.get_mut(&event.session_id) {
                    gnoem.transition_to(GnoemState::PackingUp);
                    if let Some(info) = self.session_info.get_mut(&event.session_id) {
                        info.status = "leaving".into();
                        info.last_activity = "session ended".into();
                    }
                }
            }
            EventType::PostToolUse => {
                if let Some(gnoem) = self.gnoems.get_mut(&event.session_id) {
                    gnoem.transition_to(GnoemState::Typing);
                    gnoem.desk.on_tool_use();
                    if let Some(info) = self.session_info.get_mut(&event.session_id) {
                        info.status = "typing".into();
                        info.last_activity = "used a tool".into();
                    }
                }
            }
            EventType::PostToolUseFailure => {
                if let Some(gnoem) = self.gnoems.get_mut(&event.session_id) {
                    gnoem.transition_to(GnoemState::HeadScratching);
                    gnoem.desk.on_tool_failure();
                    if let Some(info) = self.session_info.get_mut(&event.session_id) {
                        info.status = "confused".into();
                        info.last_activity = "tool failed".into();
                    }
                }
            }
            EventType::UserPromptSubmit => {
                if let Some(gnoem) = self.gnoems.get_mut(&event.session_id) {
                    gnoem.transition_to(GnoemState::ReceivingLetter);
                    if let Some(info) = self.session_info.get_mut(&event.session_id) {
                        info.status = "reading".into();
                        info.last_activity = "received prompt".into();
                    }
                }
            }
            EventType::Notification => {
                if let Some(gnoem) = self.gnoems.get_mut(&event.session_id) {
                    gnoem.transition_to(GnoemState::Jumping);
                    if let Some(info) = self.session_info.get_mut(&event.session_id) {
                        info.status = "notification!".into();
                    }
                }
            }
            EventType::SubagentStart => {
                if let Some(gnoem) = self.gnoems.get_mut(&event.session_id) {
                    let agent_id = event.data.get("agent_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();
                    gnoem.add_subagent(agent_id);
                }
            }
            EventType::SubagentStop => {
                if let Some(gnoem) = self.gnoems.get_mut(&event.session_id) {
                    let agent_id = event.data.get("agent_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    gnoem.remove_subagent(agent_id);
                }
            }
            EventType::PreCompact => {
                if let Some(gnoem) = self.gnoems.get_mut(&event.session_id) {
                    gnoem.transition_to(GnoemState::ShufflingPapers);
                    gnoem.desk.on_compact();
                    if let Some(info) = self.session_info.get_mut(&event.session_id) {
                        info.status = "tidying up".into();
                        info.last_activity = "compacting context".into();
                    }
                }
            }
            EventType::Stop => {
                if let Some(gnoem) = self.gnoems.get_mut(&event.session_id) {
                    gnoem.transition_to(GnoemState::LeaningBack);
                    gnoem.desk.on_agent_stop();
                    if let Some(info) = self.session_info.get_mut(&event.session_id) {
                        info.status = "done thinking".into();
                        info.last_activity = "finished response".into();
                    }
                }
            }
        }
    }

    pub fn tick_simulation(&mut self) {
        let mut to_remove = Vec::new();

        for (session_id, gnoem) in self.gnoems.iter_mut() {
            // Check auto-transitions (e.g., Entering → SittingDown)
            gnoem.check_auto_transition();

            // Check if gnoem has fully left
            if gnoem.has_left() {
                to_remove.push(session_id.clone());
            }
        }

        for session_id in to_remove {
            self.gnoems.remove(&session_id);
            self.session_info.remove(&session_id);
        }
    }

    pub fn active_gnoem_count(&self) -> usize {
        self.gnoems.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::persistence::GnoemRegistry;
    use std::path::Path;

    fn test_office() -> Office {
        let registry = GnoemRegistry::load(Path::new("/tmp/test-gnoems.toml")).unwrap();
        Office::new(registry, 30, 120)
    }

    fn session_start_event(session_id: &str, cwd: &str) -> GnoemEvent {
        GnoemEvent {
            event: EventType::SessionStart,
            session_id: session_id.into(),
            cwd: cwd.into(),
            timestamp: 0,
            data: serde_json::Value::Null,
        }
    }

    #[test]
    fn session_start_creates_gnoem() {
        let mut office = test_office();
        office.process_event(session_start_event("s1", "/project"));
        assert_eq!(office.active_gnoem_count(), 1);
        assert_eq!(office.gnoems["s1"].state, GnoemState::Entering);
    }

    #[test]
    fn session_end_triggers_packing_up() {
        let mut office = test_office();
        office.process_event(session_start_event("s1", "/project"));
        office.process_event(GnoemEvent {
            event: EventType::SessionEnd,
            session_id: "s1".into(),
            cwd: "/project".into(),
            timestamp: 1,
            data: serde_json::Value::Null,
        });
        assert_eq!(office.gnoems["s1"].state, GnoemState::PackingUp);
    }

    #[test]
    fn post_tool_use_triggers_typing() {
        let mut office = test_office();
        office.process_event(session_start_event("s1", "/project"));
        office.gnoems.get_mut("s1").unwrap().transition_to(GnoemState::Idle);

        office.process_event(GnoemEvent {
            event: EventType::PostToolUse,
            session_id: "s1".into(),
            cwd: "/project".into(),
            timestamp: 1,
            data: serde_json::Value::Null,
        });
        assert_eq!(office.gnoems["s1"].state, GnoemState::Typing);
        assert_eq!(office.gnoems["s1"].desk.paper_stack, 1);
    }

    #[test]
    fn tool_failure_triggers_head_scratching() {
        let mut office = test_office();
        office.process_event(session_start_event("s1", "/project"));

        office.process_event(GnoemEvent {
            event: EventType::PostToolUseFailure,
            session_id: "s1".into(),
            cwd: "/project".into(),
            timestamp: 1,
            data: serde_json::Value::Null,
        });
        assert_eq!(office.gnoems["s1"].state, GnoemState::HeadScratching);
        assert!(office.gnoems["s1"].desk.monitor_error);
    }

    #[test]
    fn notification_triggers_jumping() {
        let mut office = test_office();
        office.process_event(session_start_event("s1", "/project"));

        office.process_event(GnoemEvent {
            event: EventType::Notification,
            session_id: "s1".into(),
            cwd: "/project".into(),
            timestamp: 1,
            data: serde_json::Value::Null,
        });
        assert_eq!(office.gnoems["s1"].state, GnoemState::Jumping);
    }

    #[test]
    fn subagent_start_adds_mini_gnoem() {
        let mut office = test_office();
        office.process_event(session_start_event("s1", "/project"));

        office.process_event(GnoemEvent {
            event: EventType::SubagentStart,
            session_id: "s1".into(),
            cwd: "/project".into(),
            timestamp: 1,
            data: serde_json::json!({"agent_id": "a1"}),
        });
        assert_eq!(office.gnoems["s1"].subagents.len(), 1);
    }

    #[test]
    fn subagent_stop_removes_mini_gnoem() {
        let mut office = test_office();
        office.process_event(session_start_event("s1", "/project"));
        office.process_event(GnoemEvent {
            event: EventType::SubagentStart,
            session_id: "s1".into(),
            cwd: "/project".into(),
            timestamp: 1,
            data: serde_json::json!({"agent_id": "a1"}),
        });
        office.process_event(GnoemEvent {
            event: EventType::SubagentStop,
            session_id: "s1".into(),
            cwd: "/project".into(),
            timestamp: 2,
            data: serde_json::json!({"agent_id": "a1"}),
        });
        assert!(office.gnoems["s1"].subagents.is_empty());
    }

    #[test]
    fn unknown_session_events_are_ignored() {
        let mut office = test_office();
        office.process_event(GnoemEvent {
            event: EventType::PostToolUse,
            session_id: "nonexistent".into(),
            cwd: "/project".into(),
            timestamp: 1,
            data: serde_json::Value::Null,
        });
        assert_eq!(office.active_gnoem_count(), 0);
    }

    #[test]
    fn multiple_sessions_are_independent() {
        let mut office = test_office();
        office.process_event(session_start_event("s1", "/project-a"));
        office.process_event(session_start_event("s2", "/project-b"));
        assert_eq!(office.active_gnoem_count(), 2);
        assert_ne!(office.gnoems["s1"].name, office.gnoems["s2"].name);
    }
}
```

- [ ] **Step 2: Add `desk` field to Gnoem struct**

Update `src/simulation/gnoem.rs` — add `pub desk: DeskEnvironment` field to `Gnoem` struct and initialize it in `new()`:

```rust
use super::desk::DeskEnvironment;

// In Gnoem struct, add:
pub desk: DeskEnvironment,

// In Gnoem::new(), add:
desk: DeskEnvironment::default(),
```

- [ ] **Step 3: Add to simulation module**

Add `pub mod office;` and `pub use office::Office;` to `src/simulation/mod.rs`.

- [ ] **Step 4: Run tests**

Run: `cargo test simulation::office`
Expected: 9 tests pass

- [ ] **Step 5: Commit**

```bash
git add src/simulation/office.rs src/simulation/gnoem.rs src/simulation/mod.rs
git commit -m "feat: add office model with event processing for all event types"
```

---

## Task 10: BDD Feature Files and Cucumber Setup

**Files:**
- Create: `tests/cucumber.rs`
- Create: `tests/features/session_lifecycle.feature`
- Create: `tests/features/gnoem_reactions.feature`
- Create: `tests/features/subagents.feature`
- Create: `tests/features/desk_environment.feature`
- Create: `tests/features/persistence.feature`
- Create: `tests/steps/mod.rs`
- Create: `tests/steps/world.rs`
- Create: `tests/steps/session_steps.rs`
- Create: `tests/steps/reaction_steps.rs`
- Create: `tests/steps/subagent_steps.rs`
- Create: `tests/steps/desk_steps.rs`
- Create: `tests/steps/persistence_steps.rs`

This is a large task. Each feature file and its step definitions should be created, then verified with `cargo test --test cucumber`.

- [ ] **Step 1: Create cucumber test runner**

Create `tests/cucumber.rs`:

```rust
use cucumber::World;

mod steps;

use steps::world::GnoemWorld;

fn main() {
    futures::executor::block_on(GnoemWorld::run("tests/features"));
}
```

Add `futures = "0.3"` to `[dev-dependencies]` in `Cargo.toml`.

- [ ] **Step 2: Create World struct**

Create `tests/steps/world.rs`:

```rust
use cucumber::World;
use gnoem::simulation::{Office, GnoemRegistry};
use std::path::Path;

#[derive(Debug, World)]
#[world(init = Self::new)]
pub struct GnoemWorld {
    pub office: Office,
    pub last_gnoem_name: Option<String>,
    pub last_gnoem_color: Option<String>,
}

impl GnoemWorld {
    fn new() -> Self {
        let registry = GnoemRegistry::load(Path::new("/tmp/cucumber-gnoems.toml")).unwrap();
        Self {
            office: Office::new(registry, 30, 120),
            last_gnoem_name: None,
            last_gnoem_color: None,
        }
    }
}
```

Create `tests/steps/mod.rs`:

```rust
pub mod world;
pub mod session_steps;
pub mod reaction_steps;
pub mod subagent_steps;
pub mod desk_steps;
pub mod persistence_steps;
```

- [ ] **Step 3: Create session lifecycle feature and steps**

Create `tests/features/session_lifecycle.feature`:

```gherkin
Feature: Session lifecycle
  Gnoems enter and leave the office as Claude sessions start and stop.

  Scenario: A new session starts
    Given the office is empty
    When a session_start event arrives for "/home/user/project" with session "s1"
    Then a Gnoem should exist for session "s1"
    And the Gnoem for session "s1" should be in the "Entering" state

  Scenario: A session ends
    Given a Gnoem is working in session "s1" for "/home/user/project"
    When a session_end event arrives for session "s1"
    Then the Gnoem for session "s1" should be in the "PackingUp" state

  Scenario: Multiple sessions run simultaneously
    Given the office is empty
    When a session_start event arrives for "/home/user/project-a" with session "s1"
    And a session_start event arrives for "/home/user/project-b" with session "s2"
    Then there should be 2 Gnoems in the office
```

Create `tests/steps/session_steps.rs`:

```rust
use cucumber::{given, when, then};
use gnoem::event::{EventType, GnoemEvent};
use gnoem::simulation::GnoemState;
use super::world::GnoemWorld;

#[given("the office is empty")]
fn office_is_empty(world: &mut GnoemWorld) {
    assert_eq!(world.office.active_gnoem_count(), 0);
}

#[given(expr = "a Gnoem is working in session {string} for {string}")]
fn gnoem_is_working(world: &mut GnoemWorld, session_id: String, cwd: String) {
    world.office.process_event(GnoemEvent {
        event: EventType::SessionStart,
        session_id: session_id.clone(),
        cwd,
        timestamp: 0,
        data: serde_json::Value::Null,
    });
    // Fast-forward past Entering → SittingDown → Idle
    world.office.gnoems.get_mut(&session_id).unwrap()
        .transition_to(GnoemState::Idle);
}

#[when(expr = "a session_start event arrives for {string} with session {string}")]
fn session_start_arrives(world: &mut GnoemWorld, cwd: String, session_id: String) {
    world.office.process_event(GnoemEvent {
        event: EventType::SessionStart,
        session_id,
        cwd,
        timestamp: 0,
        data: serde_json::Value::Null,
    });
}

#[when(expr = "a session_end event arrives for session {string}")]
fn session_end_arrives(world: &mut GnoemWorld, session_id: String) {
    world.office.process_event(GnoemEvent {
        event: EventType::SessionEnd,
        session_id,
        cwd: String::new(),
        timestamp: 1,
        data: serde_json::Value::Null,
    });
}

#[then(expr = "a Gnoem should exist for session {string}")]
fn gnoem_exists(world: &mut GnoemWorld, session_id: String) {
    assert!(world.office.gnoems.contains_key(&session_id));
}

#[then(expr = "the Gnoem for session {string} should be in the {string} state")]
fn gnoem_in_state(world: &mut GnoemWorld, session_id: String, state: String) {
    let gnoem = &world.office.gnoems[&session_id];
    let expected = match state.as_str() {
        "Entering" => GnoemState::Entering,
        "Idle" => GnoemState::Idle,
        "Typing" => GnoemState::Typing,
        "Jumping" => GnoemState::Jumping,
        "HeadScratching" => GnoemState::HeadScratching,
        "ReceivingLetter" => GnoemState::ReceivingLetter,
        "LeaningBack" => GnoemState::LeaningBack,
        "ShufflingPapers" => GnoemState::ShufflingPapers,
        "PackingUp" => GnoemState::PackingUp,
        "Leaving" => GnoemState::Leaving,
        _ => panic!("Unknown state: {}", state),
    };
    assert_eq!(gnoem.state, expected);
}

#[then(expr = "there should be {int} Gnoems in the office")]
fn gnoem_count(world: &mut GnoemWorld, count: usize) {
    assert_eq!(world.office.active_gnoem_count(), count);
}
```

- [ ] **Step 4: Create remaining feature files**

Create `tests/features/gnoem_reactions.feature`:

```gherkin
Feature: Gnoem reactions to events
  Gnoems react visually to different Claude Code events.

  Scenario: Gnoem types when a tool is used
    Given a Gnoem is working in session "s1" for "/project"
    When a post_tool_use event arrives for session "s1"
    Then the Gnoem for session "s1" should be in the "Typing" state

  Scenario: Gnoem scratches head on tool failure
    Given a Gnoem is working in session "s1" for "/project"
    When a post_tool_use_failure event arrives for session "s1"
    Then the Gnoem for session "s1" should be in the "HeadScratching" state

  Scenario: Gnoem jumps on notification
    Given a Gnoem is working in session "s1" for "/project"
    When a notification event arrives for session "s1"
    Then the Gnoem for session "s1" should be in the "Jumping" state

  Scenario: Gnoem receives letter on user prompt
    Given a Gnoem is working in session "s1" for "/project"
    When a user_prompt_submit event arrives for session "s1"
    Then the Gnoem for session "s1" should be in the "ReceivingLetter" state

  Scenario: Gnoem leans back when agent stops
    Given a Gnoem is working in session "s1" for "/project"
    When a stop event arrives for session "s1"
    Then the Gnoem for session "s1" should be in the "LeaningBack" state

  Scenario: Gnoem shuffles papers on context compaction
    Given a Gnoem is working in session "s1" for "/project"
    When a pre_compact event arrives for session "s1"
    Then the Gnoem for session "s1" should be in the "ShufflingPapers" state
```

Create `tests/steps/reaction_steps.rs`:

```rust
use cucumber::when;
use gnoem::event::{EventType, GnoemEvent};
use super::world::GnoemWorld;

fn send_event(world: &mut GnoemWorld, event_type: EventType, session_id: &str) {
    world.office.process_event(GnoemEvent {
        event: event_type,
        session_id: session_id.into(),
        cwd: String::new(),
        timestamp: 1,
        data: serde_json::Value::Null,
    });
}

#[when(expr = "a post_tool_use event arrives for session {string}")]
fn post_tool_use(world: &mut GnoemWorld, session_id: String) {
    send_event(world, EventType::PostToolUse, &session_id);
}

#[when(expr = "a post_tool_use_failure event arrives for session {string}")]
fn post_tool_use_failure(world: &mut GnoemWorld, session_id: String) {
    send_event(world, EventType::PostToolUseFailure, &session_id);
}

#[when(expr = "a notification event arrives for session {string}")]
fn notification(world: &mut GnoemWorld, session_id: String) {
    send_event(world, EventType::Notification, &session_id);
}

#[when(expr = "a user_prompt_submit event arrives for session {string}")]
fn user_prompt_submit(world: &mut GnoemWorld, session_id: String) {
    send_event(world, EventType::UserPromptSubmit, &session_id);
}

#[when(expr = "a stop event arrives for session {string}")]
fn stop(world: &mut GnoemWorld, session_id: String) {
    send_event(world, EventType::Stop, &session_id);
}

#[when(expr = "a pre_compact event arrives for session {string}")]
fn pre_compact(world: &mut GnoemWorld, session_id: String) {
    send_event(world, EventType::PreCompact, &session_id);
}
```

Create `tests/features/subagents.feature`:

```gherkin
Feature: Subagent management
  Gnoems can have mini-Gnoems representing active subagents.

  Scenario: Subagent starts
    Given a Gnoem is working in session "s1" for "/project"
    When a subagent_start event with agent "a1" arrives for session "s1"
    Then the Gnoem for session "s1" should have 1 subagent

  Scenario: Subagent stops
    Given a Gnoem is working in session "s1" for "/project"
    And a subagent_start event with agent "a1" arrives for session "s1"
    When a subagent_stop event with agent "a1" arrives for session "s1"
    Then the Gnoem for session "s1" should have 0 subagents
```

Create `tests/steps/subagent_steps.rs`:

```rust
use cucumber::{given, when, then};
use gnoem::event::{EventType, GnoemEvent};
use super::world::GnoemWorld;

#[given(expr = "a subagent_start event with agent {string} arrives for session {string}")]
#[when(expr = "a subagent_start event with agent {string} arrives for session {string}")]
fn subagent_start(world: &mut GnoemWorld, agent_id: String, session_id: String) {
    world.office.process_event(GnoemEvent {
        event: EventType::SubagentStart,
        session_id,
        cwd: String::new(),
        timestamp: 1,
        data: serde_json::json!({"agent_id": agent_id}),
    });
}

#[when(expr = "a subagent_stop event with agent {string} arrives for session {string}")]
fn subagent_stop(world: &mut GnoemWorld, agent_id: String, session_id: String) {
    world.office.process_event(GnoemEvent {
        event: EventType::SubagentStop,
        session_id,
        cwd: String::new(),
        timestamp: 2,
        data: serde_json::json!({"agent_id": agent_id}),
    });
}

#[then(expr = "the Gnoem for session {string} should have {int} subagent(s)")]
fn subagent_count(world: &mut GnoemWorld, session_id: String, count: usize) {
    assert_eq!(world.office.gnoems[&session_id].subagents.len(), count);
}
```

Create `tests/features/desk_environment.feature`:

```gherkin
Feature: Desk environment
  The desk decorations react to session activity.

  Scenario: Paper stack grows when tools are used
    Given a Gnoem is working in session "s1" for "/project"
    When a post_tool_use event arrives for session "s1"
    And a post_tool_use event arrives for session "s1"
    Then the desk for session "s1" should have 2 papers

  Scenario: Monitor shows error on tool failure
    Given a Gnoem is working in session "s1" for "/project"
    When a post_tool_use_failure event arrives for session "s1"
    Then the desk for session "s1" should show an error on the monitor

  Scenario: Coffee fills when agent finishes
    Given a Gnoem is working in session "s1" for "/project"
    When a stop event arrives for session "s1"
    Then the desk for session "s1" should have more coffee
```

Create `tests/steps/desk_steps.rs`:

```rust
use cucumber::then;
use super::world::GnoemWorld;

#[then(expr = "the desk for session {string} should have {int} papers")]
fn desk_paper_count(world: &mut GnoemWorld, session_id: String, count: u32) {
    assert_eq!(world.office.gnoems[&session_id].desk.paper_stack, count);
}

#[then(expr = "the desk for session {string} should show an error on the monitor")]
fn desk_monitor_error(world: &mut GnoemWorld, session_id: String) {
    assert!(world.office.gnoems[&session_id].desk.monitor_error);
}

#[then(expr = "the desk for session {string} should have more coffee")]
fn desk_more_coffee(world: &mut GnoemWorld, session_id: String) {
    assert!(world.office.gnoems[&session_id].desk.coffee_level > 0.5);
}
```

Create `tests/features/persistence.feature`:

```gherkin
Feature: Gnoem persistence
  Gnoems are remembered across sessions by working directory.

  Scenario: Same project gets the same Gnoem back
    Given a Gnoem was previously created for "/home/user/project"
    When a session_start event arrives for "/home/user/project" with session "s2"
    Then the Gnoem for session "s2" should have the remembered name
    And the Gnoem for session "s2" should have the remembered color
```

Create `tests/steps/persistence_steps.rs`:

```rust
use cucumber::{given, then};
use super::world::GnoemWorld;

#[given(expr = "a Gnoem was previously created for {string}")]
fn gnoem_previously_created(world: &mut GnoemWorld, cwd: String) {
    let identity = world.office.registry.get_or_create(&cwd).clone();
    world.last_gnoem_name = Some(identity.name);
    world.last_gnoem_color = Some(identity.color);
}

#[then(expr = "the Gnoem for session {string} should have the remembered name")]
fn gnoem_has_remembered_name(world: &mut GnoemWorld, session_id: String) {
    let expected = world.last_gnoem_name.as_ref().unwrap();
    assert_eq!(&world.office.gnoems[&session_id].name, expected);
}

#[then(expr = "the Gnoem for session {string} should have the remembered color")]
fn gnoem_has_remembered_color(world: &mut GnoemWorld, session_id: String) {
    let expected = world.last_gnoem_color.as_ref().unwrap();
    assert_eq!(&world.office.gnoems[&session_id].color, expected);
}
```

- [ ] **Step 5: Run all cucumber tests**

Run: `cargo test --test cucumber`
Expected: All scenarios pass

- [ ] **Step 6: Commit**

```bash
git add tests/ Cargo.toml
git commit -m "feat: add BDD cucumber tests for all Gnoem behaviors"
```

---

## Task 11: Renderer Trait and Layout Calculator

**Files:**
- Create: `src/rendering/mod.rs`
- Create: `src/rendering/layout.rs`

- [ ] **Step 1: Write layout calculator tests and Renderer trait**

Create `src/rendering/mod.rs`:

```rust
pub mod layout;

use crate::simulation::Office;

pub enum AppAction {
    Quit,
    TogglePause,
}

pub trait Renderer {
    fn render_office(&mut self, office: &Office);
    fn handle_input(&mut self) -> Option<AppAction>;
    fn should_quit(&self) -> bool;
}
```

Create `src/rendering/layout.rs`:

```rust
pub struct CubicleSlot {
    pub col: u16,
    pub row: u16,
    pub width: u16,
    pub height: u16,
}

pub struct GridLayout {
    pub slots: Vec<CubicleSlot>,
    pub cols: u16,
    pub rows: u16,
    pub total_capacity: usize,
    pub status_bar_height: u16,
}

const CUBICLE_WIDTH: u16 = 32;
const CUBICLE_HEIGHT: u16 = 16;
const STATUS_BAR_HEIGHT: u16 = 1;

impl GridLayout {
    pub fn calculate(terminal_width: u16, terminal_height: u16) -> Self {
        let usable_height = terminal_height.saturating_sub(STATUS_BAR_HEIGHT);
        let cols = (terminal_width / CUBICLE_WIDTH).max(1);
        let rows = (usable_height / CUBICLE_HEIGHT).max(1);
        let total_capacity = (cols * rows) as usize;

        let mut slots = Vec::with_capacity(total_capacity);
        for row in 0..rows {
            for col in 0..cols {
                slots.push(CubicleSlot {
                    col: col * CUBICLE_WIDTH,
                    row: row * CUBICLE_HEIGHT,
                    width: CUBICLE_WIDTH,
                    height: CUBICLE_HEIGHT,
                });
            }
        }

        Self {
            slots,
            cols,
            rows,
            total_capacity,
            status_bar_height: STATUS_BAR_HEIGHT,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_terminal_fits_multiple_cubicles() {
        let layout = GridLayout::calculate(120, 40);
        assert_eq!(layout.cols, 3);  // 120 / 32 = 3
        assert_eq!(layout.rows, 2);  // (40-1) / 16 = 2
        assert_eq!(layout.total_capacity, 6);
        assert_eq!(layout.slots.len(), 6);
    }

    #[test]
    fn small_terminal_fits_at_least_one() {
        let layout = GridLayout::calculate(30, 15);
        assert!(layout.total_capacity >= 1);
    }

    #[test]
    fn very_small_terminal_still_gets_one_slot() {
        let layout = GridLayout::calculate(10, 5);
        assert_eq!(layout.total_capacity, 1);
    }

    #[test]
    fn slots_have_correct_positions() {
        let layout = GridLayout::calculate(64, 33);
        // 64/32 = 2 cols, (33-1)/16 = 2 rows
        assert_eq!(layout.slots[0].col, 0);
        assert_eq!(layout.slots[0].row, 0);
        assert_eq!(layout.slots[1].col, 32);
        assert_eq!(layout.slots[1].row, 0);
        assert_eq!(layout.slots[2].col, 0);
        assert_eq!(layout.slots[2].row, 16);
        assert_eq!(layout.slots[3].col, 32);
        assert_eq!(layout.slots[3].row, 16);
    }

    #[test]
    fn status_bar_height_is_reserved() {
        let layout = GridLayout::calculate(120, 40);
        assert_eq!(layout.status_bar_height, 1);
    }
}
```

- [ ] **Step 2: Wire into lib.rs and main.rs**

Add `pub mod rendering;` to both `src/lib.rs` and `src/main.rs`.

- [ ] **Step 3: Run tests**

Run: `cargo test rendering::layout`
Expected: 5 tests pass

- [ ] **Step 4: Commit**

```bash
git add src/rendering/
git commit -m "feat: add renderer trait and grid layout calculator"
```

---

## Task 12: ASCII Sprite Frames

**Files:**
- Create: `src/rendering/sprites.rs`

- [ ] **Step 1: Create sprite frames for all states**

Create `src/rendering/sprites.rs` with ASCII art frames for each `GnoemState`. Each state has 2-4 frames stored as `&[&str]` arrays. Include the desk and monitor art as well.

This is primarily art content — the tests verify frame count matches `GnoemState::frame_count()` and that all frames have consistent height.

```rust
use crate::simulation::GnoemState;

pub struct SpriteFrames;

impl SpriteFrames {
    pub fn get(state: &GnoemState, frame: usize) -> &'static [&'static str] {
        let frames = match state {
            GnoemState::Idle => &IDLE_FRAMES,
            GnoemState::Typing => &TYPING_FRAMES,
            GnoemState::Jumping => &JUMPING_FRAMES,
            GnoemState::HeadScratching => &HEAD_SCRATCHING_FRAMES,
            GnoemState::Entering => &ENTERING_FRAMES,
            GnoemState::SittingDown => &SITTING_DOWN_FRAMES,
            GnoemState::Leaving => &LEAVING_FRAMES,
            GnoemState::PackingUp => &PACKING_UP_FRAMES,
            GnoemState::ReceivingLetter => &RECEIVING_LETTER_FRAMES,
            GnoemState::LeaningBack => &LEANING_BACK_FRAMES,
            GnoemState::ShufflingPapers => &SHUFFLING_PAPERS_FRAMES,
        };
        let idx = frame % frames.len();
        frames[idx]
    }
}

// Each frame is a slice of lines (top to bottom)
const IDLE_FRAMES: [&[&str]; 2] = [
    &[
        r"    /\    ",
        r"   /  \   ",
        r"  / @@ \  ",
        r"  |    |  ",
        r"  ( o.o)  ",
        r"   ####   ",
        r"   |  |   ",
        r"  /|  |\  ",
        r"   |  |   ",
        r"  _/  \_  ",
    ],
    &[
        r"    /\    ",
        r"   /  \   ",
        r"  / @@ \  ",
        r"  |    |  ",
        r"  ( -. )  ",  // blink
        r"   ####   ",
        r"   |  |   ",
        r"  /|  |\  ",
        r"   |  |   ",
        r"  _/  \_  ",
    ],
];

const TYPING_FRAMES: [&[&str]; 4] = [
    &[
        r"    /\    ",
        r"   /  \   ",
        r"  / @@ \  ",
        r"  |    |  ",
        r"  ( o.o)  ",
        r"   ####   ",
        r"  \|  |   ",
        r"  /|  |\  ",
        r"   |  |   ",
        r"  _/  \_  ",
    ],
    &[
        r"    /\    ",
        r"   /  \   ",
        r"  / @@ \  ",
        r"  |    |  ",
        r"  ( o.o)  ",
        r"   ####   ",
        r"   |  |/  ",
        r"  /|  |\  ",
        r"   |  |   ",
        r"  _/  \_  ",
    ],
    &[
        r"    /\    ",
        r"   /  \   ",
        r"  / @@ \  ",
        r"  |    |  ",
        r"  ( o.o)  ",
        r"   ####   ",
        r"  \|  |   ",
        r"  /|  |\  ",
        r"   |  |   ",
        r"  _/  \_  ",
    ],
    &[
        r"    /\    ",
        r"   /  \   ",
        r"  / @@ \  ",
        r"  |    |  ",
        r"  ( o.o)  ",
        r"   ####   ",
        r"   |  |\  ",
        r"  /|  |\  ",
        r"   |  |   ",
        r"  _/  \_  ",
    ],
];

const JUMPING_FRAMES: [&[&str]; 3] = [
    &[
        r"    /\    ",
        r"   /  \   ",
        r"  / @@ \  ",
        r"  |    |  ",
        r"  ( ^^)   ",
        r"   ####   ",
        r"   |  |   ",
        r"  /|  |\  ",
        r"   \  /   ",
        r"    \/    ",
    ],
    &[
        r"          ",
        r"    /\    ",
        r"   /  \   ",
        r"  / @@ \  ",
        r"  |    |  ",
        r"  ( ^^)   ",
        r"   ####   ",
        r"   |  |   ",
        r"   |  |   ",
        r"  _/  \_  ",
    ],
    &[
        r"    /\    ",
        r"   /  \   ",
        r"  / @@ \  ",
        r"  |    |  ",
        r"  ( ^^)   ",
        r"   ####   ",
        r"   |  |   ",
        r"  /|  |\  ",
        r"   |  |   ",
        r"  _/  \_  ",
    ],
];

const HEAD_SCRATCHING_FRAMES: [&[&str]; 2] = [
    &[
        r"    /\  ? ",
        r"   /  \   ",
        r"  / @@ \  ",
        r"  |    |  ",
        r"  (?_? )  ",
        r"   ####   ",
        r"  /|  |   ",
        r"  /|  |\  ",
        r"   |  |   ",
        r"  _/  \_  ",
    ],
    &[
        r"    /\ ?  ",
        r"   /  \   ",
        r"  / @@ \  ",
        r"  |    |  ",
        r"  ( ?_?)  ",
        r"   ####   ",
        r"   |  |\  ",
        r"  /|  |\  ",
        r"   |  |   ",
        r"  _/  \_  ",
    ],
];

const ENTERING_FRAMES: [&[&str]; 4] = [
    &[
        r"          ",
        r"          ",
        r"          ",
        r"          ",
        r"          ",
        r"          ",
        r"          ",
        r"          ",
        r"      /\  ",
        r"     /  \ ",
    ],
    &[
        r"          ",
        r"          ",
        r"          ",
        r"          ",
        r"      /\  ",
        r"     /  \ ",
        r"    / @@ \",
        r"    ( o.o)",
        r"     #### ",
        r"     > >  ",
    ],
    &[
        r"          ",
        r"    /\    ",
        r"   /  \   ",
        r"  / @@ \  ",
        r"  |    |  ",
        r"  ( o.o)  ",
        r"   ####   ",
        r"   |  |   ",
        r"  /|  |\  ",
        r"    > >   ",
    ],
    &[
        r"    /\    ",
        r"   /  \   ",
        r"  / @@ \  ",
        r"  |    |  ",
        r"  ( o.o)  ",
        r"   ####   ",
        r"   |  |   ",
        r"  /|  |\  ",
        r"   |  |   ",
        r"  _/  \_  ",
    ],
];

const SITTING_DOWN_FRAMES: [&[&str]; 2] = [
    &[
        r"    /\    ",
        r"   /  \   ",
        r"  / @@ \  ",
        r"  |    |  ",
        r"  ( o.o)  ",
        r"   ####   ",
        r"   |  |   ",
        r"  /|  |\  ",
        r"   |  |   ",
        r"  _/  \_  ",
    ],
    &[
        r"    /\    ",
        r"   /  \   ",
        r"  / @@ \  ",
        r"  |    |  ",
        r"  ( o.o)  ",
        r"   ####   ",
        r"   |  |   ",
        r"  /|  |\  ",
        r"   |  |   ",
        r"  _/  \_  ",
    ],
];

const LEAVING_FRAMES: [&[&str]; 4] = ENTERING_FRAMES;  // reverse order handled by renderer

const PACKING_UP_FRAMES: [&[&str]; 2] = [
    &[
        r"    /\    ",
        r"   /  \   ",
        r"  / @@ \  ",
        r"  |    |  ",
        r"  ( -.- ) ",
        r"   ####   ",
        r"  \|  |/  ",
        r"  /|  |\  ",
        r"   |  |   ",
        r"  _/  \_  ",
    ],
    &[
        r"    /\    ",
        r"   /  \   ",
        r"  / @@ \  ",
        r"  |    |  ",
        r"  ( -.- ) ",
        r"   ####   ",
        r"   |  |   ",
        r"  /|  |\  ",
        r"   |  |   ",
        r"  _/  \_  ",
    ],
];

const RECEIVING_LETTER_FRAMES: [&[&str]; 2] = [
    &[
        r"    /\    ",
        r"   /  \   ",
        r"  / @@ \  ",
        r"  |    |  ",
        r"  ( o.o)  ",
        r"   ####   ",
        r"  \|  |  =",
        r"  /|  |\ =",
        r"   |  |   ",
        r"  _/  \_  ",
    ],
    &[
        r"    /\    ",
        r"   /  \   ",
        r"  / @@ \  ",
        r"  |    |  ",
        r"  ( ^.^)  ",
        r"   #### = ",
        r"  \|  |   ",
        r"  /|  |\  ",
        r"   |  |   ",
        r"  _/  \_  ",
    ],
];

const LEANING_BACK_FRAMES: [&[&str]; 2] = [
    &[
        r"   /\     ",
        r"  /  \    ",
        r" / @@ \   ",
        r" |    |   ",
        r" ( -u-)   ",
        r"  ####    ",
        r"  |  |    ",
        r" /|  |\   ",
        r"  |  |__  ",
        r" _/ _/  | ",
    ],
    &[
        r"   /\     ",
        r"  /  \    ",
        r" / @@ \   ",
        r" |    |   ",
        r" ( -u-)   ",
        r"  ####    ",
        r"  |  |    ",
        r" /|  |\   ",
        r"  | _|__  ",
        r" _/   /|  ",
    ],
];

const SHUFFLING_PAPERS_FRAMES: [&[&str]; 2] = [
    &[
        r"    /\    ",
        r"   /  \   ",
        r"  / @@ \  ",
        r"  |    |  ",
        r"  ( o.o)  ",
        r"   ####   ",
        r"  \|  |/  ",
        r"  /|  |\  ",
        r"   |  |   ",
        r"  _/  \_  ",
    ],
    &[
        r"    /\    ",
        r"   /  \   ",
        r"  / @@ \  ",
        r"  |    |  ",
        r"  ( o.o)  ",
        r"   ####   ",
        r"  /|  |\  ",
        r"  /|  |\  ",
        r"   |  |   ",
        r"  _/  \_  ",
    ],
];

pub const SPRITE_HEIGHT: usize = 10;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_states_have_matching_frame_counts() {
        let states = vec![
            GnoemState::Idle, GnoemState::Typing, GnoemState::Jumping,
            GnoemState::HeadScratching, GnoemState::Entering,
            GnoemState::SittingDown, GnoemState::Leaving,
            GnoemState::PackingUp, GnoemState::ReceivingLetter,
            GnoemState::LeaningBack, GnoemState::ShufflingPapers,
        ];
        for state in states {
            let expected = state.frame_count();
            // Verify we can access all frames without panic
            for frame in 0..expected {
                let lines = SpriteFrames::get(&state, frame);
                assert_eq!(lines.len(), SPRITE_HEIGHT, "State {:?} frame {} wrong height", state, frame);
            }
        }
    }

    #[test]
    fn frame_index_wraps_around() {
        let lines = SpriteFrames::get(&GnoemState::Idle, 100);
        assert_eq!(lines.len(), SPRITE_HEIGHT);
    }
}
```

- [ ] **Step 2: Add to rendering module**

Update `src/rendering/mod.rs` to add `pub mod sprites;`.

- [ ] **Step 3: Run tests**

Run: `cargo test rendering::sprites`
Expected: 2 tests pass

- [ ] **Step 4: Commit**

```bash
git add src/rendering/sprites.rs src/rendering/mod.rs
git commit -m "feat: add ASCII sprite frames for all gnoem states"
```

---

## Task 13: Ratatui Renderer (Cubicle Widget + TUI)

**Files:**
- Create: `src/rendering/cubicle.rs`
- Create: `src/rendering/tui.rs`

- [ ] **Step 1: Implement cubicle widget**

Create `src/rendering/cubicle.rs` — a ratatui widget that renders a single cubicle with the Gnoem sprite, desk decorations, and session info. Uses `ratatui::widgets::Widget` trait.

The widget reads from `&Gnoem` and `&SessionInfo` to draw the cubicle border, Gnoem sprite, desk items (coffee, papers, plant, monitor), and session info text.

- [ ] **Step 2: Implement RatatuiRenderer**

Create `src/rendering/tui.rs` — implements `Renderer` trait. Sets up crossterm terminal, handles input events (q, Ctrl+C, p), renders the office grid using `GridLayout` and `CubicleWidget`, and draws the status bar showing active/hidden session counts.

- [ ] **Step 3: Wire into rendering module**

Update `src/rendering/mod.rs`:

```rust
pub mod cubicle;
pub mod layout;
pub mod sprites;
pub mod tui;
```

- [ ] **Step 4: Verify it compiles**

Run: `cargo build`
Expected: Compiles successfully

- [ ] **Step 5: Commit**

```bash
git add src/rendering/cubicle.rs src/rendering/tui.rs src/rendering/mod.rs
git commit -m "feat: add ratatui renderer with cubicle widget and status bar"
```

---

## Task 14: App Loop

**Files:**
- Create: `src/app.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Implement App struct**

Create `src/app.rs` — coordinates the main loop:

1. Load config
2. Load gnoem registry
3. Create Office
4. Create EventWatcher
5. Create RatatuiRenderer
6. Run tick loop:
   - Poll events from watcher → feed to office
   - Tick simulation (auto-transitions, idle detection)
   - Advance animation frames
   - Render office
   - Handle input
   - Sleep for frame duration

- [ ] **Step 2: Implement CLI with clap**

Update `src/main.rs` with clap subcommands:

```
gnoem              → App::run()
gnoem install-hooks → generate hooks + print config
gnoem uninstall     → remove ~/.config/gnoem/
```

- [ ] **Step 3: Verify it runs**

Run: `cargo run`
Expected: TUI launches showing empty office, quit with `q`

- [ ] **Step 4: Commit**

```bash
git add src/app.rs src/main.rs
git commit -m "feat: add app loop coordinating watcher, simulation, and renderer"
```

---

## Task 15: Hook Scripts and Install Command

**Files:**
- Create: `hooks/gnoem-session-start.sh` (and all other hook scripts)
- Modify: `src/main.rs` (install-hooks subcommand)

- [ ] **Step 1: Create all 10 hook scripts**

Each script reads JSON from stdin, extracts `session_id` and `cwd` with `jq`, and writes an atomic event file to `~/.config/gnoem/events/`.

- [ ] **Step 2: Implement install-hooks command**

The command:
1. Creates `~/.config/gnoem/hooks/` directory
2. Copies bundled hook scripts there (or writes them from embedded strings)
3. Prints the JSON config snippet for Claude's `settings.json`

- [ ] **Step 3: Implement uninstall command**

The command removes `~/.config/gnoem/` entirely after confirmation.

- [ ] **Step 4: Test install-hooks manually**

Run: `cargo run -- install-hooks`
Expected: Scripts created, config snippet printed

- [ ] **Step 5: Commit**

```bash
git add hooks/ src/main.rs
git commit -m "feat: add hook scripts and install-hooks CLI command"
```

---

## Task 16: End-to-End Integration Test

**Files:**
- Create: `tests/integration/end_to_end.rs`

- [ ] **Step 1: Write integration test**

Test the full pipeline without the TUI:
1. Create temp events directory
2. Create EventWatcher
3. Create Office
4. Write event files to temp dir
5. Poll watcher
6. Process events in office
7. Assert Gnoem states

- [ ] **Step 2: Run integration test**

Run: `cargo test --test integration`
Expected: All integration tests pass

- [ ] **Step 3: Commit**

```bash
git add tests/integration/
git commit -m "test: add end-to-end integration test for event pipeline"
```

---

## Task 17: Final Polish and README

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Update README**

Replace the old Bevy-based README with the new project description, installation instructions, usage, and a screenshot/demo section.

- [ ] **Step 2: Run full test suite**

Run: `cargo test`
Expected: All tests pass (unit, integration, cucumber)

- [ ] **Step 3: Run clippy**

Run: `cargo clippy -- -D warnings`
Expected: No warnings

- [ ] **Step 4: Commit**

```bash
git add README.md
git commit -m "docs: update README for new TUI-based gnoem"
```

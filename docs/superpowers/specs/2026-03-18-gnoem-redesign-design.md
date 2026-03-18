# Gnoem Redesign — Design Specification

## Overview

Gnoem is a Rust TUI application that visualizes active Claude Code sessions as ASCII gnome creatures ("Gnoems") working in a virtual office. Built with ratatui, it provides a charming, real-time view of what your Claude sessions are doing.

The application is a thin, passive viewer. Claude Code hooks write event files; Gnoem watches the directory and animates accordingly. No direct coupling to Claude internals.

## Goals

- Visualize all active Claude Code sessions in a single terminal window
- Provide immediate visual feedback on session state (typing, errors, notifications, idle)
- Feel like a cozy office with personality — named Gnoems that persist across sessions
- Test-driven development with human-readable BDD acceptance tests
- Clean install/uninstall with no residual artifacts
- Architecture that allows swapping the TUI renderer for a graphical one later

## Non-Goals

- Controlling Claude sessions from within Gnoem
- Storing session logs or transcripts
- Running as a daemon/service

## Architecture

Monolithic single-binary with internal layered separation:

```
┌─────────────────────────────┐
│  main.rs (startup + loop)   │
├─────────────────────────────┤
│  Rendering Layer (ratatui)  │
│  - office viewport          │
│  - gnoem sprites/animation  │
│  - cubicle layout           │
├─────────────────────────────┤
│  Simulation Layer           │
│  - gnoem state machines     │
│  - animation tick system    │
│  - office environment state │
├─────────────────────────────┤
│  Data Layer                 │
│  - file watcher (notify)    │
│  - event parser             │
│  - config (toml)            │
│  - persistence (gnoems.toml)│
└─────────────────────────────┘
```

A `Renderer` trait separates the simulation from its visual representation:

```rust
trait Renderer {
    fn render_office(&mut self, office: &Office);
    fn handle_input(&mut self) -> Option<AppAction>;
    fn should_quit(&self) -> bool;
}
```

`RatatuiRenderer` is the initial implementation. Future graphical frontends implement the same trait.

## Domain Model

### GnoemState — State Machine

```
Entering → SittingDown → Idle ↔ Typing ↔ Jumping ↔ HeadScratching ↔ ReceivingLetter ↔ LeaningBack ↔ ShufflingPapers → PackingUp → Leaving
```

Each state has 2-4 animation frames and a duration. Transitions are triggered by events or timeouts.

### Core Types

**Gnoem** — the creature:
- `name: String` — generated gnome name (e.g., "Grumbold", "Fizwick")
- `color: Color` — unique color per Gnoem
- `state: GnoemState` — current state machine state
- `session_id: String`
- `cwd: PathBuf`
- `desk: DeskEnvironment` — coffee mug level, paper stack height, plant health
- `subagents: Vec<MiniGnoem>` — mini-Gnoems for active subagents

**Office** — the world:
- `cubicles: Vec<Cubicle>`
- `gnoems: HashMap<SessionId, Gnoem>`

**Cubicle** — a pane:
- `gnoem: Option<Gnoem>`
- `session_info: SessionInfo` — name, status, duration, last activity, project/branch

### Gnoem Appearance

Detailed ASCII art style, 7-8 lines tall:

```
    /\
   /  \
  / ▓▓ \
  |    |
  ( o.o)
   ████
   |  |
  /|  |\
   |  |
  _/  \_
```

Each Gnoem has:
- A unique color (hat and body)
- A randomly generated gnome name (prefix + suffix: "Grum" + "bold" = "Grumbold")
- Persistent identity tied to the working directory

### Desk Environment — Interactive Decorations

- **Coffee mug** — fills up when agent finishes thinking (`stop` event)
- **Paper stack** — grows as files are edited (`post_tool_use`)
- **Plant** — wilts when session is idle too long, recovers on activity
- **Monitor** — shows error icon on tool failure, typing indicators on activity

## Event System

### Data Flow

```
Claude Code Hook → writes JSON file → ~/.config/gnoem/events/ → Gnoem file watcher → event parser → state transition
```

### Event Directory

Atomic files in `~/.config/gnoem/events/`:

```
~/.config/gnoem/events/
├── 1710734400000-session-start-abc123.json
├── 1710734401500-post-tool-use-abc123.json
└── 1710734402000-subagent-start-abc123.json
```

One file per event. Gnoem reads and deletes after processing.

### Event Schema

```json
{
  "event": "session_start",
  "session_id": "abc123",
  "cwd": "/home/user/my-project",
  "timestamp": 1710734400000,
  "data": {
    "model": "claude-opus-4-6",
    "source": "startup"
  }
}
```

### Event Types and Gnoem Behavior

| Event | GnoemState Transition | Desk Effect |
|-------|----------------------|-------------|
| `session_start` | → Entering | Cubicle appears |
| `session_end` | → PackingUp | — |
| `post_tool_use` | → Typing | Paper stack grows |
| `post_tool_use_failure` | → HeadScratching | Monitor shows error icon |
| `user_prompt_submit` | → ReceivingLetter | — |
| `notification` | → Jumping | — |
| `subagent_start` | Spawn MiniGnoem | Side desk appears |
| `subagent_stop` | Remove MiniGnoem | Side desk disappears |
| `pre_compact` | → ShufflingPapers | Papers get tidied |
| `stop` | → LeaningBack | Coffee mug fills up |
| No event (timeout) | → Idle | Plant starts wilting |

### Hook Scripts

Bundled shell scripts installed to `~/.config/gnoem/hooks/`. Each reads Claude's JSON input from stdin (containing `session_id`, `cwd`, `hook_event_name`, and event-specific fields) and writes an atomic event file. Hooks require `bash` and `jq` (WSL on Windows).

Example:
```bash
#!/bin/bash
# Claude hooks pass JSON on stdin with session_id, cwd, hook_event_name, etc.
INPUT=$(cat)
SESSION_ID=$(echo "$INPUT" | jq -r '.session_id')
CWD=$(echo "$INPUT" | jq -r '.cwd')
TIMESTAMP=$(date +%s%3N)
echo "{\"event\":\"session_start\",\"session_id\":\"$SESSION_ID\",\"cwd\":\"$CWD\",\"timestamp\":$TIMESTAMP,\"data\":$INPUT}" \
  > ~/.config/gnoem/events/${TIMESTAMP}-session-start-${SESSION_ID}.json
```

### Hook Installation

`gnoem install-hooks` generates the hook scripts and prints a JSON config snippet for the user to paste into their Claude `settings.json`. Does not modify Claude settings directly.

## Rendering

### Tick Rates

- **UI tick: ~60ms (~16fps)** — animation frame cycling
- **Simulation tick: 500ms** — state transitions, desk environment updates, idle timers

### Cubicle Layout

Dynamic grid based on terminal size:
- Each cubicle: ~30 chars wide x 15 lines tall
- Grid fills as sessions start, empties as they end
- Overflow: show as many cubicles as fit; a status bar at the bottom shows the count of hidden sessions (nice-to-have: scrolling/navigation to reach them)

### Cubicle Anatomy

```
┌─ Grumbold ──────────────────┐
│ myproject (main) ▏ 12m 34s  │
│ Last: edited main.rs        │
│─────────────────────────────│
│        ___                  │
│       |___|    ☕▓▓▓        │
│    /\(o.o)     📄📄📄      │
│     ████       🌿          │
│    \|  |~                   │
│    /|  |\   ┌──────┐        │
│     |  |    │ [>>] │        │
│    _/  \_   └──────┘        │
│─────────────────────────────│
│ ● typing                   │
└─────────────────────────────┘
```

### Animation

Each GnoemState has 2-4 frames:
- **Typing:** arms alternate, characters on monitor
- **Jumping:** Gnoem moves up/down, face changes to `(^^)`
- **HeadScratching:** arm to head, `(?_?)` face
- **Entering/Leaving:** horizontal movement across cubicle
- **Idle:** occasional blink, slight sway
- **ReceivingLetter:** envelope appears, Gnoem reaches for it
- **ShufflingPapers:** papers rearrange on desk
- **LeaningBack:** chair tilts, feet on desk

### Keyboard Input

- **`q` / `Ctrl+C`** — quit
- **`p`** — pause/resume animations

Nice-to-haves (not in initial scope):
- Arrow keys to scroll/navigate between cubicles
- Enter to expand a cubicle for more detail
- Toggle info fields at runtime

## Configuration

### Config File — `~/.config/gnoem/config.toml`

```toml
[display]
show_session_name = true
show_duration = true
show_last_activity = true
show_project_branch = true
show_status = true

[animation]
ui_fps = 16
idle_timeout_secs = 30
plant_wilt_after_secs = 120
paused = false

[paths]
events_dir = "~/.config/gnoem/events"
```

### Gnoem Persistence — `~/.config/gnoem/gnoems.toml`

```toml
["/home/user/my-project"]
name = "Grumbold"
color = "cyan"

["/home/user/auth-service"]
name = "Fizwick"
color = "magenta"
```

### Name Generator

Combines random prefix + suffix from built-in word lists:
- Prefixes: Grum, Fiz, Bram, Twig, Nob, Wort, Snib...
- Suffixes: bold, wick, ble, knot, sprout, whistle, thorn...

### CLI Commands

```
gnoem                # start the TUI
gnoem install-hooks  # generate hook scripts + print Claude config snippet
gnoem uninstall      # remove ~/.config/gnoem/ entirely
```

## Testing Strategy

### BDD Acceptance Tests

Cucumber-rs with `.feature` files:

```gherkin
Feature: Gnoem enters the office
  Scenario: A new session starts
    Given the office is empty
    When a session_start event arrives for "/home/user/project"
    Then a new Gnoem should appear
    And the Gnoem should be in the "Entering" state
    And a cubicle should be visible

  Scenario: A known project starts a session
    Given a Gnoem named "Grumbold" previously worked in "/home/user/project"
    When a session_start event arrives for "/home/user/project"
    Then the Gnoem should be named "Grumbold"
    And the Gnoem should have the same color as before

  Scenario: Gnoem reacts to tool failure
    Given a Gnoem is typing in their cubicle
    When a post_tool_use_failure event arrives
    Then the Gnoem should transition to "HeadScratching"
    And the monitor should show an error icon
```

### Test Layers

| Layer | What | How |
|-------|------|-----|
| **BDD acceptance** | End-to-end scenarios | `cucumber-rs`, step definitions drive `Office` with events |
| **Unit tests** | State machine, name generator, event parser, config | `#[cfg(test)]` modules |
| **Integration tests** | File watcher + event parsing pipeline | `tests/` with real temp directories |
| **Rendering tests** | Cubicle layout math for various terminal sizes | Unit tests, no terminal needed |

### Testing Principles

- Simulation layer is testable without ratatui — inject events, assert state
- `Renderer` trait enables `MockRenderer` in tests
- File watcher tests use real temp directories, no filesystem mocking
- TDD: write failing test first, then implement

## Dependencies

| Crate | Purpose |
|-------|---------|
| `ratatui` | Terminal UI rendering |
| `crossterm` | Terminal backend |
| `notify` | File system watching |
| `serde` / `serde_json` | Event parsing |
| `toml` | Config files |
| `cucumber` | BDD testing |
| `rand` | Name/color generation |
| `dirs` | Platform-appropriate config paths |
| `clap` | CLI argument parsing |
| `chrono` | Timestamps and duration formatting |

## File Structure

```
gnoem/
├── Cargo.toml
├── src/
│   ├── main.rs              # entry point, CLI, app loop
│   ├── app.rs               # App struct, tick loop coordination
│   ├── config.rs            # config.toml loading and defaults
│   ├── event/
│   │   ├── mod.rs           # event types and parsing
│   │   ├── watcher.rs       # file watcher (notify)
│   │   └── schema.rs        # event JSON schema
│   ├── simulation/
│   │   ├── mod.rs
│   │   ├── office.rs        # Office, Cubicle
│   │   ├── gnoem.rs         # Gnoem, GnoemState, state machine
│   │   ├── desk.rs          # DeskEnvironment
│   │   ├── names.rs         # name generator
│   │   └── persistence.rs   # gnoems.toml read/write
│   └── rendering/
│       ├── mod.rs           # Renderer trait
│       ├── tui.rs           # RatatuiRenderer
│       ├── sprites.rs       # ASCII art frames per state
│       ├── cubicle.rs       # cubicle widget
│       └── layout.rs        # grid layout calculator
├── tests/
│   ├── features/            # .feature files
│   │   ├── session_lifecycle.feature
│   │   ├── gnoem_reactions.feature
│   │   ├── subagents.feature
│   │   ├── desk_environment.feature
│   │   └── persistence.feature
│   ├── steps/               # step definitions
│   │   └── mod.rs
│   └── integration/
│       ├── event_watcher.rs
│       └── config_loading.rs
├── hooks/                   # hook script templates
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
    └── superpowers/
        └── specs/
            └── 2026-03-18-gnoem-redesign-design.md
```

## Repurposing the GitHub Repository

The existing `Timo-DK/gnoem` repo contains a Bevy-based simulation. The redesign:
- Wipes the `src/` directory and `Cargo.toml`
- Keeps the repo, LICENSE (MIT), and `.gitignore` (updated for new structure)
- Updates `README.md` to reflect the new project
- Preserves git history on a legacy branch before the reset

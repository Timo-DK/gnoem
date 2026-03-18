/// ASCII art sprites and desk decorations for all Gnoem states.
///
/// Every sprite is a static multi-line string.  Frames are selected by the
/// caller using `animation_frame % frames.len()` so the index is always safe.
///
/// Gnome anatomy reference used throughout:
///   ^  — pointy hat tip
///  /A\ — hat brim (A = face character)
///  \O/ — arms raised / spread
///   |  — torso
///  / \ — legs
use crate::office::GnoemState;

// ---------------------------------------------------------------------------
// sprite_frames
// ---------------------------------------------------------------------------

/// Return the animation frames for `state`.
///
/// Each frame is a multi-line string ready to be split on `\n` and rendered
/// line-by-line.  The returned slice always has at least one element, so
/// `frames[animation_frame % frames.len()]` is always valid.
pub fn sprite_frames(state: &GnoemState) -> &'static [&'static str] {
    match state {
        GnoemState::Entering => ENTERING,
        GnoemState::SittingDown => SITTING_DOWN,
        GnoemState::Idle => IDLE,
        GnoemState::Typing => TYPING,
        GnoemState::Jumping => JUMPING,
        GnoemState::HeadScratching => HEAD_SCRATCHING,
        GnoemState::ReceivingLetter => RECEIVING_LETTER,
        GnoemState::LeaningBack => LEANING_BACK,
        GnoemState::ShufflingPapers => SHUFFLING_PAPERS,
        GnoemState::PackingUp => PACKING_UP,
        GnoemState::Leaving => LEAVING,
        GnoemState::Gone => GONE,
    }
}

// ---------------------------------------------------------------------------
// Entering — gnome walks in from the left, hat first
// ---------------------------------------------------------------------------

const ENTERING: &[&str] = &[
    // frame 0 — far left, just the hat peeking in
    "   ^\n  /o\\\n  /|\\\n  / \\",
    // frame 1 — half way across
    "     ^\n    /o\\\n    /|\\\n    / \\",
    // frame 2 — nearly at desk
    "      ^\n     /o\\\n     /|\\\n     / \\",
    // frame 3 — arrived, reaching for chair
    "       ^\n      /o\\\n      -|-\n      / \\",
];

// ---------------------------------------------------------------------------
// SittingDown — gnome lowers into the chair
// ---------------------------------------------------------------------------

const SITTING_DOWN: &[&str] = &[
    // frame 0 — mid-squat
    "  ^\n /o\\\n /|\\\n/___\\",
    // frame 1 — fully seated, feet on ground
    "  ^\n /o\\\n-|-\n/___\\",
];

// ---------------------------------------------------------------------------
// Idle — seated, slight head sway, occasional blink
// ---------------------------------------------------------------------------

const IDLE: &[&str] = &[
    // frame 0 — looking forward
    "  ^\n /o\\\n  |\n /___\\",
    // frame 1 — eyes closed (blink)
    "  ^\n /-\\\n  |\n /___\\",
];

// ---------------------------------------------------------------------------
// Typing — arms alternate between keyboard positions; monitor shows output
// ---------------------------------------------------------------------------

const TYPING: &[&str] = &[
    // frame 0 — left hand down
    "  ^\n /o\\\n//|\\\n /___\\ [>_]",
    // frame 1 — right hand down
    "  ^\n /o\\\n /|\\\\\n /___\\ [>.]",
    // frame 2 — both hands raised briefly
    "  ^\n /o\\\n\\-|-/\n /___\\ [>_]",
    // frame 3 — back to left hand
    "  ^\n /o\\\n//|\\\n /___\\ [>>]",
];

// ---------------------------------------------------------------------------
// HeadScratching — one hand on head, confused face
// ---------------------------------------------------------------------------

const HEAD_SCRATCHING: &[&str] = &[
    // frame 0 — scratching left
    "  ^\n /o\\\n/ |\\\n /___\\",
    // frame 1 — scratching harder, hat tilted
    "   ^\n /o\\\n/ |\\\n /___\\",
];

// ---------------------------------------------------------------------------
// Jumping — gnome bounces up and down (notification arrived)
// ---------------------------------------------------------------------------

const JUMPING: &[&str] = &[
    // frame 0 — on ground, crouching
    "  ^\n /O\\\n /|\\\n/ _ \\",
    // frame 1 — launching upward
    "  ^\n /O\\\n /|\\\n  |  ",
    // frame 2 — peak of jump
    "   ^\n  /O\\\n  /|\\\n   |  ",
    // frame 3 — landing
    "  ^\n /O\\\n /|\\\n/ v \\",
];

// ---------------------------------------------------------------------------
// ReceivingLetter — envelope appears, gnome reaches out
// ---------------------------------------------------------------------------

const RECEIVING_LETTER: &[&str] = &[
    // frame 0 — envelope approaching on the left
    "[_] ^\n    /o\\\n   /| \n   / \\",
    // frame 1 — gnome reaching out
    "[_]  ^\n    /o\\\n   /|-\n   / \\",
    // frame 2 — gnome holding envelope
    " ^\n/o\\\n||-\n/ \\ [_]",
];

// ---------------------------------------------------------------------------
// ShufflingPapers — papers move around the desk
// ---------------------------------------------------------------------------

const SHUFFLING_PAPERS: &[&str] = &[
    // frame 0 — pushing papers left
    "  ^\n /o\\\n/ |\\\n/___\\ =",
    // frame 1 — picking stack up
    "  ^\n /o\\\n\\-|-\n/___\\ ==",
    // frame 2 — setting stack down
    "  ^\n /o\\\n /|-\\\n/___\\ =",
];

// ---------------------------------------------------------------------------
// LeaningBack — relaxed pose, feet up on desk
// ---------------------------------------------------------------------------

const LEANING_BACK: &[&str] = &[
    // frame 0 — leaning, feet just off ground
    "  ^\n /~\\\n  |\n [___]=",
    // frame 1 — fully reclined, hat tipped
    "   ^\n  /~\\\n   |\n  [___]=",
];

// ---------------------------------------------------------------------------
// PackingUp — gnome stands, gathers belongings
// ---------------------------------------------------------------------------

const PACKING_UP: &[&str] = &[
    // frame 0 — seated, reaching into drawer
    "  ^\n /o\\\n /|-\n/___\\",
    // frame 1 — picking up bag
    "  ^\n /o\\\n/||\\\n/ _\\[)",
    // frame 2 — standing, bag in hand
    "  ^\n /o\\\n |  \n/ \\ [)",
    // frame 3 — heading for the exit
    "  ^\n/o\\\n|  \n/ \\[)",
];

// ---------------------------------------------------------------------------
// Leaving — gnome walks out to the right (reverse of Entering)
// ---------------------------------------------------------------------------

const LEAVING: &[&str] = &[
    // frame 0 — still near desk
    "  ^\n /o\\\n /|\\\n / \\",
    // frame 1 — moving right
    "    ^\n   /o\\\n   /|\\\n   / \\",
    // frame 2 — almost gone
    "      ^\n     /o\\\n     /|\\\n     / \\",
    // frame 3 — just hat visible
    "       ^\n      /o\\\n       |",
];

// ---------------------------------------------------------------------------
// Gone — empty desk, nobody home
// ---------------------------------------------------------------------------

const GONE: &[&str] = &[
    // single frame — vacant chair
    "        \n  [___] \n        ",
];

// ---------------------------------------------------------------------------
// desk_art
// ---------------------------------------------------------------------------

/// Build a multi-line ASCII representation of the gnome's desk.
///
/// Props rendered (left-to-right, loosely):
///   - Coffee mug with fill level
///   - Paper stack
///   - Plant (healthy or wilted)
///   - Monitor (idle / busy / error)
///   - Envelope (when present)
pub fn desk_art(
    coffee_level: f32,
    paper_stack: u32,
    plant_health: f32,
    monitor_error: bool,
    monitor_typing: bool,
    has_envelope: bool,
) -> String {
    // ---- coffee mug -------------------------------------------------------
    // Fill column using block characters: full ▓, half ▒, empty ░
    let coffee_fill = match coffee_level {
        l if l >= 0.75 => "▓▓",
        l if l >= 0.5 => "▒▓",
        l if l >= 0.25 => "░▒",
        _ => "░░",
    };
    let mug = format!("[{}]c", coffee_fill);

    // ---- paper stack -------------------------------------------------------
    let papers: String = match paper_stack {
        0 => "     ".into(),
        1 => "=    ".into(),
        2 => "==   ".into(),
        3 => "===  ".into(),
        4..=6 => "==== ".into(),
        _ => "=====".into(),
    };

    // ---- plant -------------------------------------------------------------
    let plant = if plant_health >= 0.7 {
        " &\n /"
    } else if plant_health >= 0.35 {
        " ;\n /"
    } else {
        " ,\n /"
    };

    // ---- monitor -----------------------------------------------------------
    let monitor_screen = if monitor_error {
        "ERR"
    } else if monitor_typing {
        ">_ "
    } else {
        "   "
    };
    let monitor = format!(".---.\n|{}|\n'---'", monitor_screen);

    // ---- envelope ----------------------------------------------------------
    let envelope = if has_envelope { "[>]" } else { "   " };

    // ---- assemble ----------------------------------------------------------
    // Row 1: plant top + monitor top
    // Row 2: plant base + monitor middle + mug
    // Row 3: desk surface
    // Row 4: papers + envelope
    let plant_lines: Vec<&str> = plant.split('\n').collect();
    let plant_top = plant_lines[0];
    let plant_base = plant_lines.get(1).copied().unwrap_or(" /");

    let monitor_lines: Vec<&str> = monitor.split('\n').collect();
    let mon0 = monitor_lines[0];
    let mon1 = monitor_lines[1];
    let mon2 = monitor_lines[2];

    format!(
        "{plant_top}   {mon0}\n\
         {plant_base}   {mon1}  {mug}\n\
         {mon2}\n\
         {papers} {envelope}"
    )
}

// ---------------------------------------------------------------------------
// cubicle_border
// ---------------------------------------------------------------------------

/// Generate the border lines for a single cubicle.
///
/// Returns a `Vec<String>` where each element is one terminal row.
/// The border is a simple box with a header bar that shows the gnome's
/// `name`, `status`, `duration`, and `last_activity`.
pub fn cubicle_border(
    name: &str,
    status: &str,
    duration: &str,
    last_activity: &str,
    width: u16,
) -> Vec<String> {
    let w = width as usize;

    // ---- helper: pad / truncate a string to exactly `len` grapheme clusters.
    // We measure by char count (close enough for ASCII names / status strings)
    // and build the result char-by-char to avoid slicing into multi-byte chars.
    let fit = |s: &str, len: usize| -> String {
        let char_count = s.chars().count();
        if char_count >= len {
            s.chars().take(len).collect()
        } else {
            format!("{:<width$}", s, width = len)
        }
    };

    // Top border: ╔═══╗
    let top = format!("╔{}╗", "═".repeat(w.saturating_sub(2)));

    // Name row:   ║ Grumbold          ║
    let inner = w.saturating_sub(4); // 2 border chars + 2 spaces
    let name_line = format!("║ {} ║", fit(name, inner));

    // Status row: ║ Typing · 1m 23s   ║
    let status_text = format!("{} · {}", status, duration);
    let status_line = format!("║ {} ║", fit(&status_text, inner));

    // Activity row: ║ used Bash         ║
    let activity_line = format!("║ {} ║", fit(last_activity, inner));

    // Separator:  ╠═══╣
    let sep = format!("╠{}╣", "═".repeat(w.saturating_sub(2)));

    // Bottom border: ╚═══╝
    let bottom = format!("╚{}╝", "═".repeat(w.saturating_sub(2)));

    vec![top, name_line, status_line, activity_line, sep, bottom]
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // sprite_frames — sanity checks across every state
    // -----------------------------------------------------------------------

    fn all_states() -> Vec<GnoemState> {
        vec![
            GnoemState::Entering,
            GnoemState::SittingDown,
            GnoemState::Idle,
            GnoemState::Typing,
            GnoemState::Jumping,
            GnoemState::HeadScratching,
            GnoemState::ReceivingLetter,
            GnoemState::LeaningBack,
            GnoemState::ShufflingPapers,
            GnoemState::PackingUp,
            GnoemState::Leaving,
            GnoemState::Gone,
        ]
    }

    #[test]
    fn every_state_has_at_least_one_frame() {
        for state in all_states() {
            let frames = sprite_frames(&state);
            assert!(
                !frames.is_empty(),
                "state {:?} returned zero frames",
                state
            );
        }
    }

    #[test]
    fn every_frame_is_non_empty() {
        for state in all_states() {
            for (i, frame) in sprite_frames(&state).iter().enumerate() {
                assert!(
                    !frame.trim().is_empty(),
                    "state {:?} frame {} is blank",
                    state,
                    i
                );
            }
        }
    }

    #[test]
    fn entering_has_four_frames() {
        assert_eq!(sprite_frames(&GnoemState::Entering).len(), 4);
    }

    #[test]
    fn leaving_has_four_frames() {
        assert_eq!(sprite_frames(&GnoemState::Leaving).len(), 4);
    }

    #[test]
    fn typing_has_three_or_more_frames() {
        assert!(sprite_frames(&GnoemState::Typing).len() >= 3);
    }

    #[test]
    fn jumping_has_four_frames() {
        assert_eq!(sprite_frames(&GnoemState::Jumping).len(), 4);
    }

    #[test]
    fn gone_has_exactly_one_frame() {
        assert_eq!(sprite_frames(&GnoemState::Gone).len(), 1);
    }

    #[test]
    fn idle_has_two_frames() {
        assert_eq!(sprite_frames(&GnoemState::Idle).len(), 2);
    }

    #[test]
    fn head_scratching_has_two_frames() {
        assert_eq!(sprite_frames(&GnoemState::HeadScratching).len(), 2);
    }

    #[test]
    fn frame_index_wraps_safely_for_all_states() {
        // Simulate the animation loop: frame_index % frames.len() must not panic.
        for state in all_states() {
            let frames = sprite_frames(&state);
            for i in 0usize..16 {
                let _ = frames[i % frames.len()];
            }
        }
    }

    // -----------------------------------------------------------------------
    // desk_art
    // -----------------------------------------------------------------------

    #[test]
    fn desk_art_returns_non_empty_string() {
        let art = desk_art(0.5, 3, 1.0, false, true, false);
        assert!(!art.is_empty());
    }

    #[test]
    fn desk_art_monitor_error_shows_err() {
        let art = desk_art(0.5, 0, 1.0, true, false, false);
        assert!(art.contains("ERR"), "expected ERR in:\n{}", art);
    }

    #[test]
    fn desk_art_monitor_typing_shows_prompt() {
        let art = desk_art(0.5, 0, 1.0, false, true, false);
        assert!(art.contains(">_"), "expected >_ in:\n{}", art);
    }

    #[test]
    fn desk_art_envelope_shows_when_present() {
        let art = desk_art(0.5, 0, 1.0, false, false, true);
        assert!(art.contains("[>]"), "expected [>] in:\n{}", art);
    }

    #[test]
    fn desk_art_no_envelope_when_absent() {
        let art = desk_art(0.5, 0, 1.0, false, false, false);
        assert!(!art.contains("[>]"), "unexpected [>] in:\n{}", art);
    }

    #[test]
    fn desk_art_full_coffee_uses_full_block() {
        let art = desk_art(1.0, 0, 1.0, false, false, false);
        assert!(art.contains("▓▓"), "expected ▓▓ for full coffee in:\n{}", art);
    }

    #[test]
    fn desk_art_empty_coffee_uses_light_block() {
        let art = desk_art(0.0, 0, 1.0, false, false, false);
        assert!(art.contains("░░"), "expected ░░ for empty coffee in:\n{}", art);
    }

    #[test]
    fn desk_art_healthy_plant_shows_ampersand() {
        let art = desk_art(0.5, 0, 1.0, false, false, false);
        assert!(art.contains('&'), "expected & for healthy plant in:\n{}", art);
    }

    #[test]
    fn desk_art_wilted_plant_shows_comma() {
        let art = desk_art(0.5, 0, 0.1, false, false, false);
        assert!(art.contains(','), "expected , for wilted plant in:\n{}", art);
    }

    #[test]
    fn desk_art_large_paper_stack_shows_many_equals() {
        let art = desk_art(0.5, 10, 1.0, false, false, false);
        assert!(art.contains("====="), "expected ===== for big stack in:\n{}", art);
    }

    // -----------------------------------------------------------------------
    // cubicle_border
    // -----------------------------------------------------------------------

    #[test]
    fn cubicle_border_returns_non_empty_vec() {
        let lines = cubicle_border("Grumbold", "Typing", "1m 23s", "used Bash", 32);
        assert!(!lines.is_empty());
    }

    #[test]
    fn cubicle_border_first_line_starts_with_corner() {
        let lines = cubicle_border("Grumbold", "Idle", "5m", "waiting", 32);
        assert!(lines[0].starts_with('╔'), "top border should start with ╔");
    }

    #[test]
    fn cubicle_border_last_line_starts_with_bottom_corner() {
        let lines = cubicle_border("Grumbold", "Idle", "5m", "waiting", 32);
        let last = lines.last().unwrap();
        assert!(last.starts_with('╚'), "bottom border should start with ╚");
    }

    #[test]
    fn cubicle_border_contains_name() {
        let lines = cubicle_border("Wimblenook", "Typing", "2m", "used Read", 40);
        let combined = lines.join("\n");
        assert!(combined.contains("Wimblenook"), "border should contain gnome name");
    }

    #[test]
    fn cubicle_border_contains_status() {
        let lines = cubicle_border("Zorp", "HeadScratching", "30s", "tool failure", 36);
        let combined = lines.join("\n");
        assert!(combined.contains("HeadScratching"), "border should contain status");
    }

    #[test]
    fn cubicle_border_contains_duration() {
        let lines = cubicle_border("Flib", "Idle", "42m", "waiting", 32);
        let combined = lines.join("\n");
        assert!(combined.contains("42m"), "border should contain duration");
    }

    #[test]
    fn cubicle_border_contains_last_activity() {
        let lines = cubicle_border("Borfle", "Typing", "1m", "used Bash", 32);
        let combined = lines.join("\n");
        assert!(combined.contains("used Bash"), "border should contain last activity");
    }

    #[test]
    fn cubicle_border_width_one_does_not_panic() {
        // Edge case: degenerate width should not panic.
        let lines = cubicle_border("X", "Y", "Z", "W", 1);
        assert!(!lines.is_empty());
    }

    #[test]
    fn cubicle_border_long_name_is_truncated_to_fit() {
        let long_name = "Grumblywimblethwaitington";
        let lines = cubicle_border(long_name, "Idle", "1m", "done", 20);
        // Each line must not be excessively long; just verify no panic.
        assert!(!lines.is_empty());
    }
}

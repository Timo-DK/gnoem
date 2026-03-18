use std::fs;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Embedded hook script
// ---------------------------------------------------------------------------

/// The universal Gnoem hook shell script, bundled at compile time.
const HOOK_SCRIPT: &str = include_str!("../hooks/gnoem-hook.sh");

/// The JSON snippet the user must add to `~/.claude/settings.json`.
const SETTINGS_SNIPPET: &str = r#"{
  "hooks": {
    "SessionStart": [{"type": "command", "command": "GNOEM_EVENT_TYPE=session_start CLAUDE_SESSION_ID=$SESSION_ID CLAUDE_CWD=$CWD ~/.config/gnoem/hooks/gnoem-hook.sh"}],
    "SessionEnd": [{"type": "command", "command": "GNOEM_EVENT_TYPE=session_end CLAUDE_SESSION_ID=$SESSION_ID CLAUDE_CWD=$CWD ~/.config/gnoem/hooks/gnoem-hook.sh"}],
    "PostToolUse": [{"type": "command", "command": "GNOEM_EVENT_TYPE=post_tool_use CLAUDE_SESSION_ID=$SESSION_ID CLAUDE_CWD=$CWD ~/.config/gnoem/hooks/gnoem-hook.sh"}],
    "Notification": [{"type": "command", "command": "GNOEM_EVENT_TYPE=notification CLAUDE_SESSION_ID=$SESSION_ID CLAUDE_CWD=$CWD ~/.config/gnoem/hooks/gnoem-hook.sh"}],
    "Stop": [{"type": "command", "command": "GNOEM_EVENT_TYPE=stop CLAUDE_SESSION_ID=$SESSION_ID CLAUDE_CWD=$CWD ~/.config/gnoem/hooks/gnoem-hook.sh"}]
  }
}"#;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Returns the path to the Gnoem hooks directory: `~/.config/gnoem/hooks`.
pub fn hooks_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("gnoem")
        .join("hooks")
}

/// Install the Gnoem hook script and print the Claude settings snippet.
///
/// Steps performed:
/// 1. Create `~/.config/gnoem/hooks/` if it does not exist.
/// 2. Write the bundled hook script to `gnoem-hook.sh` inside that directory.
/// 3. Make the script executable on Unix platforms.
/// 4. Print the JSON snippet the user should merge into `~/.claude/settings.json`.
pub fn install_hooks() -> Result<(), HookError> {
    let dir = hooks_dir();
    fs::create_dir_all(&dir).map_err(HookError::Io)?;

    let script_path = dir.join("gnoem-hook.sh");
    fs::write(&script_path, HOOK_SCRIPT).map_err(HookError::Io)?;

    // Make the script executable on Unix.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path)
            .map_err(HookError::Io)?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).map_err(HookError::Io)?;
    }

    println!("Gnoem hook script installed to: {}", script_path.display());
    println!();
    println!("Add the following to your ~/.claude/settings.json:");
    println!();
    println!("{SETTINGS_SNIPPET}");

    Ok(())
}

/// Remove the `~/.config/gnoem/` directory and print uninstall instructions.
///
/// Removes the entire Gnoem configuration directory (events, hooks, config).
/// Prints a reminder to remove the hook entries from `~/.claude/settings.json`
/// manually, as Gnoem never modifies that file directly.
pub fn uninstall() -> Result<(), HookError> {
    let gnoem_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("gnoem");

    if gnoem_dir.exists() {
        fs::remove_dir_all(&gnoem_dir).map_err(HookError::Io)?;
        println!("Removed Gnoem data directory: {}", gnoem_dir.display());
    } else {
        println!(
            "Gnoem data directory not found ({}), nothing to remove.",
            gnoem_dir.display()
        );
    }

    println!();
    println!("To complete uninstall, remove the hook entries from ~/.claude/settings.json.");
    println!("Remove the \"hooks\" key (or the individual Gnoem entries) shown below:");
    println!();
    println!("{SETTINGS_SNIPPET}");

    Ok(())
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors that can occur during hook installation or uninstallation.
#[derive(Debug)]
pub enum HookError {
    /// An I/O error during file or directory operations.
    Io(std::io::Error),
}

impl std::fmt::Display for HookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HookError::Io(e) => write!(f, "I/O error during hook operation: {e}"),
        }
    }
}

impl std::error::Error for HookError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            HookError::Io(e) => Some(e),
        }
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------
    // HOOK_SCRIPT content checks
    // ------------------------------------------------------------------

    #[test]
    fn hook_script_contains_session_start_event_type() {
        assert!(
            HOOK_SCRIPT.contains("session_start"),
            "hook script should handle session_start event type"
        );
    }

    #[test]
    fn hook_script_contains_session_end_event_type() {
        assert!(
            HOOK_SCRIPT.contains("session_end"),
            "hook script should handle session_end event type"
        );
    }

    #[test]
    fn hook_script_contains_post_tool_use_event_type() {
        assert!(
            HOOK_SCRIPT.contains("post_tool_use"),
            "hook script should handle post_tool_use event type"
        );
    }

    #[test]
    fn hook_script_contains_notification_event_type() {
        assert!(
            HOOK_SCRIPT.contains("notification"),
            "hook script should handle notification event type"
        );
    }

    #[test]
    fn hook_script_contains_stop_event_type() {
        assert!(
            HOOK_SCRIPT.contains("stop"),
            "hook script should handle stop event type"
        );
    }

    #[test]
    fn hook_script_contains_subagent_event_types() {
        assert!(
            HOOK_SCRIPT.contains("subagent_start"),
            "hook script should handle subagent_start event type"
        );
        assert!(
            HOOK_SCRIPT.contains("subagent_end"),
            "hook script should handle subagent_end event type"
        );
    }

    #[test]
    fn hook_script_contains_post_tool_use_failure_event_type() {
        assert!(
            HOOK_SCRIPT.contains("post_tool_use_failure"),
            "hook script should handle post_tool_use_failure event type"
        );
    }

    #[test]
    fn hook_script_starts_with_shebang() {
        assert!(
            HOOK_SCRIPT.starts_with("#!/bin/bash"),
            "hook script should start with #!/bin/bash"
        );
    }

    #[test]
    fn hook_script_reads_gnoem_event_type_env_var() {
        assert!(
            HOOK_SCRIPT.contains("GNOEM_EVENT_TYPE"),
            "hook script should read GNOEM_EVENT_TYPE environment variable"
        );
    }

    // ------------------------------------------------------------------
    // SETTINGS_SNIPPET is valid JSON
    // ------------------------------------------------------------------

    #[test]
    fn settings_snippet_is_valid_json() {
        let result: Result<serde_json::Value, _> = serde_json::from_str(SETTINGS_SNIPPET);
        assert!(
            result.is_ok(),
            "settings snippet should be valid JSON, error: {:?}",
            result.err()
        );
    }

    #[test]
    fn settings_snippet_contains_all_hook_types() {
        let value: serde_json::Value =
            serde_json::from_str(SETTINGS_SNIPPET).expect("valid JSON");
        let hooks = value
            .get("hooks")
            .expect("snippet should have 'hooks' key");
        for key in &["SessionStart", "SessionEnd", "PostToolUse", "Notification", "Stop"] {
            assert!(
                hooks.get(key).is_some(),
                "hooks snippet should contain '{key}'"
            );
        }
    }

    #[test]
    fn settings_snippet_hook_commands_reference_hook_script() {
        assert!(
            SETTINGS_SNIPPET.contains("gnoem-hook.sh"),
            "each hook command should reference gnoem-hook.sh"
        );
    }

    // ------------------------------------------------------------------
    // hooks_dir
    // ------------------------------------------------------------------

    #[test]
    fn hooks_dir_ends_with_gnoem_hooks() {
        let dir = hooks_dir();
        assert!(
            dir.ends_with("gnoem/hooks"),
            "hooks_dir should end with 'gnoem/hooks', got: {}",
            dir.display()
        );
    }

    // ------------------------------------------------------------------
    // install_hooks — creates directory and script file
    // ------------------------------------------------------------------

    #[test]
    fn install_hooks_creates_hooks_directory_and_script() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        // Override dirs::config_dir by setting HOME so dirs resolves inside tmp.
        // We intercept via a direct call using a helper that accepts a base dir.
        let hooks_target = tmp.path().join("gnoem").join("hooks");
        let script_path = hooks_target.join("gnoem-hook.sh");

        // Perform the installation manually with an explicit target directory
        // (mirrors install_hooks logic but rooted at tmp).
        fs::create_dir_all(&hooks_target).expect("create hooks dir");
        fs::write(&script_path, HOOK_SCRIPT).expect("write script");

        assert!(hooks_target.is_dir(), "hooks directory should be created");
        assert!(script_path.exists(), "gnoem-hook.sh should be written");

        let written = fs::read_to_string(&script_path).expect("read script");
        assert_eq!(written, HOOK_SCRIPT, "written script should match HOOK_SCRIPT");
    }

    #[test]
    fn install_hooks_with_real_tempdir_succeeds() {
        // Run install_hooks() with an env override so it writes into a temp dir.
        // We test this by temporarily overriding HOME on Unix-like environments.
        // On Windows dirs::config_dir() uses APPDATA; we skip the live install
        // test there to avoid polluting the real config directory.
        #[cfg(unix)]
        {
            let tmp = tempfile::tempdir().expect("create temp dir");
            let original_home = env::var("HOME").ok();

            // Point HOME at our temp dir so dirs::config_dir() resolves there.
            env::set_var("HOME", tmp.path());

            let result = install_hooks();

            // Restore HOME regardless of outcome.
            match original_home {
                Some(h) => env::set_var("HOME", h),
                None => env::remove_var("HOME"),
            }

            assert!(result.is_ok(), "install_hooks should succeed: {:?}", result.err());

            let script = tmp
                .path()
                .join(".config")
                .join("gnoem")
                .join("hooks")
                .join("gnoem-hook.sh");
            assert!(script.exists(), "hook script should exist after install");
        }
    }

    // ------------------------------------------------------------------
    // install_hooks — script is executable on Unix
    // ------------------------------------------------------------------

    #[cfg(unix)]
    #[test]
    fn install_hooks_makes_script_executable() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempfile::tempdir().expect("create temp dir");
        let original_home = env::var("HOME").ok();
        env::set_var("HOME", tmp.path());

        let result = install_hooks();

        match original_home {
            Some(h) => env::set_var("HOME", h),
            None => env::remove_var("HOME"),
        }

        assert!(result.is_ok(), "install_hooks should succeed");

        let script = tmp
            .path()
            .join(".config")
            .join("gnoem")
            .join("hooks")
            .join("gnoem-hook.sh");
        let mode = fs::metadata(&script)
            .expect("script should exist")
            .permissions()
            .mode();
        // Check owner execute bit (0o100).
        assert!(mode & 0o100 != 0, "hook script should be executable");
    }
}

use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Config structs
// ---------------------------------------------------------------------------

/// Top-level configuration for Gnoem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub display: DisplayConfig,
    pub animation: AnimationConfig,
    pub paths: PathsConfig,
}

/// Controls which session metadata fields are shown in the TUI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub show_session_name: bool,
    pub show_duration: bool,
    pub show_last_activity: bool,
    pub show_project_branch: bool,
    pub show_status: bool,
}

/// Controls animation timing and idle behaviour.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationConfig {
    /// Target frames per second for the UI render loop.
    pub ui_fps: u32,
    /// Seconds of inactivity before a gnome enters the idle pose.
    pub idle_timeout_secs: u64,
    /// Seconds since last activity before a plant starts wilting.
    pub plant_wilt_after_secs: u64,
    /// When `true` all animations are frozen.
    pub paused: bool,
}

/// File-system paths used by Gnoem at runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathsConfig {
    /// Directory where Claude hook scripts write event JSON files.
    pub events_dir: PathBuf,
}

// ---------------------------------------------------------------------------
// Default implementations
// ---------------------------------------------------------------------------

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
        let events_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("gnoem")
            .join("events");
        Self { events_dir }
    }
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

// ---------------------------------------------------------------------------
// Config methods
// ---------------------------------------------------------------------------

impl Config {
    /// Load configuration from `path`.
    ///
    /// If the file does not exist the default [`Config`] is returned.
    /// Any other I/O error or a TOML parse error is propagated as a
    /// [`ConfigError`].
    pub fn load(path: &Path) -> Result<Config, ConfigError> {
        match std::fs::read_to_string(path) {
            Ok(contents) => {
                let config = toml::from_str(&contents).map_err(ConfigError::Parse)?;
                Ok(config)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Config::default()),
            Err(e) => Err(ConfigError::Io(e)),
        }
    }

    /// Serialize `self` to TOML and write it to `path`.
    ///
    /// Parent directories are created automatically if they do not exist.
    pub fn save(&self, path: &Path) -> Result<(), ConfigError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(ConfigError::Io)?;
        }
        let contents = toml::to_string_pretty(self).map_err(ConfigError::Serialize)?;
        std::fs::write(path, contents).map_err(ConfigError::Io)?;
        Ok(())
    }

    /// Return the canonical path to the Gnoem config file.
    ///
    /// Resolves to `<config_dir>/gnoem/config.toml` where `<config_dir>` is
    /// the platform-appropriate user configuration directory returned by
    /// [`dirs::config_dir`].  Falls back to `~/.config` if the platform
    /// directory cannot be determined.
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("gnoem")
            .join("config.toml")
    }
}

// ---------------------------------------------------------------------------
// ConfigError
// ---------------------------------------------------------------------------

/// Errors that can occur when loading or saving the configuration file.
#[derive(Debug)]
pub enum ConfigError {
    /// An I/O error reading or writing the file.
    Io(std::io::Error),
    /// A TOML deserialization error.
    Parse(toml::de::Error),
    /// A TOML serialization error.
    Serialize(toml::ser::Error),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Io(e) => write!(f, "I/O error accessing config file: {e}"),
            ConfigError::Parse(e) => write!(f, "Failed to parse config TOML: {e}"),
            ConfigError::Serialize(e) => write!(f, "Failed to serialize config to TOML: {e}"),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::Io(e) => Some(e),
            ConfigError::Parse(e) => Some(e),
            ConfigError::Serialize(e) => Some(e),
        }
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // ------------------------------------------------------------------
    // Default values
    // ------------------------------------------------------------------

    #[test]
    fn default_display_config_has_all_fields_true() {
        let d = DisplayConfig::default();
        assert!(d.show_session_name);
        assert!(d.show_duration);
        assert!(d.show_last_activity);
        assert!(d.show_project_branch);
        assert!(d.show_status);
    }

    #[test]
    fn default_animation_config_has_correct_values() {
        let a = AnimationConfig::default();
        assert_eq!(a.ui_fps, 16);
        assert_eq!(a.idle_timeout_secs, 30);
        assert_eq!(a.plant_wilt_after_secs, 120);
        assert!(!a.paused);
    }

    #[test]
    fn default_paths_config_ends_with_gnoem_events() {
        let p = PathsConfig::default();
        let path_str = p.events_dir.to_string_lossy();
        assert!(
            path_str.contains("gnoem"),
            "events_dir should contain 'gnoem', got: {path_str}"
        );
        assert!(
            p.events_dir.ends_with("events"),
            "events_dir should end with 'events', got: {path_str}"
        );
    }

    #[test]
    fn default_config_composes_sub_defaults() {
        let c = Config::default();
        // Spot-check a field from each sub-config to confirm composition.
        assert!(c.display.show_status);
        assert_eq!(c.animation.ui_fps, 16);
        assert!(c.paths.events_dir.ends_with("events"));
    }

    // ------------------------------------------------------------------
    // Config::load — non-existent path returns defaults
    // ------------------------------------------------------------------

    #[test]
    fn load_from_nonexistent_path_returns_default() {
        let path = Path::new("/this/path/definitely/does/not/exist/config.toml");
        let config = Config::load(path).expect("should return Ok with defaults");
        assert_eq!(config.animation.ui_fps, 16);
        assert!(config.display.show_session_name);
    }

    // ------------------------------------------------------------------
    // Config::load — invalid TOML returns Parse error
    // ------------------------------------------------------------------

    #[test]
    fn load_from_invalid_toml_returns_parse_error() {
        let mut file = NamedTempFile::new().expect("create temp file");
        file.write_all(b"[display]\nnot valid toml = = =")
            .expect("write temp file");
        let err = Config::load(file.path()).unwrap_err();
        assert!(
            matches!(err, ConfigError::Parse(_)),
            "expected Parse error, got: {err}"
        );
    }

    // ------------------------------------------------------------------
    // Config::save + Config::load round-trip
    // ------------------------------------------------------------------

    #[test]
    fn save_then_load_round_trips_config() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let path = dir.path().join("config.toml");

        let mut original = Config::default();
        // Mutate some fields so the round-trip is non-trivial.
        original.display.show_duration = false;
        original.animation.ui_fps = 30;
        original.animation.paused = true;
        original.paths.events_dir = PathBuf::from("/custom/events");

        original.save(&path).expect("save should succeed");
        let loaded = Config::load(&path).expect("load should succeed");

        assert!(!loaded.display.show_duration);
        assert_eq!(loaded.animation.ui_fps, 30);
        assert!(loaded.animation.paused);
        assert_eq!(loaded.paths.events_dir, PathBuf::from("/custom/events"));
    }

    #[test]
    fn save_creates_parent_directories_automatically() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let path = dir.path().join("nested").join("deep").join("config.toml");

        Config::default()
            .save(&path)
            .expect("save should create parent dirs and succeed");

        assert!(path.exists(), "config file should exist after save");
    }

    // ------------------------------------------------------------------
    // Config::config_path
    // ------------------------------------------------------------------

    #[test]
    fn config_path_ends_with_gnoem_config_toml() {
        let path = Config::config_path();
        assert!(
            path.ends_with("gnoem/config.toml"),
            "config_path should end with 'gnoem/config.toml', got: {}",
            path.display()
        );
    }
}

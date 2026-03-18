use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// GnoemIdentity
// ---------------------------------------------------------------------------

/// A persistent identity for a gnome associated with a working directory.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GnoemIdentity {
    pub name: String,
    /// Color stored as a lowercase name, e.g. `"cyan"`, `"magenta"`.
    pub color: String,
}

// ---------------------------------------------------------------------------
// GnoemRegistry
// ---------------------------------------------------------------------------

/// In-memory registry of gnome identities, keyed by working-directory path.
/// Backed by a TOML file on disk.
pub struct GnoemRegistry {
    /// Path to the TOML persistence file.
    path: PathBuf,
    /// Map from cwd string to identity.
    identities: HashMap<String, GnoemIdentity>,
}

impl GnoemRegistry {
    /// Create a new, empty registry that will persist to `path`.
    pub fn new(path: PathBuf) -> Self {
        GnoemRegistry {
            path,
            identities: HashMap::new(),
        }
    }

    /// Load the registry from a TOML file at `path`.
    ///
    /// If the file does not exist an empty registry is returned.
    /// Any other I/O error, or a parse error, is propagated as a
    /// [`PersistenceError`].
    pub fn load(path: PathBuf) -> Result<Self, PersistenceError> {
        match std::fs::read_to_string(&path) {
            Ok(contents) => {
                let identities: HashMap<String, GnoemIdentity> =
                    toml::from_str(&contents).map_err(PersistenceError::Parse)?;
                Ok(GnoemRegistry { path, identities })
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Ok(GnoemRegistry::new(path))
            }
            Err(e) => Err(PersistenceError::Io(e)),
        }
    }

    /// Write the registry to disk as TOML, creating parent directories as
    /// needed.
    pub fn save(&self) -> Result<(), PersistenceError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(PersistenceError::Io)?;
        }
        let contents =
            toml::to_string(&self.identities).map_err(PersistenceError::Serialize)?;
        std::fs::write(&self.path, contents).map_err(PersistenceError::Io)?;
        Ok(())
    }

    /// Look up the identity for a working directory.
    pub fn get(&self, cwd: &str) -> Option<&GnoemIdentity> {
        self.identities.get(cwd)
    }

    /// Add or update the identity for a working directory.
    pub fn insert(&mut self, cwd: String, identity: GnoemIdentity) {
        self.identities.insert(cwd, identity);
    }

    /// Returns the canonical path for the gnoems registry file:
    /// `~/.config/gnoem/gnoems.toml`.
    pub fn registry_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from(".config"));
        path.push("gnoem");
        path.push("gnoems.toml");
        path
    }
}

// ---------------------------------------------------------------------------
// Color helpers
// ---------------------------------------------------------------------------

/// Convert a [`ratatui::style::Color`] to a lowercase string name.
///
/// Named colors map to their lowercase names. Indexed and RGB variants
/// fall back to `"white"`.
pub fn color_to_string(color: Color) -> String {
    match color {
        Color::Black => "black".into(),
        Color::Red => "red".into(),
        Color::Green => "green".into(),
        Color::Yellow => "yellow".into(),
        Color::Blue => "blue".into(),
        Color::Magenta => "magenta".into(),
        Color::Cyan => "cyan".into(),
        Color::Gray => "gray".into(),
        Color::DarkGray => "darkgray".into(),
        Color::LightRed => "lightred".into(),
        Color::LightGreen => "lightgreen".into(),
        Color::LightYellow => "lightyellow".into(),
        Color::LightBlue => "lightblue".into(),
        Color::LightMagenta => "lightmagenta".into(),
        Color::LightCyan => "lightcyan".into(),
        Color::White => "white".into(),
        // Indexed / RGB variants are not stored by name; fall back gracefully.
        _ => "white".into(),
    }
}

/// Parse a lowercase color name back into a [`ratatui::style::Color`].
///
/// Returns [`Color::White`] for unrecognised strings.
pub fn string_to_color(s: &str) -> Color {
    match s {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "gray" => Color::Gray,
        "darkgray" => Color::DarkGray,
        "lightred" => Color::LightRed,
        "lightgreen" => Color::LightGreen,
        "lightyellow" => Color::LightYellow,
        "lightblue" => Color::LightBlue,
        "lightmagenta" => Color::LightMagenta,
        "lightcyan" => Color::LightCyan,
        "white" => Color::White,
        _ => Color::White,
    }
}

// ---------------------------------------------------------------------------
// PersistenceError
// ---------------------------------------------------------------------------

/// Errors that can occur when loading or saving the registry.
#[derive(Debug)]
pub enum PersistenceError {
    /// An I/O error (e.g. permission denied, disk full).
    Io(std::io::Error),
    /// A TOML deserialisation error.
    Parse(toml::de::Error),
    /// A TOML serialisation error.
    Serialize(toml::ser::Error),
}

impl fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PersistenceError::Io(e) => write!(f, "I/O error accessing registry: {e}"),
            PersistenceError::Parse(e) => write!(f, "Failed to parse registry TOML: {e}"),
            PersistenceError::Serialize(e) => {
                write!(f, "Failed to serialise registry TOML: {e}")
            }
        }
    }
}

impl std::error::Error for PersistenceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PersistenceError::Io(e) => Some(e),
            PersistenceError::Parse(e) => Some(e),
            PersistenceError::Serialize(e) => Some(e),
        }
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn temp_registry_path(dir: &TempDir) -> PathBuf {
        dir.path().join("gnoems.toml")
    }

    fn sample_identity(name: &str, color: &str) -> GnoemIdentity {
        GnoemIdentity {
            name: name.into(),
            color: color.into(),
        }
    }

    // -----------------------------------------------------------------------
    // GnoemRegistry::new
    // -----------------------------------------------------------------------

    #[test]
    fn new_creates_empty_registry() {
        let path = PathBuf::from("/tmp/gnoems_test.toml");
        let registry = GnoemRegistry::new(path.clone());
        assert_eq!(registry.path, path);
        assert!(registry.identities.is_empty());
    }

    // -----------------------------------------------------------------------
    // insert and get
    // -----------------------------------------------------------------------

    #[test]
    fn insert_and_get_round_trip() {
        let mut registry = GnoemRegistry::new(PathBuf::from("/dev/null"));
        let identity = sample_identity("Grumbold", "cyan");
        registry.insert("/home/user/project".into(), identity.clone());

        let retrieved = registry.get("/home/user/project");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), &identity);
    }

    #[test]
    fn insert_overwrites_existing_entry() {
        let mut registry = GnoemRegistry::new(PathBuf::from("/dev/null"));
        registry.insert("/repo".into(), sample_identity("Grumbold", "cyan"));
        registry.insert("/repo".into(), sample_identity("Fizwick", "magenta"));

        let retrieved = registry.get("/repo").unwrap();
        assert_eq!(retrieved.name, "Fizwick");
        assert_eq!(retrieved.color, "magenta");
    }

    // -----------------------------------------------------------------------
    // get for unknown cwd
    // -----------------------------------------------------------------------

    #[test]
    fn get_returns_none_for_unknown_cwd() {
        let registry = GnoemRegistry::new(PathBuf::from("/dev/null"));
        assert!(registry.get("/nonexistent/path").is_none());
    }

    // -----------------------------------------------------------------------
    // save → load round-trip
    // -----------------------------------------------------------------------

    #[test]
    fn save_then_load_round_trip() {
        let dir = TempDir::new().expect("create tempdir");
        let path = temp_registry_path(&dir);

        let mut original = GnoemRegistry::new(path.clone());
        original.insert(
            "/home/user/my-project".into(),
            sample_identity("Grumbold", "cyan"),
        );
        original.insert(
            "/home/user/auth-service".into(),
            sample_identity("Fizwick", "magenta"),
        );
        original.save().expect("save registry");

        let loaded = GnoemRegistry::load(path).expect("load registry");
        assert_eq!(
            loaded.get("/home/user/my-project"),
            Some(&sample_identity("Grumbold", "cyan"))
        );
        assert_eq!(
            loaded.get("/home/user/auth-service"),
            Some(&sample_identity("Fizwick", "magenta"))
        );
    }

    #[test]
    fn save_creates_parent_directories() {
        let dir = TempDir::new().expect("create tempdir");
        // Nest the file several levels deep — none of the subdirs exist yet.
        let path = dir.path().join("a").join("b").join("gnoems.toml");

        let registry = GnoemRegistry::new(path.clone());
        registry.save().expect("save should create parent dirs");

        assert!(path.exists(), "registry file should exist after save");
    }

    // -----------------------------------------------------------------------
    // load from non-existent file
    // -----------------------------------------------------------------------

    #[test]
    fn load_from_nonexistent_file_returns_empty_registry() {
        let path = PathBuf::from("/this/path/does/not/exist/gnoems.toml");
        let registry = GnoemRegistry::load(path).expect("should not error for missing file");
        assert!(registry.identities.is_empty());
    }

    // -----------------------------------------------------------------------
    // registry_path
    // -----------------------------------------------------------------------

    #[test]
    fn registry_path_ends_with_expected_suffix() {
        let path = GnoemRegistry::registry_path();
        assert!(
            path.ends_with("gnoem/gnoems.toml"),
            "unexpected path: {path:?}"
        );
    }

    // -----------------------------------------------------------------------
    // color_to_string / string_to_color
    // -----------------------------------------------------------------------

    /// All named Color variants that have explicit string mappings.
    fn all_named_colors() -> Vec<(Color, &'static str)> {
        vec![
            (Color::Black, "black"),
            (Color::Red, "red"),
            (Color::Green, "green"),
            (Color::Yellow, "yellow"),
            (Color::Blue, "blue"),
            (Color::Magenta, "magenta"),
            (Color::Cyan, "cyan"),
            (Color::Gray, "gray"),
            (Color::DarkGray, "darkgray"),
            (Color::LightRed, "lightred"),
            (Color::LightGreen, "lightgreen"),
            (Color::LightYellow, "lightyellow"),
            (Color::LightBlue, "lightblue"),
            (Color::LightMagenta, "lightmagenta"),
            (Color::LightCyan, "lightcyan"),
            (Color::White, "white"),
        ]
    }

    #[test]
    fn color_to_string_produces_correct_names() {
        for (color, expected) in all_named_colors() {
            assert_eq!(
                color_to_string(color),
                expected,
                "color_to_string({color:?}) should be \"{expected}\""
            );
        }
    }

    #[test]
    fn string_to_color_produces_correct_variants() {
        for (expected_color, name) in all_named_colors() {
            assert_eq!(
                string_to_color(name),
                expected_color,
                "string_to_color(\"{name}\") should be {expected_color:?}"
            );
        }
    }

    #[test]
    fn color_round_trip_for_all_named_colors() {
        for (color, _) in all_named_colors() {
            let name = color_to_string(color);
            let restored = string_to_color(&name);
            assert_eq!(
                restored, color,
                "round-trip failed for {color:?}: got {restored:?}"
            );
        }
    }

    #[test]
    fn string_to_color_unknown_returns_white() {
        assert_eq!(string_to_color("purple"), Color::White);
        assert_eq!(string_to_color(""), Color::White);
        assert_eq!(string_to_color("CYAN"), Color::White); // case-sensitive
        assert_eq!(string_to_color("0xff0000"), Color::White);
    }

    #[test]
    fn color_to_string_indexed_falls_back_to_white() {
        assert_eq!(color_to_string(Color::Indexed(42)), "white");
    }

    #[test]
    fn color_to_string_rgb_falls_back_to_white() {
        assert_eq!(color_to_string(Color::Rgb(255, 128, 0)), "white");
    }

    // -----------------------------------------------------------------------
    // PersistenceError Display
    // -----------------------------------------------------------------------

    #[test]
    fn persistence_error_display_io_contains_context() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let err = PersistenceError::Io(io_err);
        let msg = err.to_string();
        assert!(msg.contains("I/O error"), "unexpected message: {msg}");
    }

    #[test]
    fn persistence_error_display_parse_contains_context() {
        let parse_err = toml::from_str::<HashMap<String, GnoemIdentity>>("!!!").unwrap_err();
        let err = PersistenceError::Parse(parse_err);
        let msg = err.to_string();
        assert!(msg.contains("parse registry TOML"), "unexpected message: {msg}");
    }
}

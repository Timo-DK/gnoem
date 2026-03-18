use std::fmt;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// All events that Claude hooks can emit, written as atomic JSON files to
/// `~/.config/gnoem/events/`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum Event {
    SessionStart {
        session_id: String,
        cwd: String,
        timestamp: u64,
    },
    SessionEnd {
        session_id: String,
        cwd: String,
        timestamp: u64,
    },
    PostToolUse {
        session_id: String,
        cwd: String,
        timestamp: u64,
        tool_name: String,
    },
    PostToolUseFailure {
        session_id: String,
        cwd: String,
        timestamp: u64,
        tool_name: String,
        error: String,
    },
    UserPromptSubmit {
        session_id: String,
        cwd: String,
        timestamp: u64,
    },
    Notification {
        session_id: String,
        cwd: String,
        timestamp: u64,
        message: String,
    },
    SubagentStart {
        session_id: String,
        cwd: String,
        timestamp: u64,
        subagent_id: String,
    },
    SubagentEnd {
        session_id: String,
        cwd: String,
        timestamp: u64,
        subagent_id: String,
    },
    PreCompact {
        session_id: String,
        cwd: String,
        timestamp: u64,
    },
    Stop {
        session_id: String,
        cwd: String,
        timestamp: u64,
    },
}

impl Event {
    /// Read and deserialize an event from a JSON file.
    pub fn from_file(path: &Path) -> Result<Event, EventError> {
        let contents = std::fs::read_to_string(path).map_err(EventError::Io)?;
        let event = serde_json::from_str(&contents).map_err(EventError::Parse)?;
        Ok(event)
    }

    /// Return the session ID common to all event variants.
    pub fn session_id(&self) -> &str {
        match self {
            Event::SessionStart { session_id, .. }
            | Event::SessionEnd { session_id, .. }
            | Event::PostToolUse { session_id, .. }
            | Event::PostToolUseFailure { session_id, .. }
            | Event::UserPromptSubmit { session_id, .. }
            | Event::Notification { session_id, .. }
            | Event::SubagentStart { session_id, .. }
            | Event::SubagentEnd { session_id, .. }
            | Event::PreCompact { session_id, .. }
            | Event::Stop { session_id, .. } => session_id,
        }
    }

    /// Return the working directory common to all event variants.
    pub fn cwd(&self) -> &str {
        match self {
            Event::SessionStart { cwd, .. }
            | Event::SessionEnd { cwd, .. }
            | Event::PostToolUse { cwd, .. }
            | Event::PostToolUseFailure { cwd, .. }
            | Event::UserPromptSubmit { cwd, .. }
            | Event::Notification { cwd, .. }
            | Event::SubagentStart { cwd, .. }
            | Event::SubagentEnd { cwd, .. }
            | Event::PreCompact { cwd, .. }
            | Event::Stop { cwd, .. } => cwd,
        }
    }

    /// Return the Unix timestamp common to all event variants.
    pub fn timestamp(&self) -> u64 {
        match self {
            Event::SessionStart { timestamp, .. }
            | Event::SessionEnd { timestamp, .. }
            | Event::PostToolUse { timestamp, .. }
            | Event::PostToolUseFailure { timestamp, .. }
            | Event::UserPromptSubmit { timestamp, .. }
            | Event::Notification { timestamp, .. }
            | Event::SubagentStart { timestamp, .. }
            | Event::SubagentEnd { timestamp, .. }
            | Event::PreCompact { timestamp, .. }
            | Event::Stop { timestamp, .. } => *timestamp,
        }
    }
}

// ---------------------------------------------------------------------------
// EventError
// ---------------------------------------------------------------------------

/// Errors that can occur when reading or parsing an event file.
#[derive(Debug)]
pub enum EventError {
    /// An I/O error while reading the file (e.g. file not found).
    Io(std::io::Error),
    /// A JSON parse error.
    Parse(serde_json::Error),
}

impl fmt::Display for EventError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventError::Io(e) => write!(f, "I/O error reading event file: {e}"),
            EventError::Parse(e) => write!(f, "Failed to parse event JSON: {e}"),
        }
    }
}

impl std::error::Error for EventError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            EventError::Io(e) => Some(e),
            EventError::Parse(e) => Some(e),
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
    // Helpers
    // ------------------------------------------------------------------

    fn write_temp(json: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().expect("create temp file");
        file.write_all(json.as_bytes()).expect("write temp file");
        file
    }

    // ------------------------------------------------------------------
    // Deserialization: one test per event variant
    // ------------------------------------------------------------------

    #[test]
    fn deserializes_session_start() {
        let json = r#"{"event":"session_start","session_id":"s1","cwd":"/home/user","timestamp":1000}"#;
        let event: Event = serde_json::from_str(json).unwrap();
        assert_eq!(
            event,
            Event::SessionStart {
                session_id: "s1".into(),
                cwd: "/home/user".into(),
                timestamp: 1000,
            }
        );
    }

    #[test]
    fn deserializes_session_end() {
        let json = r#"{"event":"session_end","session_id":"s2","cwd":"/tmp","timestamp":2000}"#;
        let event: Event = serde_json::from_str(json).unwrap();
        assert_eq!(
            event,
            Event::SessionEnd {
                session_id: "s2".into(),
                cwd: "/tmp".into(),
                timestamp: 2000,
            }
        );
    }

    #[test]
    fn deserializes_post_tool_use() {
        let json = r#"{"event":"post_tool_use","session_id":"s3","cwd":"/proj","timestamp":3000,"tool_name":"Bash"}"#;
        let event: Event = serde_json::from_str(json).unwrap();
        assert_eq!(
            event,
            Event::PostToolUse {
                session_id: "s3".into(),
                cwd: "/proj".into(),
                timestamp: 3000,
                tool_name: "Bash".into(),
            }
        );
    }

    #[test]
    fn deserializes_post_tool_use_failure() {
        let json = r#"{"event":"post_tool_use_failure","session_id":"s4","cwd":"/proj","timestamp":4000,"tool_name":"Read","error":"file not found"}"#;
        let event: Event = serde_json::from_str(json).unwrap();
        assert_eq!(
            event,
            Event::PostToolUseFailure {
                session_id: "s4".into(),
                cwd: "/proj".into(),
                timestamp: 4000,
                tool_name: "Read".into(),
                error: "file not found".into(),
            }
        );
    }

    #[test]
    fn deserializes_user_prompt_submit() {
        let json = r#"{"event":"user_prompt_submit","session_id":"s5","cwd":"/home","timestamp":5000}"#;
        let event: Event = serde_json::from_str(json).unwrap();
        assert_eq!(
            event,
            Event::UserPromptSubmit {
                session_id: "s5".into(),
                cwd: "/home".into(),
                timestamp: 5000,
            }
        );
    }

    #[test]
    fn deserializes_notification() {
        let json = r#"{"event":"notification","session_id":"s6","cwd":"/ws","timestamp":6000,"message":"Task complete"}"#;
        let event: Event = serde_json::from_str(json).unwrap();
        assert_eq!(
            event,
            Event::Notification {
                session_id: "s6".into(),
                cwd: "/ws".into(),
                timestamp: 6000,
                message: "Task complete".into(),
            }
        );
    }

    #[test]
    fn deserializes_subagent_start() {
        let json = r#"{"event":"subagent_start","session_id":"s7","cwd":"/a","timestamp":7000,"subagent_id":"sub-1"}"#;
        let event: Event = serde_json::from_str(json).unwrap();
        assert_eq!(
            event,
            Event::SubagentStart {
                session_id: "s7".into(),
                cwd: "/a".into(),
                timestamp: 7000,
                subagent_id: "sub-1".into(),
            }
        );
    }

    #[test]
    fn deserializes_subagent_end() {
        let json = r#"{"event":"subagent_end","session_id":"s8","cwd":"/b","timestamp":8000,"subagent_id":"sub-1"}"#;
        let event: Event = serde_json::from_str(json).unwrap();
        assert_eq!(
            event,
            Event::SubagentEnd {
                session_id: "s8".into(),
                cwd: "/b".into(),
                timestamp: 8000,
                subagent_id: "sub-1".into(),
            }
        );
    }

    #[test]
    fn deserializes_pre_compact() {
        let json = r#"{"event":"pre_compact","session_id":"s9","cwd":"/c","timestamp":9000}"#;
        let event: Event = serde_json::from_str(json).unwrap();
        assert_eq!(
            event,
            Event::PreCompact {
                session_id: "s9".into(),
                cwd: "/c".into(),
                timestamp: 9000,
            }
        );
    }

    #[test]
    fn deserializes_stop() {
        let json = r#"{"event":"stop","session_id":"s10","cwd":"/d","timestamp":10000}"#;
        let event: Event = serde_json::from_str(json).unwrap();
        assert_eq!(
            event,
            Event::Stop {
                session_id: "s10".into(),
                cwd: "/d".into(),
                timestamp: 10000,
            }
        );
    }

    // ------------------------------------------------------------------
    // Serialization round-trip
    // ------------------------------------------------------------------

    #[test]
    fn serializes_session_start_with_event_tag() {
        let event = Event::SessionStart {
            session_id: "abc".into(),
            cwd: "/x".into(),
            timestamp: 42,
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["event"], "session_start");
        assert_eq!(parsed["session_id"], "abc");
        assert_eq!(parsed["timestamp"], 42);
    }

    #[test]
    fn round_trips_post_tool_use_failure() {
        let original = Event::PostToolUseFailure {
            session_id: "x".into(),
            cwd: "/y".into(),
            timestamp: 99,
            tool_name: "Write".into(),
            error: "permission denied".into(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: Event = serde_json::from_str(&json).unwrap();
        assert_eq!(original, restored);
    }

    // ------------------------------------------------------------------
    // from_file
    // ------------------------------------------------------------------

    #[test]
    fn from_file_reads_valid_json_file() {
        let json = r#"{"event":"session_start","session_id":"file-s1","cwd":"/repo","timestamp":1234}"#;
        let file = write_temp(json);
        let event = Event::from_file(file.path()).unwrap();
        assert_eq!(
            event,
            Event::SessionStart {
                session_id: "file-s1".into(),
                cwd: "/repo".into(),
                timestamp: 1234,
            }
        );
    }

    #[test]
    fn from_file_returns_parse_error_for_invalid_json() {
        let file = write_temp("not valid json at all {{{");
        let err = Event::from_file(file.path()).unwrap_err();
        assert!(
            matches!(err, EventError::Parse(_)),
            "expected Parse error, got: {err}"
        );
    }

    #[test]
    fn from_file_returns_parse_error_for_wrong_schema() {
        let file = write_temp(r#"{"event":"unknown_event","session_id":"x"}"#);
        let err = Event::from_file(file.path()).unwrap_err();
        assert!(
            matches!(err, EventError::Parse(_)),
            "expected Parse error for unknown variant, got: {err}"
        );
    }

    #[test]
    fn from_file_returns_io_error_for_missing_file() {
        let path = Path::new("/this/path/definitely/does/not/exist/event.json");
        let err = Event::from_file(path).unwrap_err();
        assert!(
            matches!(err, EventError::Io(_)),
            "expected Io error, got: {err}"
        );
    }

    // ------------------------------------------------------------------
    // Accessor methods
    // ------------------------------------------------------------------

    #[test]
    fn session_id_accessor_returns_correct_value() {
        let event = Event::Notification {
            session_id: "my-session".into(),
            cwd: "/x".into(),
            timestamp: 0,
            message: "hello".into(),
        };
        assert_eq!(event.session_id(), "my-session");
    }

    #[test]
    fn cwd_accessor_returns_correct_value() {
        let event = Event::Stop {
            session_id: "s".into(),
            cwd: "/workspace/project".into(),
            timestamp: 0,
        };
        assert_eq!(event.cwd(), "/workspace/project");
    }

    #[test]
    fn timestamp_accessor_returns_correct_value() {
        let event = Event::PreCompact {
            session_id: "s".into(),
            cwd: "/".into(),
            timestamp: 1_700_000_000,
        };
        assert_eq!(event.timestamp(), 1_700_000_000);
    }

    #[test]
    fn accessors_work_for_all_variants() {
        let variants: Vec<Event> = vec![
            Event::SessionStart {
                session_id: "id".into(),
                cwd: "/cwd".into(),
                timestamp: 1,
            },
            Event::SessionEnd {
                session_id: "id".into(),
                cwd: "/cwd".into(),
                timestamp: 1,
            },
            Event::PostToolUse {
                session_id: "id".into(),
                cwd: "/cwd".into(),
                timestamp: 1,
                tool_name: "t".into(),
            },
            Event::PostToolUseFailure {
                session_id: "id".into(),
                cwd: "/cwd".into(),
                timestamp: 1,
                tool_name: "t".into(),
                error: "e".into(),
            },
            Event::UserPromptSubmit {
                session_id: "id".into(),
                cwd: "/cwd".into(),
                timestamp: 1,
            },
            Event::Notification {
                session_id: "id".into(),
                cwd: "/cwd".into(),
                timestamp: 1,
                message: "m".into(),
            },
            Event::SubagentStart {
                session_id: "id".into(),
                cwd: "/cwd".into(),
                timestamp: 1,
                subagent_id: "sub".into(),
            },
            Event::SubagentEnd {
                session_id: "id".into(),
                cwd: "/cwd".into(),
                timestamp: 1,
                subagent_id: "sub".into(),
            },
            Event::PreCompact {
                session_id: "id".into(),
                cwd: "/cwd".into(),
                timestamp: 1,
            },
            Event::Stop {
                session_id: "id".into(),
                cwd: "/cwd".into(),
                timestamp: 1,
            },
        ];

        for event in &variants {
            assert_eq!(event.session_id(), "id", "session_id failed for {event:?}");
            assert_eq!(event.cwd(), "/cwd", "cwd failed for {event:?}");
            assert_eq!(event.timestamp(), 1, "timestamp failed for {event:?}");
        }
    }

    // ------------------------------------------------------------------
    // EventError Display
    // ------------------------------------------------------------------

    #[test]
    fn event_error_display_io_contains_context() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "no such file");
        let err = EventError::Io(io_err);
        let msg = err.to_string();
        assert!(msg.contains("I/O error"), "unexpected message: {msg}");
    }

    #[test]
    fn event_error_display_parse_contains_context() {
        let parse_err = serde_json::from_str::<serde_json::Value>("!!!").unwrap_err();
        let err = EventError::Parse(parse_err);
        let msg = err.to_string();
        assert!(msg.contains("parse event JSON"), "unexpected message: {msg}");
    }
}

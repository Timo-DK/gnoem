#!/bin/bash
# Gnoem hook script - writes event files for the Gnoem TUI visualizer
# Usage: Set GNOEM_EVENT_TYPE environment variable, pipe Claude hook data via stdin

set -euo pipefail

EVENTS_DIR="${GNOEM_EVENTS_DIR:-$HOME/.config/gnoem/events}"
mkdir -p "$EVENTS_DIR"

EVENT_TYPE="${GNOEM_EVENT_TYPE:-unknown}"
TIMESTAMP=$(date +%s%3N 2>/dev/null || date +%s)
SESSION_ID="${CLAUDE_SESSION_ID:-unknown}"
CWD="${CLAUDE_CWD:-$(pwd)}"

# Read stdin (Claude hook passes JSON data)
INPUT=$(cat)

# Extract tool_name from input if present
TOOL_NAME=$(echo "$INPUT" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('tool_name',''))" 2>/dev/null || echo "")
ERROR_MSG=$(echo "$INPUT" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('error',''))" 2>/dev/null || echo "")
SUBAGENT_ID=$(echo "$INPUT" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('subagent_id',''))" 2>/dev/null || echo "")
MESSAGE=$(echo "$INPUT" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('message',''))" 2>/dev/null || echo "")

# Build JSON event
EVENT_FILE="$EVENTS_DIR/${TIMESTAMP}-${EVENT_TYPE}-${SESSION_ID}.json"

# Build the JSON based on event type
case "$EVENT_TYPE" in
    session_start|session_end|user_prompt_submit|pre_compact|stop)
        cat > "$EVENT_FILE" <<EVENTJSON
{"event":"$EVENT_TYPE","session_id":"$SESSION_ID","cwd":"$CWD","timestamp":$TIMESTAMP}
EVENTJSON
        ;;
    post_tool_use)
        cat > "$EVENT_FILE" <<EVENTJSON
{"event":"$EVENT_TYPE","session_id":"$SESSION_ID","cwd":"$CWD","timestamp":$TIMESTAMP,"tool_name":"$TOOL_NAME"}
EVENTJSON
        ;;
    post_tool_use_failure)
        cat > "$EVENT_FILE" <<EVENTJSON
{"event":"$EVENT_TYPE","session_id":"$SESSION_ID","cwd":"$CWD","timestamp":$TIMESTAMP,"tool_name":"$TOOL_NAME","error":"$ERROR_MSG"}
EVENTJSON
        ;;
    subagent_start|subagent_end)
        cat > "$EVENT_FILE" <<EVENTJSON
{"event":"$EVENT_TYPE","session_id":"$SESSION_ID","cwd":"$CWD","timestamp":$TIMESTAMP,"subagent_id":"$SUBAGENT_ID"}
EVENTJSON
        ;;
    notification)
        cat > "$EVENT_FILE" <<EVENTJSON
{"event":"$EVENT_TYPE","session_id":"$SESSION_ID","cwd":"$CWD","timestamp":$TIMESTAMP,"message":"$MESSAGE"}
EVENTJSON
        ;;
esac

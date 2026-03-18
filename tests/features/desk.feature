Feature: Desk environment
  The desk reflects session activity with interactive decorations

  Scenario: Papers accumulate with tool use
    Given a gnoem is in the office with session id "s1"
    When a tool use event arrives for session "s1" with tool "Bash"
    And a tool use event arrives for session "s1" with tool "Read"
    And a tool use event arrives for session "s1" with tool "Write"
    Then the desk for "s1" should have 3 papers

  Scenario: Compaction reduces paper stack
    Given a gnoem is in the office with session id "s1"
    And a tool use event arrives for session "s1" with tool "Bash"
    And a tool use event arrives for session "s1" with tool "Read"
    And a tool use event arrives for session "s1" with tool "Write"
    And a tool use event arrives for session "s1" with tool "Edit"
    And a tool use event arrives for session "s1" with tool "Grep"
    When a pre-compact event arrives for session "s1"
    Then the desk for "s1" should have 2 papers

  Scenario: Monitor shows error on tool failure
    Given a gnoem is in the office with session id "s1"
    When a tool failure event arrives for session "s1" with tool "Read"
    Then the desk for "s1" should show a monitor error

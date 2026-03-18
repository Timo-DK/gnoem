Feature: Gnoem lifecycle
  Gnoems enter and leave the office based on Claude session events

  Scenario: Gnoem starts typing after tool use
    Given a gnoem is in the office with session id "s1"
    When a tool use event arrives for session "s1" with tool "Bash"
    Then the gnoem "s1" should be in the "Typing" state
    And the last activity for "s1" should be "used Bash"

  Scenario: Gnoem scratches head on tool failure
    Given a gnoem is in the office with session id "s1"
    When a tool failure event arrives for session "s1" with tool "Read"
    Then the gnoem "s1" should be in the "HeadScratching" state

  Scenario: Gnoem jumps on notification
    Given a gnoem is in the office with session id "s1"
    When a notification event arrives for session "s1"
    Then the gnoem "s1" should be in the "Jumping" state

  Scenario: Gnoem receives letter on user prompt
    Given a gnoem is in the office with session id "s1"
    When a user prompt event arrives for session "s1"
    Then the gnoem "s1" should be in the "ReceivingLetter" state
    And the desk for "s1" should have an envelope

  Scenario: Gnoem packs up when session ends
    Given a gnoem is in the office with session id "s1"
    When a session end event arrives for session "s1"
    Then the gnoem "s1" should be in the "PackingUp" state

  Scenario: Subagent creates a mini gnoem
    Given a gnoem is in the office with session id "s1"
    When a subagent start event arrives for session "s1" with subagent "sub-1"
    Then the gnoem "s1" should have 1 mini gnoem

  Scenario: Subagent ending removes mini gnoem
    Given a gnoem is in the office with session id "s1"
    And a subagent start event arrives for session "s1" with subagent "sub-1"
    When a subagent end event arrives for session "s1" with subagent "sub-1"
    Then the gnoem "s1" should have 0 mini gnoems

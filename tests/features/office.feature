Feature: Office management
  The office tracks all active Gnoem sessions

  Scenario: Empty office at startup
    Given the office is empty
    Then there should be 0 active gnoems

  Scenario: A new session creates a gnoem
    Given the office is empty
    When a session starts for "/home/user/project" with id "session-1"
    Then there should be 1 active gnoem
    And the gnoem "session-1" should be in the "Entering" state

  Scenario: Known project reuses gnoem identity
    Given the office has a registered gnoem named "Grumbold" for "/home/user/project"
    When a session starts for "/home/user/project" with id "session-1"
    Then the gnoem "session-1" should be named "Grumbold"

  Scenario: Multiple concurrent sessions
    Given the office is empty
    When a session starts for "/home/user/project-a" with id "s1"
    And a session starts for "/home/user/project-b" with id "s2"
    Then there should be 2 active gnoems

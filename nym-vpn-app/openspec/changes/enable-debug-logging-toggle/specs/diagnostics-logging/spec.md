## MODIFIED Requirements

### Requirement: Application Logging Setup

The backend SHALL configure the tracing subscriber with an environment filter (default Info) and rotate the previous log file on startup. File logging SHALL NOT be controlled by a CLI argument or an environment variable; instead it SHALL be enabled solely when the persisted debug-logging preference is on. When file logging is enabled, the previous log file SHALL be rotated to an `.old` file and a new log file SHALL be created in the log directory. The file layer SHALL be installed through a reloadable layer so it can be toggled at runtime (see "Runtime Debug Logging Toggle") without re-initializing the subscriber.

#### Scenario: File logging enabled at startup by preference

- **GIVEN** the persisted debug-logging preference is on
- **WHEN** logging is set up on app start
- **THEN** the previous log file is rotated to an `.old` file and a new log file is created in the log directory

#### Scenario: File logging disabled at startup

- **GIVEN** the persisted debug-logging preference is off
- **WHEN** logging is set up on app start
- **THEN** no log file is created and no `app.log` is written to the log directory

## ADDED Requirements

### Requirement: Runtime Debug Logging Toggle

The backend SHALL allow app file logging to be enabled and disabled at runtime, and SHALL persist the user's choice in the app config file so it applies on subsequent starts. Enabling SHALL begin writing app logs to a file immediately; disabling SHALL stop writing immediately and leave no active log file. Toggling SHALL NOT restart the application, its daemon gRPC connection, or the VPN tunnel. The backend SHALL expose a command to set the preference and a command to read the current preference for the UI.

#### Scenario: Enable debug logging at runtime

- **GIVEN** app file logging is currently disabled
- **WHEN** the set-debug-logging command is invoked with enabled = true
- **THEN** the preference is persisted as enabled, a new log file is created, subsequent app log events are written to it, and no restart of the app or tunnel occurs

#### Scenario: Disable debug logging at runtime

- **GIVEN** app file logging is currently enabled
- **WHEN** the set-debug-logging command is invoked with enabled = false
- **THEN** the preference is persisted as disabled, the file writer is flushed and stopped, subsequent app log events are no longer written to a file, and no restart of the app or tunnel occurs

#### Scenario: Read debug-logging preference for the UI

- **GIVEN** the app is running
- **WHEN** the debug-logging-enabled command is invoked
- **THEN** it returns the current effective state of app file logging

#### Scenario: Toggle exposed in settings

- **GIVEN** the user is on the Settings → Data, privacy & logs screen
- **WHEN** the screen renders
- **THEN** an "Enable debug logging" switch reflects the current preference and toggling it invokes the set-debug-logging command

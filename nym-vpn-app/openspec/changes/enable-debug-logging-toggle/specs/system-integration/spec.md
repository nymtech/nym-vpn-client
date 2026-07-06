## MODIFIED Requirements

### Requirement: CLI Argument Parsing

The application SHALL parse CLI arguments including build-info, log-level, nosplash, clean-local-files, and trailing deep-link arguments, and SHALL act on them before launching the UI where appropriate. The application SHALL NOT provide a `log-file` CLI argument; app file logging is controlled solely by the persisted debug-logging preference instead (see the diagnostics-logging capability).

#### Scenario: Print build info and exit

- **GIVEN** the app is started with the build-info flag
- **WHEN** arguments are parsed
- **THEN** build metadata is printed instead of launching the UI

#### Scenario: Clean local files and exit

- **GIVEN** the app is started with the clean-local-files flag
- **WHEN** arguments are parsed
- **THEN** the app deletes all local files and exits without starting

#### Scenario: No log-file argument

- **GIVEN** the app is started with a `--log-file` argument
- **WHEN** arguments are parsed
- **THEN** argument parsing fails with an unknown-argument error because the flag no longer exists

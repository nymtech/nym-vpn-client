## ADDED Requirements

### Requirement: Slider Reports The Committed Value

The shared `Slider` control SHALL report the value the user actually committed to its
`onValueCommitted` callback, including on synchronous commits (keyboard arrow keys and
click-to-position), where the change and commit occur in the same event tick. It SHALL NOT
report a stale value from a prior render.

#### Scenario: Keyboard commit forwards the new value
- **GIVEN** a `Slider` bound to `onValueCommitted`
- **WHEN** the user changes the value with the keyboard (which fires change and commit
  synchronously)
- **THEN** `onValueCommitted` is called with the new value, not the value from before the
  keypress

#### Scenario: Drag commit forwards the final value
- **GIVEN** a `Slider` bound to `onValueCommitted`
- **WHEN** the user drags the thumb and releases
- **THEN** `onValueCommitted` is called with the final dragged value

#### Scenario: UI scaler persists the committed font size
- **GIVEN** the Display settings UI scaler
- **WHEN** the user commits a new font size
- **THEN** the committed size is dispatched to the store and persisted (not reverted to the
  previous size)

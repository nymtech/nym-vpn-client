## Why

The shared `Slider` component (`src/ui/Slider.tsx`) reports a **stale value** to its
`onValueCommitted` callback. The commit handler ignores the value base-ui hands it and
instead passes the closed-over `internalValue` state:

```tsx
onValueCommitted={() => onValueCommitted?.(internalValue)}
```

On synchronous commits — keyboard arrow keys and click-to-position — base-ui fires
`onValueChange` and `onValueCommitted` in the same event tick, before React re-renders, so
`internalValue` is still the previous value. The committed callback therefore receives the
value from *before* the interaction.

This surfaces as a user-visible bug in the UI Scaler (`Display` settings): committing a new
font size with the keyboard persists the *old* size to the store and `document` font-size,
snapping the setting back. Every `onValueCommitted` consumer is affected — the UI Scaler and
the mixnet-tuning sliders (`MixingDelayCard`, `ContinuousTrafficCard`).

The existing tests for these components had to *work around* the bug (asserting the displayed
value rather than the persisted value), which masks it.

## What Changes

- Fix `src/ui/Slider.tsx` so `onValueCommitted` forwards the value base-ui provides in its
  callback, matching how `onValueChange` already forwards `val`:
  ```tsx
  onValueCommitted={(val) => onValueCommitted?.(val)}
  ```
- Update the tests that worked around the bug to assert the **correct** committed value:
  - `src/ui/Slider.test.tsx` — assert `onValueCommitted` receives the committed value.
  - `src/screens/settings/appearance/display/UiScaler.test.tsx` — assert the committed font
    size reaches the store / `document` (remove the displayed-value workaround).
  - mixnet-tuning card tests (`MixingDelayCard`, `ContinuousTrafficCard`) — assert commit
    persists the committed value if they currently work around it.
- Add a regression test proving a keyboard/synchronous commit forwards the fresh value.

## Capabilities

### New Capabilities
<!-- None -->

### Modified Capabilities
- `frontend-ui`: Adds a requirement specifying that the shared slider control reports the
  final committed value (not a stale one) to `onValueCommitted`, including on synchronous
  keyboard/click commits.

## Impact

- **Source:** `src/ui/Slider.tsx` (one-line behavior fix in the `onValueCommitted` wiring).
- **Tests:** `src/ui/Slider.test.tsx`, `src/screens/settings/appearance/display/UiScaler.test.tsx`,
  and the mixnet-tuning card tests updated to assert correct behavior + a new regression case.
- **Behavior:** UI Scaler and mixnet-tuning sliders now persist the value the user actually
  committed. No API change to `Slider`'s props — callers already pass an `onValueCommitted`
  that expects the committed value.
- **Risk:** Low; isolated to one component. All existing Slider consumers benefit.

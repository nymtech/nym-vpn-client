# fix/disconnected-state-account-preflight

## Summary

This branch adds an internal account preflight state before entering `ConnectingState`.

The key invariant is: the state machine must not call `ConnectingState::enter()` until the
account controller is ready to connect (`ReadyToConnect`, `Decentralised`, or `UpgradeMode`).
`ConnectingState::enter()` raises the VPN API firewall immediately, so inactive/no-plan
accounts must be rejected or waited on before that point.

Covered callsites:

- `DisconnectedState` user connect command
- `ErrorState` user connect command
- `OfflineState` auto-reconnect when connectivity returns
- reconnect paths through `ConnectedState` tunnel-down recovery and `DisconnectingState`

The preflight state exposes `Connecting/AwaitingAccountReadiness` to the UI while it waits,
but it first resets to unrestricted networking and does not apply the tunnel firewall. If
the account becomes ready, it transitions to the normal `ConnectingState`. If the account
reaches a terminal non-connectable state (`LoggedOut`, `Offline`, or `Error(_)`), it returns
to `DisconnectedState`; the existing account-state UI then shows the specific account
message and, for `no-subscription`, routes the main button to pricing.

## Tests run

```text
cargo fmt --manifest-path nym-vpn-core\Cargo.toml --package nym-vpn-lib
cargo check --manifest-path nym-vpn-core\Cargo.toml -p nym-vpn-lib
```

Result: `cargo check` passed.

`cargo fmt` prints the existing rustfmt warning:

```text
Warning: can't set `imports_granularity = Crate`, unstable features are only available in nightly channel.
```

## Manual repro still needed

Use the machine/account state from the original report:

1. Start with an account that has no active subscription.
2. Click Connect while the account controller is still `Syncing`/settling.
3. Expected:
   - UI may show "Setting up your account (2/6)" briefly.
   - Internet must remain available; no "Allowing endpoints: none" blackout.
   - Once account state resolves to no subscription, app returns to disconnected/no-plan UI.
4. Also test reconnect after network offline/online while account is not ready.

## Open questions

- If product wants no visible "2/6" state for no-plan accounts, add a public non-firewall
  account-preflight state and UI copy. I avoided that broader binding/UI change here.
- GitHub issue/PR creation is pending repository access from this environment.

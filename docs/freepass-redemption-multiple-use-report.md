# Report: free-pass voucher code can be redeemed multiple times

**Date:** 2026-06-25
**Reporter:** Android client (scan-QR free-pass feature, debug build `net.nymtech.nymvpn.debug`)
**Environment:** Production API (`https://nymvpn.com/api`).
**Severity:** High — a single-use free-pass voucher can be redeemed repeatedly across distinct accounts on production.
**Status:** Client side verified correct; server-side single-use enforcement is missing. Needs backend fix.

## Summary

When applying a free-pass voucher code from the Android client, the API returns **success every time**, even for a code that should already be consumed, and even across **multiple distinct freshly-created accounts**. No "already redeemed" / "voucher consumed" rejection is ever returned. The code under test (`hkB4sgMgfU8`) was provided as an *already-redeemed* code, yet redemption succeeds.

## What the client does (verified correct)

The client calls the same endpoint as the web dashboard:

```
POST https://nymvpn.com/api/public/v1/account/{account_id}/freepass
Authorization: Bearer <account JWT, secp256k1>
Content-Type: application/json

{"code":"<voucher code>"}
```

- Endpoint construction: `nym-vpn-api-client/src/client.rs::apply_freepass` →
  `post_authorized([PUBLIC, V1, ACCOUNT, account.id(), FREEPASS], {code}, account)`.
  This matches the dashboard curl (`/api/public/v1/account/{id}/freepass`, body `{"code":...}`, account Bearer JWT).
- Response handling: `post_authorized`/`post_query` deserialize the body into `NymVpnSubscription`; any non-2xx becomes an `HttpClientError`. So a client-side "success" means the **server returned HTTP 2xx with a valid subscription JSON** — not a no-op.
- Proof the client surfaces real server errors: in the same flow the `POST /account/android` (account registration) call returns a real `403 Forbidden` and is propagated correctly. So the success on `/freepass` is a genuine 2xx from the server, not swallowed error handling.
- After a successful apply, the account summary reflects `kind: Freepass`, i.e. the server granted the subscription.

## Observed behavior

Per-run client log (representative; tag `ui-generate-account-vm`):

```
CreateAccountSuccess (freepass)          # brand-new account/mnemonic generated locally
registerAccount requested                # POST /account/android
... 403 "Account already exists" x3      # see caveat below
Account already registered — continuing
applyFreepass requested                  # POST /account/{id}/freepass  {"code":"hkB4sgMgfU8"}
ApplyFreepass took ~540 ms               # real network round-trip
ApplyFreepassSuccess                     # HTTP 2xx + NymVpnSubscription returned
kind: Freepass                           # account summary now shows a free-pass subscription
```

This sequence succeeded on **repeated submissions of the same code**, each starting from a fresh `createAccount` (local data cleared between runs via `pm clear`). Zero `ApplyFreepassFailed` were recorded across runs.

> **Verified (2026-06-25):** the repeated redemptions used **distinct account IDs** (distinct JWT `sub`). Captured via a diagnostic log of `getAccountId()` immediately before each apply:
> - Run 1: `accountId=n1dhm9389ugdv6ss8rksy26l97pl4ds9vt7f7juq`, `code=hkB4sgMgfU8` → `ApplyFreepassSuccess`
> - Run 2: `accountId=n12mwjjs5w502yf47dvjyqg2tj3q795f7uxadl2q`, `code=hkB4sgMgfU8` → `ApplyFreepassSuccess`
>
> So `createAccount` produces fresh identities, and the **same already-redeemed voucher was accepted by two different accounts**. (Each request also carries a freshly-minted account JWT whose `sub` is the account address above.) This is the core finding: the voucher is not enforced as single-use across accounts.

> **Verified again (2026-06-26)** with the full account-summary captured before/after each apply (code `5CPswQADDGD`). This pins down the exact behavior — **idempotent per-account, but reusable across accounts**:
>
> **Account 1** (`n1l4zpkppnrjvqthswrz52mxfjmcg708jfv6xvhr`):
> - 1st apply → success; after sync the summary shows `subscription = ACTIVE NymVpnSubscription(id=tb751m1racdvr0u, kind=Freepass, validUntil=1785062860, isRecurring=false)`, `trafficLimitGb=25000`.
> - 2nd apply of the **same code on the same account** → `RedeemVoucherSuccess` (HTTP 200), but the subscription is **unchanged**: identical `id=tb751m1racdvr0u`, same `validUntil`, same traffic limit. ⇒ a re-apply on an account that already holds the pass is a **200 no-op** — "success but nothing applied".
>
> **Account 2** (`n1cjr0x8klt5vuujl0q8wm5kyz6gg3qxuzvmkd55`, after logging out account 1 → `REMOVING ALL ACCOUNT AND DEVICE DATA`):
> - Same code applied → success → **VPN connected** (`New tunnel state: Connected wg to 172.245.232.254`). ⇒ the same voucher grants a working pass to a **second, distinct account**.
>
> Net: single-use is enforced **per account** (re-applying does nothing) but **not globally per code** (each new account can redeem it and connect). Evidence log: `freepass-same-vs-new-account-evidence.log`.

## Expected behavior

A free-pass voucher code should be single-use:
- A second redemption of the same code (by the same or a different account) should fail, e.g. `409`/`403` with a `messageId` like `...freepass...already-redeemed` / `...voucher-consumed`.
- The Android client already has UI for this: an "already redeemed" error dialog, keyed off the API error `message`/`message_id`. It has simply never been triggered because the server keeps returning success.

## Questions for the backend team

1. Does `POST /account/{id}/freepass` enforce single-use on the voucher code? Is the voucher marked consumed atomically on first redemption?
2. Is single-use scoped per-account or globally per-code? **Confirmed:** the same code was accepted by two distinct account IDs (`n1dhm93…` and `n12mwjj…`) — so there is no global per-code enforcement.
3. What is the exact error shape (status + `messageId`) returned for an already-redeemed code? The client needs this to map to the "already redeemed" UI state (`GeneratingViewModel.classifyFreepassError`).

## Secondary anomalies observed on production (same flow)

These were seen against the **production** API during the same test and are worth a look while investigating:

- `POST /account/android` (account registration) returns the **same `codeReferenceId`** on every request, across days and distinct accounts:
  `"codeReferenceId":"m7n8o9p0-q1r2-s3t4-u5v6-w7x8y9z0a1b2"`. A per-request reference is expected to be unique — a fixed value suggests a hardcoded/placeholder response path on this endpoint.
- Every registration of a **freshly generated** account address returns `403 "Account already exists"` (`messageId: ...create-account.access_denied.account-already-exists`). A brand-new random account address should not already exist. The client now tolerates this (treats "already exists" as success so the free-pass can still be applied), but the server behavior itself looks wrong and may be related to the missing single-use enforcement.

## Reproduction

1. Install the debug client; ensure logged out (`adb shell pm clear net.nymtech.nymvpn.debug`).
2. Welcome → "Scan QR code" → enter code `hkB4sgMgfU8` (already-redeemed) via manual entry.
3. Observe success and landing on the connect screen.
4. Repeat from step 1 (fresh account) with the same code — succeeds again.
5. Client log: `adb logcat -s ui-generate-account-vm` shows repeated `ApplyFreepassSuccess`, no failures.

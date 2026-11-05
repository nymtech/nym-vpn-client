# QR pairing format

Pairing signs a new device in to an existing account. The signed-in device shows a QR code; the new device scans it.
Both apps call the VPN core and share the rules below so that a scanned code is judged the same way on Android and iOS.

## What the QR contains

The text of `QrUrl.url` returned by the core's `startPairing()`:

```
https://nym.com/qr-pairing?pubkey=<key>
```

`<key>` is the showing device's x25519 public key, Base58 encoded (Bitcoin alphabet). The scanning device passes it,
unchanged, to the core's `scannedKey(pubKey)`.

## Accepting scanned text

Accept only when **all** of these hold; otherwise show "not a pairing code" and do not call the core.

1. At most 512 characters, with no surrounding whitespace.
2. Scheme is exactly `https` (lowercase).
3. Host is exactly `nym.com`: no subdomain, no userinfo, no port.
4. Path is exactly `/qr-pairing` (no trailing slash).
5. The query is exactly `pubkey=<key>`: no other or repeated parameters, no percent-encoding.
6. No fragment.
7. `<key>` is 32–44 characters from `[1-9A-HJ-NP-Za-km-z]`.

These are shape checks only. The key itself is verified by the core; a key the core rejects is shown as an invalid
code as well.

## Test vectors

`app/src/test/resources/pairing/qr-pairing-vectors.json` lists inputs and the expected key (`null` = reject). Android
runs them in `PairingQrCodeTest`; iOS should run the same file.

## Showing the code

- Draw it black on white with a quiet zone, error correction M.
- Block screenshots and screen recording while it's visible.
- The code isn't secret on its own: pairing completes only after both screens show the same PIN, the user confirms it
  on the signed-in device, and passes the device's own lock screen check.
- The core reports expiry with a `TimedOut` pairing event. Apps count down from 5 minutes as a hint only.
- Never log the URL, the key or the PIN.

# Detecting captive-portal Wi-Fi — proposal

**For:** Head of Product
**Status:** Proposal, awaiting decision
**Engineering effort:** ~1.5 days
**Backend infrastructure needed:** small (one HTTP endpoint), optional

---

## Decision asked

Approve building **captive-portal detection** into nym-vpn-app, so users on hotel / café / airport Wi-Fi that requires a browser sign-in get a clear "log in to this network first" message instead of a generic connection failure. Three sub-decisions inside that — see end of doc.

---

## The problem

A "captive portal" is the sign-in page hotels, cafés, airports, hospitals, university campuses, conferences, and most public networks show before letting your device reach the internet. Until you click through that browser page, **nothing else on the network works** — no email, no app traffic, no VPN.

What a user on a captive-portal network experiences with nym-vpn today:

1. Connect to "Hotel Wi-Fi" or "Aeroport_Free".
2. Open nym-vpn, press **Connect**.
3. App spins for ~30 seconds, then shows a generic "failed to connect" toast.
4. User retries. Same result.
5. User assumes nym-vpn is broken, opens a support ticket or uninstalls.

The actual problem is that they never opened a browser to log in to the Wi-Fi. nym-vpn has no way to tell them that. The daemon log shows `Connection refused` on every outbound connection — including to nym's own servers — because the captive portal is silently swallowing all their packets.

We have at least one confirmed instance this exact pattern produced a support investigation. It's also the dominant failure mode for travel-oriented VPN users.

## Why now

We just shipped detection for "another VPN already active" (different problem, same shape — user gets a clear warning instead of a mysterious failure). Captive-portal is the next-most-common silent-fail scenario for users in transit. The engineering pattern is the same, so the second feature is much cheaper than the first.

## How common is captive portal Wi-Fi

Practically every public network sold-by-the-hour or requiring T&C acceptance:
- **Hotels** (essentially universal worldwide)
- **Airports, airline lounges, in-flight Wi-Fi**
- **Train stations, train Wi-Fi** (TGV, ICE, Eurostar all use them)
- **Cafés** (Starbucks, Costa, McDonald's, most independents)
- **Hospitals, gyms, libraries**
- **Universities and conferences** (often with SAML behind the portal)
- **Some hotspots from your own mobile carrier** when roaming

A VPN user who travels at all hits one of these within their first week of using the app. A VPN user who works remotely from cafés hits one multiple times a day.

## What everyone else does

Captive portals are a fully solved problem at the platform level. Every major actor detects them:

| Platform | Detects portals? | How they communicate it |
|---|---|---|
| **Firefox** | yes | Yellow banner in URL bar with "Open Network Login Page" link |
| **Chrome / Edge** | yes | Same — a dedicated portal sign-in tab |
| **Safari** | yes | macOS opens a stripped-down sign-in browser automatically |
| **Android** | yes | System notification "Sign in to network" |
| **iOS** | yes | Pop-up sign-in sheet on Wi-Fi connect |
| **macOS** | yes | Same as iOS — system sheet |
| **Windows** | yes | NCSI shows a globe icon in tray + system toast |
| **NetworkManager (Linux)** | yes | Banner in GNOME/KDE, or an icon in tray |
| **Mullvad, NordVPN, ExpressVPN, Surfshark, ProtonVPN** | yes — all of them | All show a dedicated "log in to the network first" UI |
| **nym-vpn** | **no** | Generic connection failure |

We're the outlier. Every VPN competitor we benchmark against has this. Every browser the user might be using already has it.

## Proposed user experience

Two surfaces, both gated on platform-level detection (free) with an HTTP probe as fallback.

### Surface 1 — passive banner on Home

When the app is open and disconnected, and a captive portal is detected on the current network:

```
┌──────────────────────────────────────────────────────────────┐
│   Hotel-Wifi requires sign-in                                │
│   Open the sign-in page in your browser, then come back.     │
│   [ Open sign-in page ]                                      │
└──────────────────────────────────────────────────────────────┘

           [ Connect ]   ← still tappable; we don't lock the user out
```

- **Always non-blocking.** The Connect button stays alive — some captive portals are sloppy about what they block (and the user might be on a corporate-VPN-style portal that *does* let our gateway through). "Try anyway" is the universal escape hatch.
- **"Open sign-in page"** — best-effort one-tap shortcut that opens the user's default browser at the portal's sign-in URL (we get it from the HTTP redirect the portal serves). Where the portal doesn't redirect cleanly, opens the user's home page so the portal can intercept.
- Banner clears automatically when the OS-level detector says "online" again.

### Surface 2 — pre-connect dialog

If the user presses **Connect** while a portal is detected:

```
┌────── Network requires sign-in ────────────────────┐
│ "Hotel-Wifi" needs you to sign in via your browser │
│ before any apps — including NymVPN — can reach the │
│ internet.                                          │
│                                                    │
│   [ Open sign-in page ]                            │
│   [ Try anyway ]                                   │
│   [ Close ]                                        │
└────────────────────────────────────────────────────┘
```

Same shape as the "another VPN active" dialog we just shipped. If the user picks "Try anyway", we proceed with the normal connect and let the failure path do its thing (matching today's behaviour for forced edge cases).

### What the user sees over time

- **First visit to a hotel:** opens nym-vpn → sees banner immediately → taps "Open sign-in page" → signs in to Wi-Fi in browser → returns to nym-vpn → banner is gone → connects normally.
- **Five seconds of usability work.** No support ticket, no confusion.

## Privacy posture — the only nuance worth thinking about

To detect a captive portal you have to send a packet *before* the VPN is up. That's intrinsic to the problem — the portal is what you're trying to detect.

The minimum-viable probe is one HTTP request (yes, HTTP, not HTTPS — that's how portals reveal themselves) to a known URL, comparing the response. There are three options for which URL:

| Option | What it costs us | What it costs the user |
|---|---|---|
| Use the OS's built-in detection (NetworkManager / NCSI) | nothing — already there | nothing — the OS already does this |
| **Use a Nym-hosted endpoint** (e.g. `http://captive.nymvpn.com/check`) (**recommended**) | one cheap nginx container | one tiny request, to Nym's own infra |
| Use Firefox's well-known endpoint (`detectportal.firefox.com`) | nothing | tiny request to Mozilla |
| Use Google/Apple/Microsoft's endpoint | nothing | every connection probe goes to Google/Apple/Microsoft |

**Recommended posture:**

1. **Always try the OS API first** — on most user systems (any modern Linux with NetworkManager, all of Windows 10+, all macOS), the OS has already done the probe. We just ask. Zero traffic of our own, zero privacy cost.
2. **HTTP probe only as fallback** — when the OS doesn't expose the answer, hit a **Nym-controlled endpoint** if SRE can stand one up (~1 hour of work), or Mozilla's as the interim before that exists. Avoid Google.
3. **The probe is opt-out** — surfaced in Settings → Privacy as "Detect networks that need browser sign-in" with a description. Default ON because the usability gain dwarfs the privacy cost of one HTTP request to a Mozilla endpoint.

The branding here matters: this is something we do **for** the user (so they know to log in) not **to** the user (we're not phoning home, we're testing whether the *network* is restricting them).

## Scope and cost

| Work | Effort | Owner |
|---|---|---|
| Linux: query NetworkManager via D-Bus (zero-cost OS path) | 0.25 day | App dev |
| Windows: query NCSI via WinRT | 0.25 day | App dev |
| Fallback HTTP probe + timeout handling | 0.25 day | App dev |
| Wire into connect command + frontend banner + dialog | 0.5 day | App dev |
| iOS app equivalent (separate Swift codebase) | 0.5–1 day | iOS dev (separate ticket) |
| **Optional:** stand up `captive.nymvpn.com` | ~1 hour | SRE |
| Translations (new strings ~10) | parallel | i18n |

Total for desktop: **~1.5 dev-days** to ship for Linux + Windows. iOS is a separate ticket on the Swift codebase.

## What this proposal explicitly does NOT cover

- **We do not embed a sign-in browser.** Apple does this on iOS for system-level reasons we can't replicate. Opening the user's default browser at the portal's URL is enough — and is what every desktop browser does.
- **We do not "punch through" the portal.** That's not technically possible; the portal is a network-level gate.
- **We do not extend this to handle "limited connectivity" or "metered network" states.** Those are different signals (NM reports them separately). Scope creep candidate — leave for later if useful.
- **We do not auto-retry connect after the portal is signed in.** The user pressing Connect again is one tap. Auto-reconnect could come later; not needed for v1.

## Decisions

1. **Should we ship this?** Yes/no. (Recommendation: yes — engineering is cheap, user gain is large.)
2. **Should SRE host a Nym-owned probe endpoint?** Yes/no. (Recommendation: yes, file the infra ticket. Mozilla endpoint is fine in the meantime.)
3. **Should we surface the "Detect captive portals" setting** in Settings → Privacy, or leave it implicit? (Recommendation: surface it, default ON. Privacy-minded users will appreciate the transparency.)

## Engineering reference

Technical implementation note for the dev team lives in `docs/captive-portal-implementation.md` (to be written after product approval). It will mirror the shape of the recently shipped "another VPN active" detection.

---

*Prepared in response to a confirmed user-side failure on a captive-portal Wi-Fi network where the daemon log showed `Connection refused` across all outbound endpoints (gateways and Nym CDN alike). The investigation that led here lives in `docs/fast-mode-connect-investigation.md`.*

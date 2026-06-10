![NymVPN](https://github.com/nymtech/Nym-brand-assets/blob/main/NymVPN%20(cover).png)

NymVPN is the most private way to be online. Open-source, cross-platform VPN client written in Rust. NymVPN routes traffic through [Nym](https://nym.com)'s decentralized mixnet for metadata-level anonymity, or over multi-hop AmneziaWG (WireGuard) for low-latency, censorship-resistant tunneling. Unlike conventional VPNs, no single node can correlate who you are with what you do.

# What is NymVPN?

A decentralized VPN (dVPN) that protects **traffic patterns, not just packet contents**. Conventional VPNs replace your ISP with a single trusted operator who can see both your IP and your destinations. NymVPN removes that trusted party:

- **Mixnet routing** — 5-hop onion-encrypted routing through Nym's [Noise Generating Mixnet](https://nym.com/mixnet). Anonymized packets are sent through randomized routes, mixed other packet flows and cover traffic, and timing obfuscated to defeat traffic-analysis attacks.
- **Multi-hop WireGuard** — 2-hop AmneziaWG tunneling for performance-sensitive use, with no single node seeing both your origin IP and that of your destination on the web.
- **Zero-knowledge credentials ([zk-nyms](https://nym.com/zk-nyms))** — authentication and payment are cryptographically unlinkable from network usage. No email or account identity is required to connect.
- **No single point of failure** — operated by independent node operators, with no central server to compromise, subpoena, or surveil.

Tech details and cryptography are documented in the [NymVPN Litepaper](https://nym.com/nymvpn-litepaper) and the [Nym Whitepaper](https://nym.com/nym-whitepaper.pdf).

# NymVPN Downloads

<a href="https://github.com/nymtech/nym-vpn-client/releases?q=android&expanded=true"><img src=".github/assets/apk-download-badge-1745835177551.png" height="54" alt="Android (GitHub releases)"></a>
<a href="https://f-droid.org/packages/net.nymtech.nymvpn/"><img src=".github/assets/fdroid-badge.png" height="54" alt="F-Droid"></a>
<a href="https://flathub.org/apps/net.nymtech.NymVPN"><img src=".github/assets/flathub-store.svg" height="54" alt="Flathub"></a>
<a href="https://play.google.com/store/apps/details?id=net.nymtech.nymvpn"><img src=".github/assets/play-badge.png" height="54" alt="Google Play"></a>
<a href="https://apps.apple.com/app/id6471254143"><img src=".github/assets/app-store-badge.svg" height="54" alt="App Store"></a>
<a href="https://testflight.apple.com/join/0vmRJNrL"><img src=".github/assets/testflight-badge.svg" height="54" alt="TestFlight"></a>
<a href="https://github.com/nymtech/nym-vpn-client/releases?q=linux&expanded=true"><img src=".github/assets/linux-badge.png" height="54" alt="Linux"></a>
<a href="https://github.com/nymtech/nym-vpn-client/releases?q=macos&expanded=true"><img src=".github/assets/macos-badge.png" height="54" alt="macOS"></a>
<a href="https://github.com/nymtech/nym-vpn-client/releases?q=windows&expanded=true"><img src=".github/assets/windows-badge.png" height="54" alt="Windows"></a>

# Two VPN modes

Two routing modes in a single client, selectable per connection.

**Mixnet mode — 5-hop Mixnet.** Routes traffic through Nym's [mixnet](https://nym.com/mixnet): five independently operated hops with Sphinx packet format, per-hop onion encryption, packet reordering, and cover traffic. Breaks the timing and volume correlations that deanonymize traditional onion and VPN traffic. For threat models where metadata exposure matters — crypto wallets, email, private messaging.

**Fast mode — 2-hop AmneziaWG.** A decentralized 2-hop tunnel on [AmneziaWG](https://github.com/amnezia-vpn/amneziawg-go), a censorship-resistant WireGuard fork. Lower latency for streaming and browsing, while still ensuring no single operator observes both endpoints.

```
┌─────────────────┐    ┌──────────────┐    ┌─────────────────┐
│                 │ -> │   Mixnet     │ -> │   Destination   │
│                 │    │  (5 hops)    │    │                 │
│    NymVPN App   │    └──────────────┘    └─────────────────┘
│    (Rust Core)  │    ┌──────────────┐    ┌─────────────────┐
│                 │ -> │  AmneziaWG   │ -> │   Destination   │
│                 │    │  (2 hops)    │    │                 │
└─────────────────┘    └──────────────┘    └─────────────────┘
```

# NymVPN Features

- Multi-hop by default — no node sees both your IP and your activity
- zk-nym credentials unlink payment/auth data from network usage; no email or account identity required
- No centralized logging — the multi-hop architecture makes meaningful traffic logs impractical to keep
- Multi-layer onion encryption
- Entry/exit node selection
- Built-in kill switch with data-leak prevention
- Custom DNS
- Split tunneling to include or exclude apps from the VPN tunnel
- Built-in ad blocking to stop ads, trackers, and malware
- Censorship resistance via AmneziaWG, Stealth connect, and QUIC to prevent VPN blocking

**Cryptographic stack:** Curve25519, AES, ChaCha20-Poly1305, BLAKE2/BLAKE3, Lioness wide-block cipher, Pointcheval-Sanders signatures, Pedersen commitments, NIZK proofs, BLS12-381, post-quantum key exchange.

**Independent audits:** JP Aumasson (2021), Oak Security (2022), Cryspen (2023–2024), Cure53 (2024). See [Nym Audits](https://nym.com/trust-center/independently-audited).

# Platforms & Stack

This monorepo contains all NymVPN client source, separate from the Nym network [monorepo](https://github.com/nymtech/nym).

| Component | Path | Stack |
|---|---|---|
| Core VPN engine | `nym-vpn-core` | Rust |
| Android | `nym-vpn-android` | Kotlin |
| iOS / macOS | `nym-vpn-apple` | SwiftUI |
| Linux / Windows (desktop) | `nym-vpn-app`, `nym-vpn-windows` | Tauri + TypeScript |
| WireGuard integration | `wireguard` | Rust / C |

# Contributing

Contributions welcome across the stack: Rust core (networking, crypto, protocols), mobile (Kotlin/SwiftUI), desktop (SwiftUI/Tauri), protocol research, and security review. See the [Contribution Guide](https://github.com/nymtech/nym-vpn-client/blob/develop/CONTRIBUTING.md), [Code of Conduct](https://github.com/nymtech/nym-vpn-client/blob/develop/CODE_OF_CONDUCT.md), and [Security Policy](https://github.com/nymtech/nym-vpn-client/blob/develop/SECURITY.md). Localization is crowdsourced via [Crowdin](https://crowdin.com/editor/nymvpn-apps).

# Resources

[Litepaper](https://nym.com/nymvpn-litepaper) · [Whitepaper](https://nym.com/nym-whitepaper.pdf) · [Roadmap](https://nym.com/blog/nym-roadmap-2026) · [Audits](https://nym.com/trust-center/independently-audited) · [Trust Center](https://nym.com/trust-center) · [Blog](https://nym.com/blog)

# Licensing & Acknowledgements

GPL-3.0. ©2018–2026 Nym Technologies SA ([contact@nymtech.net](mailto:contact@nymtech.net)). Built with [adblock-rust](https://github.com/brave/adblock-rust/), [Mullvad's open-source libraries](https://github.com/mullvad/mullvadvpn-app/) (local routing, wireguard-go wrapping), [AmneziaWG wg-go](https://github.com/amnezia-vpn/amneziawg-go), and [WireGuard](https://github.com/WireGuard).

# Community

[![Telegram](https://img.shields.io/badge/Telegram-26A5E4.svg?style=for-the-badge&logo=Telegram&logoColor=white)](https://nym.com/go/telegram)
[![Matrix](https://img.shields.io/badge/Matrix-000000.svg?style=for-the-badge&logo=Matrix&logoColor=white)](https://nym.com/go/matrix)
[![YouTube](https://img.shields.io/badge/YouTube-FF0000.svg?style=for-the-badge&logo=YouTube&logoColor=white)](https://nym.com/go/youtube)
[![Discord](https://img.shields.io/badge/Discord-5865F2.svg?style=for-the-badge&logo=Discord&logoColor=white)](https://nym.com/go/discord)
[![X](https://img.shields.io/badge/X-000000.svg?style=for-the-badge&logo=X&logoColor=white)](https://nym.com/go/x)

Hello and welcome to the NymVPN GitHub page! For more information about NymVPN and to request beta access credentials, visit https://nymvpn.com/en.

# NymVPN client

The NymVPN client is a VPN-like app built on [Nym's signature mixnet](https://nymtech.net), offering the following [features](https://nymvpn.com/en/features):
- Anonymous 5-hop mixnet mode
- Fast 2-hop WireGuard-based decentralized VPN mode
- Private credentials using zk-nyms (zero-knowledge proofs)

NymVPN is available on all major platforms incl. [Android](https://nymvpn.com/en/download/android), [iOS](https://nymvpn.com/en/download/ios), [Linux](https://nymvpn.com/en/download/linux), [macOS](https://nymvpn.com/en/download/macos) and [Windows](https://nymvpn.com/en/download/windows).

NymVPN relies on [Mullvad open source libraries](https://github.com/mullvad/mullvadvpn-app/) to handle setting up local routing and wrapping wireguard-go.

Visit [NymVPN's blog](https://nymvpn.com/en/blog) for the latest announcements and articles on privacy and security. Visit our [Help Center](https://support.nymvpn.com/hc/en-us) or contact our [Support team](https://support.nymvpn.com/hc/en-us/requests/new) with any questions about NymVPN.



## Core

The `nym-vpn-core` Rust workspace contains among other things the daemon (`nym-vpnd`)  and the CLI client (`nym-vpnc`).

[nym-vpnd](nym-vpn-core/crates/nym-vpnd)
[nym-vpnc](nym-vpn-core/crates/nym-vpnc)


## GUI clients

Interacting either with `nym-vpnd` or directly to `nym-vpn-lib` using FFI are a number of GUI clients.

[nym-vpn-android](nym-vpn-android/README.md)\
[nym-vpn-apple](nym-vpn-apple/README.md)\
[nym-vpn-desktop](nym-vpn-x/README.md)


## Nym's mixnet overview


```
                      ┌─►mix──┐  mix     mix
                      │       │
            entry     │       │                   exit
client ───► gateway ──┘  mix  │  mix  ┌─►mix ───► gateway ───► internet
                              │       │
                              │       │
                         mix  └─►mix──┘  mix
```

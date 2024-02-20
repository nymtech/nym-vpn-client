# Nym VPN client

VPN client that uses the [Nym mixnet](https://nymtech.net). 

- 5-hops (incl entry and exit gateways), 
- optional: 2-hop straight from entry to exit gateway.
- optional: tunnel the connection to the entry gateway through wireguard

Makes use of the fantastic [Mullvad open source libraries](https://github.com/mullvad/mullvadvpn-app/) to handle setting up local routing and wrapping wireguard-go.

```
                      ┌─►mix──┐  mix     mix
                      │       │
            entry     │       │                   exit
client ───► gateway ──┘  mix  │  mix  ┌─►mix ───► gateway ───► internet
                              │       │
                              │       │
                         mix  └─►mix──┘  mix
```

## CLI client

[nym-vpn-cli](nym-vpn-cli/README.md)

## GUI client

[nym-vpn-desktop](nym-vpn-desktop/README.md)

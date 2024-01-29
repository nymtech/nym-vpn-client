# Nym VPN client

VPN client that uses the [Nym mixnet](https://nymtech.net). 

By default it will do 5-hops (incl entry and exit gateways).

```
                      ┌─►mix──┐  mix     mix
                      │       │
            entry     │       │                   exit
client ───► gateway ──┘  mix  │  mix  ┌─►mix ───► gateway ───► internet
                              │       │
                              │       │
                         mix  └─►mix──┘  mix
```

It can optionally do the first connection to the entry gateway using wireguard, and it uses Mullvad libraries for wrapping wireguard-go and to setup local routing rules to route all traffic to the TUN virtual network device.

## CLI client

[nym-vpn-cli](nym-vpn-cli/README.md)

## GUI client

[nym-vpn-desktop](nym-vpn-desktop/README.md)

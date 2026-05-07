# nym-socks5-proxy

A SOCKS5 proxy that selectively routes outbound connections either through the Nym VPN tunnel or directly via the default network interface, based on GeoIP data and domain exclusion lists. This is used to implement the Geo Exclusion feature, which allows certain traffic (e.g. traffic destined for excluded country IP ranges) to bypass the VPN.

This is both a binary and a library crate. The binary is used on desktop, where it runs as a managed subprocess of the daemon. The library is used on Android, where the proxy runs in-process.

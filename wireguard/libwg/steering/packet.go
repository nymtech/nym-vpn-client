/* SPDX-License-Identifier: GPL-3.0-only
 *
 * Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
 */

package steering

import (
	"net/netip"

	"gvisor.dev/gvisor/pkg/tcpip/header"
)

type PacketInfo struct {
	Key      FlowKey
	IsIPv4   bool
	IsTCPSyn bool
}

// ParsePacket extracts the flow key from a raw IP packet. It returns false
// for any packet that is not well-formed IPv4/IPv6 TCP or UDP; such packets
// must be routed through the tunnel.
func ParsePacket(pkt []byte) (PacketInfo, bool) {
	if len(pkt) == 0 {
		return PacketInfo{}, false
	}
	switch header.IPVersion(pkt) {
	case header.IPv4Version:
		ip := header.IPv4(pkt)
		if len(pkt) < header.IPv4MinimumSize || !ip.IsValid(len(pkt)) {
			return PacketInfo{}, false
		}
		// Reject non-initial fragments (fail-closed): only initial fragments
		// (offset 0) carry the transport header with port information.
		if ip.FragmentOffset() != 0 {
			return PacketInfo{}, false
		}
		srcBytes := ip.SourceAddress().As4()
		dstBytes := ip.DestinationAddress().As4()
		src, _ := netip.AddrFromSlice(srcBytes[:])
		dst, _ := netip.AddrFromSlice(dstBytes[:])
		return parseTransport(pkt[ip.HeaderLength():], uint8(ip.Protocol()), src, dst, true)
	case header.IPv6Version:
		if len(pkt) < header.IPv6MinimumSize {
			return PacketInfo{}, false
		}
		ip := header.IPv6(pkt)
		srcBytes := ip.SourceAddress().As16()
		dstBytes := ip.DestinationAddress().As16()
		src, _ := netip.AddrFromSlice(srcBytes[:])
		dst, _ := netip.AddrFromSlice(dstBytes[:])
		// NextHeader chains (extension headers) are rare on first hop; treat
		// anything other than a directly nested TCP/UDP as tunnel traffic.
		return parseTransport(pkt[header.IPv6MinimumSize:], uint8(ip.NextHeader()), src, dst, false)
	default:
		return PacketInfo{}, false
	}
}

func parseTransport(payload []byte, proto uint8, src, dst netip.Addr, isIPv4 bool) (PacketInfo, bool) {
	switch Proto(proto) {
	case ProtoTCP:
		if len(payload) < header.TCPMinimumSize {
			return PacketInfo{}, false
		}
		tcp := header.TCP(payload)
		return PacketInfo{
			Key: FlowKey{
				Proto: ProtoTCP,
				Src:   netip.AddrPortFrom(src, tcp.SourcePort()),
				Dst:   netip.AddrPortFrom(dst, tcp.DestinationPort()),
			},
			IsIPv4:   isIPv4,
			IsTCPSyn: tcp.Flags()&header.TCPFlagSyn != 0 && tcp.Flags()&header.TCPFlagAck == 0,
		}, true
	case ProtoUDP:
		if len(payload) < header.UDPMinimumSize {
			return PacketInfo{}, false
		}
		udp := header.UDP(payload)
		return PacketInfo{
			Key: FlowKey{
				Proto: ProtoUDP,
				Src:   netip.AddrPortFrom(src, udp.SourcePort()),
				Dst:   netip.AddrPortFrom(dst, udp.DestinationPort()),
			},
			IsIPv4: isIPv4,
		}, true
	default:
		return PacketInfo{}, false
	}
}

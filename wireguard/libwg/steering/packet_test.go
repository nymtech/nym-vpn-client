/* SPDX-License-Identifier: GPL-3.0-only
 *
 * Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
 */

package steering

import (
	"net/netip"
	"testing"

	"gvisor.dev/gvisor/pkg/tcpip"
	"gvisor.dev/gvisor/pkg/tcpip/header"
)

func buildIPv4UDP(src, dst netip.Addr, srcPort, dstPort uint16) []byte {
	payload := []byte("hi")
	length := header.IPv4MinimumSize + header.UDPMinimumSize + len(payload)
	buf := make([]byte, length)
	ip := header.IPv4(buf)
	ip.Encode(&header.IPv4Fields{
		TotalLength: uint16(length),
		TTL:         64,
		Protocol:    uint8(header.UDPProtocolNumber),
		SrcAddr:     tcpip.AddrFrom4(src.As4()),
		DstAddr:     tcpip.AddrFrom4(dst.As4()),
	})
	ip.SetChecksum(^ip.CalculateChecksum())
	udp := header.UDP(buf[header.IPv4MinimumSize:])
	udp.Encode(&header.UDPFields{
		SrcPort: srcPort,
		DstPort: dstPort,
		Length:  uint16(header.UDPMinimumSize + len(payload)),
	})
	copy(buf[header.IPv4MinimumSize+header.UDPMinimumSize:], payload)
	return buf
}

func TestParseIPv4UDP(t *testing.T) {
	src := netip.MustParseAddr("10.0.0.2")
	dst := netip.MustParseAddr("9.9.9.9")
	info, ok := ParsePacket(buildIPv4UDP(src, dst, 5353, 53))
	if !ok {
		t.Fatal("expected parse success")
	}
	if info.Key.Proto != ProtoUDP || !info.IsIPv4 {
		t.Fatalf("wrong proto/family: %+v", info)
	}
	if info.Key.Src != netip.AddrPortFrom(src, 5353) || info.Key.Dst != netip.AddrPortFrom(dst, 53) {
		t.Fatalf("wrong addrs: %+v", info.Key)
	}
}

func TestParseRejectsNonTcpUdp(t *testing.T) {
	// ICMP echo: minimal IPv4 header with protocol 1
	buf := buildIPv4UDP(netip.MustParseAddr("10.0.0.2"), netip.MustParseAddr("9.9.9.9"), 1, 1)
	header.IPv4(buf).Encode(&header.IPv4Fields{
		TotalLength: uint16(len(buf)),
		TTL:         64,
		Protocol:    1, // ICMP
		SrcAddr:     tcpip.AddrFrom4(netip.MustParseAddr("10.0.0.2").As4()),
		DstAddr:     tcpip.AddrFrom4(netip.MustParseAddr("9.9.9.9").As4()),
	})
	if _, ok := ParsePacket(buf); ok {
		t.Fatal("expected ICMP to be rejected")
	}
}

func TestParseRejectsTruncated(t *testing.T) {
	if _, ok := ParsePacket([]byte{0x45, 0x00}); ok {
		t.Fatal("expected truncated packet to be rejected")
	}
}

func buildIPv6TCP(src, dst netip.Addr, srcPort, dstPort uint16, flags header.TCPFlags) []byte {
	payload := []byte("hi")
	length := header.IPv6MinimumSize + header.TCPMinimumSize + len(payload)
	buf := make([]byte, length)
	ip := header.IPv6(buf)
	ip.Encode(&header.IPv6Fields{
		PayloadLength:     uint16(header.TCPMinimumSize + len(payload)),
		TransportProtocol: header.TCPProtocolNumber,
		HopLimit:          64,
		SrcAddr:           tcpip.AddrFrom16(src.As16()),
		DstAddr:           tcpip.AddrFrom16(dst.As16()),
	})
	tcp := header.TCP(buf[header.IPv6MinimumSize:])
	tcp.Encode(&header.TCPFields{
		SrcPort:    srcPort,
		DstPort:    dstPort,
		SeqNum:     1000,
		AckNum:     0,
		DataOffset: header.TCPMinimumSize,
		Flags:      flags,
		WindowSize: 65535,
	})
	copy(buf[header.IPv6MinimumSize+header.TCPMinimumSize:], payload)
	return buf
}

func TestParseIPv6TCP(t *testing.T) {
	src := netip.MustParseAddr("2001:db8::1")
	dst := netip.MustParseAddr("2001:db8::2")
	info, ok := ParsePacket(buildIPv6TCP(src, dst, 12345, 443, header.TCPFlagSyn))
	if !ok {
		t.Fatal("expected parse success")
	}
	if info.Key.Proto != ProtoTCP || info.IsIPv4 {
		t.Fatalf("wrong proto/family: %+v", info)
	}
	if info.Key.Src != netip.AddrPortFrom(src, 12345) || info.Key.Dst != netip.AddrPortFrom(dst, 443) {
		t.Fatalf("wrong addrs: %+v", info.Key)
	}
	if !info.IsTCPSyn {
		t.Fatalf("expected IsTCPSyn to be true when SYN flag is set")
	}
}

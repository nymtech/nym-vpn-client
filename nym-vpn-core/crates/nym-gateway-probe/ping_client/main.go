/* SPDX-License-Identifier: MIT
 *
 * Copyright (C) 2017-2023 WireGuard LLC. All Rights Reserved.
 */

package main

import (
	"bytes"
	"flag"
	"fmt"
	"log"
	"net/netip"
	"strings"
	"time"

	"golang.org/x/net/icmp"
	"golang.org/x/net/ipv4"

	"golang.zx2c4.com/wireguard/conn"
	"golang.zx2c4.com/wireguard/device"
	"golang.zx2c4.com/wireguard/tun/netstack"
)

const PING_ADDR = "1.1.1.1"
const WRITE_TIMEOUT = 1
const READ_TIMEOUT = 5

func main() {
	var (
		ip          = flag.String("ip", "", "Internal IP assigned to the probe")
		private_key = flag.String("private-key", "", "Private key of the probe")
		public_key  = flag.String("public-key", "", "Gateway public key")
		endpoint    = flag.String("endpoint", "", "Gateway WG endpoint")
	)

	flag.Parse()

	tun, tnet, err := netstack.CreateNetTUN(
		[]netip.Addr{netip.MustParseAddr(*ip)},
		[]netip.Addr{netip.MustParseAddr("1.1.1.1")},
		1280)
	if err != nil {
		log.Panic(err)
	}
	dev := device.NewDevice(tun, conn.NewDefaultBind(), device.NewLogger(device.LogLevelVerbose, ""))

	var ipc strings.Builder

	ipc.WriteString("private_key=")
	ipc.WriteString(*private_key)
	ipc.WriteString("\npublic_key=")
	ipc.WriteString(*public_key)
	ipc.WriteString("\nendpoint=")
	ipc.WriteString(*endpoint)
	ipc.WriteString("\nallowed_ip=0.0.0.0/0\n")

	dev.IpcSet(ipc.String())
	err = dev.Up()
	if err != nil {
		log.Panic(err)
	}

	for i := uint16(0); i < 3; i++ {
		log.Printf("Send ping seq=%d", i)

		rt, err := sendPing(PING_ADDR, i, tnet)
		if err != nil {
			log.Printf("Failed to send ping: %v\n", err)
			continue
		}
		log.Printf("Ping latency: %v\n", rt)
		break
	}
}

func sendPing(address string, seq uint16, tnet *netstack.Net) (time.Duration, error) {
	socket, err := tnet.Dial("ping4", address)
	if err != nil {
		return 0, err
	}

	requestPing := icmp.Echo{
		ID:   1337,
		Seq:  int(seq),
		Data: []byte("gopher burrow"),
	}
	icmpBytes, _ := (&icmp.Message{Type: ipv4.ICMPTypeEcho, Code: 0, Body: &requestPing}).Marshal(nil)
	start := time.Now()

	socket.SetWriteDeadline(time.Now().Add(time.Second * WRITE_TIMEOUT))
	_, err = socket.Write(icmpBytes)
	if err != nil {
		return 0, err
	}

	// Wait until either the right reply arrives or timeout
	for {
		socket.SetReadDeadline(time.Now().Add(time.Second * READ_TIMEOUT))
		n, err := socket.Read(icmpBytes[:])
		if err != nil {
			return 0, err
		}

		replyPacket, err := icmp.ParseMessage(1, icmpBytes[:n])
		if err != nil {
			return 0, err
		}
		replyPing, ok := replyPacket.Body.(*icmp.Echo)

		if !ok {
			return 0, fmt.Errorf("invalid reply type: %v", replyPacket)
		}

		if bytes.Equal(replyPing.Data, requestPing.Data) {
			// Check if seq is the same, because otherwise we might have received a reply from the preceding ping request.
			if replyPing.Seq != requestPing.Seq {
				log.Printf("Got echo reply from timed out request (expected %d, received %d)", requestPing.Seq, replyPing.Seq)
			} else {
				return time.Since(start), nil
			}
		} else {
			return 0, fmt.Errorf("invalid ping reply: %v (request: %v)", replyPing, requestPing)
		}
	}
}

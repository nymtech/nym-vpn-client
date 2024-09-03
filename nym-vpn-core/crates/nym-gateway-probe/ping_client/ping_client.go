//go:build ignore

/* SPDX-License-Identifier: MIT
 *
 * Copyright (C) 2017-2023 WireGuard LLC. All Rights Reserved.
 */

package main

import (
	"bytes"
	"flag"
	"log"
	"math/rand"
	"net/netip"
	"strings"
	"time"

	"golang.org/x/net/icmp"
	"golang.org/x/net/ipv4"

	"golang.zx2c4.com/wireguard/conn"
	"golang.zx2c4.com/wireguard/device"
	"golang.zx2c4.com/wireguard/tun/netstack"
)

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

	socket, err := tnet.Dial("ping4", "google.com")
	if err != nil {
		log.Panic(err)
	}
	requestPing := icmp.Echo{
		Seq:  rand.Intn(1 << 16),
		Data: []byte("gopher burrow"),
	}
	icmpBytes, _ := (&icmp.Message{Type: ipv4.ICMPTypeEcho, Code: 0, Body: &requestPing}).Marshal(nil)
	socket.SetReadDeadline(time.Now().Add(time.Second * 20))
	start := time.Now()
	_, err = socket.Write(icmpBytes)
	if err != nil {
		log.Panic(err)
	}
	n, err := socket.Read(icmpBytes[:])
	if err != nil {
		log.Panic(err)
	}
	replyPacket, err := icmp.ParseMessage(1, icmpBytes[:n])
	if err != nil {
		log.Panic(err)
	}
	replyPing, ok := replyPacket.Body.(*icmp.Echo)
	if !ok {
		log.Panicf("invalid reply type: %v", replyPacket)
	}
	if !bytes.Equal(replyPing.Data, requestPing.Data) || replyPing.Seq != requestPing.Seq {
		log.Panicf("invalid ping reply: %v", replyPing)
	}
	log.Printf("Ping latency: %v", time.Since(start))
}

package main

import (
	"bytes"
	"fmt"
	"log"
	"net/netip"
	"strings"
	"time"

	"golang.org/x/net/icmp"
	"golang.org/x/net/ipv4"
	"github.com/amnezia-vpn/amneziawg-go/conn"
	"github.com/amnezia-vpn/amneziawg-go/device"
	"github.com/amnezia-vpn/amneziawg-go/tun/netstack"
)

type Netstack struct{}

func init() {
	NetstackCallImpl = Netstack{}
}

func (Netstack) ping(req NetstackRequest) NetstackResponse {
	tun, tnet, err := netstack.CreateNetTUN(
		[]netip.Addr{netip.MustParseAddr(req.wg_ip)},
		[]netip.Addr{netip.MustParseAddr(req.dns)},
		1280)

	if err != nil {
		log.Panic(err)
	}
	dev := device.NewDevice(tun, conn.NewDefaultBind(), device.NewLogger(device.LogLevelError, ""))

	var ipc strings.Builder

	ipc.WriteString("private_key=")
	ipc.WriteString(req.private_key)
	ipc.WriteString("\npublic_key=")
	ipc.WriteString(req.public_key)
	ipc.WriteString("\nendpoint=")
	ipc.WriteString(req.endpoint)
	ipc.WriteString("\nallowed_ip=0.0.0.0/0\n")

	response := NetstackResponse{false, 0, 0, 0, 0, false}

	dev.IpcSet(ipc.String())
	err = dev.Up()
	if err != nil {
		log.Panic(err)
	}

	response.can_handshake = true

	for _, host := range req.ping_hosts {
		for i := uint8(0); i < req.num_ping; i++ {
			log.Printf("Pinging %s seq=%d", host, i)
			response.sent_hosts += 1
			rt, err := sendPing(host, i, req.send_timeout_sec, req.recv_timeout_sec, tnet)
			if err != nil {
				log.Printf("Failed to send ping: %v\n", err)
				continue
			}
			response.received_hosts += 1
			response.can_resolve_dns = true
			log.Printf("Ping latency: %v\n", rt)
		}
	}

	for _, ip := range req.ping_ips {
		for i := uint8(0); i < req.num_ping; i++ {
			log.Printf("Pinging %s seq=%d", ip, i)
			response.sent_ips += 1
			rt, err := sendPing(ip, i, req.send_timeout_sec, req.recv_timeout_sec, tnet)
			if err != nil {
				log.Printf("Failed to send ping: %v\n", err)
				continue
			}
			response.received_ips += 1
			log.Printf("Ping latency: %v\n", rt)
		}
	}

	return response
}

func sendPing(address string, seq uint8, send_timeout_secs uint64, recieve_timout_secs uint64, tnet *netstack.Net) (time.Duration, error) {
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

	socket.SetWriteDeadline(time.Now().Add(time.Second * time.Duration(send_timeout_secs)))
	_, err = socket.Write(icmpBytes)
	if err != nil {
		return 0, err
	}

	// Wait until either the right reply arrives or timeout
	for {
		socket.SetReadDeadline(time.Now().Add(time.Second * time.Duration(recieve_timout_secs)))
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

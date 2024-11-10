package main

import (
	"bytes"
	"fmt"
	"log"
	"net"
	"net/netip"
	"strings"
	"time"

	"golang.org/x/net/icmp"
	"golang.org/x/net/ipv4"
	"golang.org/x/net/ipv6"
	"golang.zx2c4.com/wireguard/conn"
	"golang.zx2c4.com/wireguard/device"
	"golang.zx2c4.com/wireguard/tun/netstack"
)

type Netstack struct{}

func init() {
	NetstackCallImpl = Netstack{}
}

func (Netstack) ping(req NetstackRequestGo) NetStackResponse {

	fmt.Printf("Endpoint: %s\n", req.endpoint)
	fmt.Printf("WireGuard IP: %s\n", req.wg_ip)
	fmt.Printf("IP version: %d\n", req.ip_version)

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
	if req.ip_version == 4 {
		ipc.WriteString("\nallowed_ip=0.0.0.0/0\n")
	} else {
		ipc.WriteString("\nallowed_ip=::/0\n")
	}

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
			rt, err := sendPing(host, i, req.send_timeout_sec, req.recv_timeout_sec, tnet, req.ip_version)
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
			rt, err := sendPing(ip, i, req.send_timeout_sec, req.recv_timeout_sec, tnet, req.ip_version)
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

func sendPing(address string, seq uint8, send_timeout_secs uint64, recieve_timout_secs uint64, tnet *netstack.Net, ip_version uint8) (time.Duration, error) {
	var socket net.Conn
	var err error
	if ip_version == 4 {
		socket, err = tnet.Dial("ping4", address)
	} else {
		socket, err = tnet.Dial("ping6", address)
	}

	if err != nil {
		return 0, err
	}

	var icmpBytes []byte

	requestPing := icmp.Echo{
		ID:   1337,
		Seq:  int(seq),
		Data: []byte("gopher burrow"),
	}

	if ip_version == 4 {
		icmpBytes, _ = (&icmp.Message{Type: ipv4.ICMPTypeEcho, Code: 0, Body: &requestPing}).Marshal(nil)
	} else {
		icmpBytes, _ = (&icmp.Message{Type: ipv6.ICMPTypeEchoRequest, Code: 0, Body: &requestPing}).Marshal(nil)
	}

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

		var proto int
		if ip_version == 4 {
			proto = 1
		} else {
			proto = 58
		}

		replyPacket, err := icmp.ParseMessage(proto, icmpBytes[:n])
		if err != nil {
			return 0, err
		}

		var ok bool

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

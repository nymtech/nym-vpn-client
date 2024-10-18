package main

import (
	"bytes"
	"context"
	"fmt"
	"io"
	"log"
	"net"
	"net/http"
	"net/netip"
	"strings"
	"time"

	"math/rand"

	"golang.org/x/net/icmp"
	"golang.org/x/net/ipv4"
	"golang.zx2c4.com/wireguard/conn"
	"golang.zx2c4.com/wireguard/device"
	"golang.zx2c4.com/wireguard/tun/netstack"
)

type Netstack struct{}

func init() {
	NetstackCallImpl = Netstack{}
}

var fileUrls = []string{
	"https://hil-speed.hetzner.com/100MB.bin",
	"https://nbg1-speed.hetzner.com/100MB.bin",
	"https://fsn1-speed.hetzner.com/100MB.bin",
	"https://ash-speed.hetzner.com/100MB.bin",
	"https://hel1-speed.hetzner.com/100MB.bin",
	"https://proof.ovh.net/files/100Mb.dat",
	"http://cachefly.cachefly.net/100mb.test",
	"https://sin-speed.hetzner.com/100MB.bin",
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

	response := NetstackResponse{false, 0, 0, 0, 0, false, "", 0, ""}

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

	randomIndex := rand.Intn(len(fileUrls))
	fileURL := fileUrls[randomIndex]

	// Download the file
	fileContent, downloadDuration, err := downloadFile(fileURL, req.download_timeout_sec, tnet)
	if err != nil {
		log.Printf("Failed to download file: %v\n", err)
	} else {
		log.Printf("Downloaded file content length: %.2f MB\n", float64(len(fileContent))/1024/1024)
		log.Printf("Download duration: %v\n", downloadDuration)
	}

	response.download_duration = uint64(downloadDuration.Seconds())
	response.downloaded_file = fileURL
	if err != nil {
		response.download_err = err.Error()
	} else {
		response.download_err = ""
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

func downloadFile(url string, timeoutSecs uint64, tnet *netstack.Net) ([]byte, time.Duration, error) {
	transport := &http.Transport{
		DialContext: func(ctx context.Context, network, addr string) (net.Conn, error) {
			return tnet.Dial(network, addr)
		},
	}

	client := &http.Client{
		Transport: transport,
		Timeout:   time.Second * time.Duration(timeoutSecs),
	}

	start := time.Now() // Start timing

	resp, err := client.Get(url)
	if err != nil {
		return nil, 0, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, 0, fmt.Errorf("failed to download file: %s", resp.Status)
	}

	var buf bytes.Buffer
	_, err = io.Copy(&buf, resp.Body)
	if err != nil {
		return nil, 0, err
	}

	duration := time.Since(start) // Calculate duration

	return buf.Bytes(), duration, nil
}

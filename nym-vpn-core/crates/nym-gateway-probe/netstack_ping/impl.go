package main

import (
	"bytes"
	"context"
	"fmt"
	"io"
	"log"
	"math/rand"
	"net"
	"net/http"
	"net/netip"
	"strings"
	"time"

	"github.com/amnezia-vpn/amneziawg-go/conn"
	"github.com/amnezia-vpn/amneziawg-go/device"
	"github.com/amnezia-vpn/amneziawg-go/tun/netstack"
	"golang.org/x/net/icmp"
	"golang.org/x/net/ipv4"
	"golang.org/x/net/ipv6"
)

var fileUrls = []string{
	"https://proof.ovh.net/files/1Mb.dat",
	"https://proof.ovh.net/files/10Mb.dat",
}

var fileUrlsV6 = []string{
	"https://proof.ovh.net/files/1Mb.dat",
	"https://proof.ovh.net/files/10Mb.dat",
}

type Netstack struct{}

func init() {
	NetstackCallImpl = Netstack{}
}

func (Netstack) ping(req NetstackRequestGo) NetstackResponse {

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
	if req.awg_args != "" {
		awg := strings.ReplaceAll(req.awg_args, "\\n", "\n")
		ipc.WriteString(fmt.Sprintf("\n%s", awg))
	}
	ipc.WriteString("\npublic_key=")
	ipc.WriteString(req.public_key)
	ipc.WriteString("\nendpoint=")
	ipc.WriteString(req.endpoint)
	if req.ip_version == 4 {
		ipc.WriteString("\nallowed_ip=0.0.0.0/0\n")
	} else {
		ipc.WriteString("\nallowed_ip=::/0\n")
	}

	response := NetstackResponse{false, 0, 0, 0, 0, false, "", 0, ""}

	dev.IpcSet(ipc.String())

	config, err := dev.IpcGet()
	if err != nil {
		log.Panic(err)
	}
	log.Printf("%s", config)

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
			func() {
				defer time.Sleep(5 * time.Second)
				log.Printf("Pinging %s seq=%d", ip, i)
				response.sent_ips += 1
				rt, err := sendPing(ip, i, req.send_timeout_sec, req.recv_timeout_sec, tnet, req.ip_version)
				if err != nil {
					log.Printf("Failed to send ping: %v\n", err)
					return
				}
				response.received_ips += 1
				log.Printf("Ping latency: %v\n", rt)
			}()
		}
	}

	var fileURL string

	if req.ip_version == 4 {
		randomIndex := rand.Intn(len(fileUrls))
		fileURL = fileUrls[randomIndex]
	} else {
		randomIndex := rand.Intn(len(fileUrlsV6))
		fileURL = fileUrlsV6[randomIndex]
	}

	// Download the file
	fileContent, downloadDuration, err := downloadFile(fileURL, req.download_timeout_sec, tnet)
	if err != nil {
		log.Printf("Failed to download file: %v\n", err)
	} else {
		log.Printf("Downloaded file content length: %.2f MB\n", float64(len(fileContent))/1024/1024)
		log.Printf("Download duration: %v\n", downloadDuration)
	}

	response.download_duration_sec = uint64(downloadDuration.Seconds())
	response.downloaded_file = fileURL
	if err != nil {
		response.download_error = err.Error()
	} else {
		response.download_error = ""
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

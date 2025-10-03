/* SPDX-License-Identifier: MIT
 *
 * Copyright (C) 2024 Nym Technologies SA <contact@nymtech.net>. All Rights Reserved.
 */

package main

// #include <stdlib.h>
import "C"

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"math/rand"
	"net"
	"net/http"
	"net/netip"
	"strings"
	"time"
	"unsafe"

	"github.com/amnezia-vpn/amneziawg-go/conn"
	"github.com/amnezia-vpn/amneziawg-go/device"
	"github.com/amnezia-vpn/amneziawg-go/tun/netstack"
	"golang.org/x/net/icmp"
	"golang.org/x/net/ipv4"
	"golang.org/x/net/ipv6"
)

var fileUrls = []string{
	"https://proof.ovh.net/files/1Mb.dat",
}

var fileUrlsV6 = []string{
	"https://proof.ovh.net/files/1Mb.dat",
}

type NetstackRequestGo struct {
	WgIp               string   `json:"wg_ip"`
	PrivateKey         string   `json:"private_key"`
	PublicKey          string   `json:"public_key"`
	Endpoint           string   `json:"endpoint"`
	Dns                string   `json:"dns"`
	IpVersion          uint8    `json:"ip_version"`
	PingHosts          []string `json:"ping_hosts"`
	PingIps            []string `json:"ping_ips"`
	NumPing            uint8    `json:"num_ping"`
	SendTimeoutSec     uint64   `json:"send_timeout_sec"`
	RecvTimeoutSec     uint64   `json:"recv_timeout_sec"`
	DownloadTimeoutSec uint64   `json:"download_timeout_sec"`
	AwgArgs            string   `json:"awg_args"`
}

type NetstackResponse struct {
	CanHandshake                 bool   `json:"can_handshake"`
	SentIps                      uint16 `json:"sent_ips"`
	ReceivedIps                  uint16 `json:"received_ips"`
	SentHosts                    uint16 `json:"sent_hosts"`
	ReceivedHosts                uint16 `json:"received_hosts"`
	CanResolveDns                bool   `json:"can_resolve_dns"`
	DownloadedFile               string `json:"downloaded_file"`
	DownloadedFileSizeBytes      uint64 `json:"downloaded_file_size_bytes"`
	DownloadDurationSec          uint64 `json:"download_duration_sec"`
	DownloadDurationMilliseconds uint64 `json:"download_duration_milliseconds"`
	DownloadError                string `json:"download_error"`
}

type SuccessResult = struct {
	Response NetstackResponse `json:"response"`
}

type ErrorResult = struct {
	Error string `json:"error"`
}

func jsonResponse(response NetstackResponse) *C.char {
	bytes, serializeErr := json.Marshal(SuccessResult{
		Response: response,
	})
	if serializeErr == nil {
		return C.CString(string(bytes))
	} else {
		return C.CString("{\"error\":\"" + serializeErr.Error() + "\"}")
	}
}

func jsonError(err error) *C.char {
	jsonErr := ErrorResult{
		Error: fmt.Sprintf("failed to parse request: %s", err.Error()),
	}
	bytes, serializeErr := json.Marshal(jsonErr)
	if serializeErr == nil {
		return C.CString(string(bytes))
	} else {
		return C.CString("{\"error\":\"" + serializeErr.Error() + "\"}")
	}
}

//export wgPing
func wgPing(cReq *C.char) *C.char {
	reqStr := C.GoString(cReq)

	var req NetstackRequestGo
	err := json.Unmarshal([]byte(reqStr), &req)
	if err != nil {
		log.Printf("Failed to parse request: %s", err)
		return jsonError(err)
	}

	response, err := ping(req)
	if err != nil {
		log.Printf("Failed to ping: %s", err)
		return jsonError(err)
	}

	return jsonResponse(response)
}

//export wgFreePtr
func wgFreePtr(ptr unsafe.Pointer) {
	C.free(ptr)
}

func ping(req NetstackRequestGo) (NetstackResponse, error) {
	fmt.Printf("Endpoint: %s\n", req.Endpoint)
	fmt.Printf("WireGuard IP: %s\n", req.WgIp)
	fmt.Printf("IP version: %d\n", req.IpVersion)

	tun, tnet, err := netstack.CreateNetTUN(
		[]netip.Addr{netip.MustParseAddr(req.WgIp)},
		[]netip.Addr{netip.MustParseAddr(req.Dns)},
		1280)

	if err != nil {
		return NetstackResponse{}, err
	}
	dev := device.NewDevice(tun, conn.NewDefaultBind(), device.NewLogger(device.LogLevelError, ""))

	var ipc strings.Builder

	ipc.WriteString("private_key=")
	ipc.WriteString(req.PrivateKey)
	if req.AwgArgs != "" {
		awg := strings.ReplaceAll(req.AwgArgs, "\\n", "\n")
		ipc.WriteString(fmt.Sprintf("\n%s", awg))
	}
	ipc.WriteString("\npublic_key=")
	ipc.WriteString(req.PublicKey)
	ipc.WriteString("\nendpoint=")
	ipc.WriteString(req.Endpoint)
	if req.IpVersion == 4 {
		ipc.WriteString("\nallowed_ip=0.0.0.0/0\n")
	} else {
		ipc.WriteString("\nallowed_ip=::/0\n")
	}

	response := NetstackResponse{false, 0, 0, 0, 0, false, "", 0, 0, 0, ""}

	dev.IpcSet(ipc.String())

	config, err := dev.IpcGet()
	if err != nil {
		return NetstackResponse{}, err
	}
	log.Printf("%s", config)

	err = dev.Up()
	if err != nil {
		return NetstackResponse{}, err
	}

	response.CanHandshake = true

	for _, host := range req.PingHosts {
		for i := uint8(0); i < req.NumPing; i++ {
			log.Printf("Pinging %s seq=%d", host, i)
			response.SentHosts += 1
			rt, err := sendPing(host, i, req.SendTimeoutSec, req.RecvTimeoutSec, tnet, req.IpVersion)
			if err != nil {
				log.Printf("Failed to send ping: %v\n", err)
				continue
			}
			response.ReceivedHosts += 1
			response.CanResolveDns = true
			log.Printf("Ping latency: %v\n", rt)
		}
	}

	for _, ip := range req.PingIps {
		for i := uint8(0); i < req.NumPing; i++ {
			func() {
				defer time.Sleep(5 * time.Second)
				log.Printf("Pinging %s seq=%d", ip, i)
				response.SentIps += 1
				rt, err := sendPing(ip, i, req.SendTimeoutSec, req.RecvTimeoutSec, tnet, req.IpVersion)
				if err != nil {
					log.Printf("Failed to send ping: %v\n", err)
					return
				}
				response.ReceivedIps += 1
				log.Printf("Ping latency: %v\n", rt)
			}()
		}
	}

	var fileURL string

	if req.IpVersion == 4 {
		randomIndex := rand.Intn(len(fileUrls))
		fileURL = fileUrls[randomIndex]
	} else {
		randomIndex := rand.Intn(len(fileUrlsV6))
		fileURL = fileUrlsV6[randomIndex]
	}

	// Download the file
	fileContent, downloadDuration, err := downloadFile(fileURL, req.DownloadTimeoutSec, tnet)
	if err != nil {
		log.Printf("Failed to download file: %v\n", err)
	} else {
		log.Printf("Downloaded file content length: %.2f MB\n", float64(len(fileContent))/1024/1024)
		log.Printf("Download duration: %v\n", downloadDuration)
	}

	response.DownloadDurationSec = uint64(downloadDuration.Seconds())
	response.DownloadDurationMilliseconds = uint64(downloadDuration.Milliseconds())
	response.DownloadedFile = fileURL
	if err != nil {
		response.DownloadError = err.Error()
		response.DownloadedFileSizeBytes = 0
	} else {
		response.DownloadError = ""
		response.DownloadedFileSizeBytes = uint64(len(fileContent))
	}

	return response, nil
}

func sendPing(address string, seq uint8, sendTtimeoutSecs uint64, receiveTimoutSecs uint64, tnet *netstack.Net, ipVersion uint8) (time.Duration, error) {
	var socket net.Conn
	var err error
	if ipVersion == 4 {
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

	if ipVersion == 4 {
		icmpBytes, _ = (&icmp.Message{Type: ipv4.ICMPTypeEcho, Code: 0, Body: &requestPing}).Marshal(nil)
	} else {
		icmpBytes, _ = (&icmp.Message{Type: ipv6.ICMPTypeEchoRequest, Code: 0, Body: &requestPing}).Marshal(nil)
	}

	start := time.Now()

	socket.SetWriteDeadline(time.Now().Add(time.Second * time.Duration(sendTtimeoutSecs)))
	_, err = socket.Write(icmpBytes)
	if err != nil {
		return 0, err
	}

	// Wait until either the right reply arrives or timeout
	for {
		socket.SetReadDeadline(time.Now().Add(time.Second * time.Duration(receiveTimoutSecs)))
		n, err := socket.Read(icmpBytes[:])
		if err != nil {
			return 0, err
		}

		var proto int
		if ipVersion == 4 {
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

func main() {}

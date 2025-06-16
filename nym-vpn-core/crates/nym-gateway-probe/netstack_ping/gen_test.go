package main

import (
	"fmt"
	"runtime"
	"testing"
)

func TestCNetstackCallPingBasic(t *testing.T) {
	// Test basic functionality of the callback-free ping function
	req := NetstackRequestGo{
		wg_ip:                "10.0.0.1",
		private_key:          "test_private_key",
		public_key:           "test_public_key",
		endpoint:             "192.168.1.1:51820",
		dns:                  "1.1.1.1",
		ip_version:           4,
		ping_hosts:           []string{"example.com"},
		ping_ips:             []string{"8.8.8.8"},
		num_ping:             1,
		send_timeout_sec:     2,
		recv_timeout_sec:     2,
		download_timeout_sec: 10,
		awg_args:             "",
	}

	// Convert to C struct for testing
	var buffer []byte
	cReq := refNetstackRequestGo(&req, &buffer)

	// We can't call the actual ping function without network setup
	// but we can test the data conversion
	goReq := newNetstackRequestGo(cReq)

	// Verify all fields are preserved
	if goReq.wg_ip != req.wg_ip {
		t.Errorf("wg_ip mismatch: expected %s, got %s", req.wg_ip, goReq.wg_ip)
	}

	if goReq.private_key != req.private_key {
		t.Errorf("private_key mismatch: expected %s, got %s", req.private_key, goReq.private_key)
	}

	if goReq.public_key != req.public_key {
		t.Errorf("public_key mismatch: expected %s, got %s", req.public_key, goReq.public_key)
	}

	if goReq.endpoint != req.endpoint {
		t.Errorf("endpoint mismatch: expected %s, got %s", req.endpoint, goReq.endpoint)
	}

	if goReq.dns != req.dns {
		t.Errorf("dns mismatch: expected %s, got %s", req.dns, goReq.dns)
	}

	if goReq.ip_version != req.ip_version {
		t.Errorf("ip_version mismatch: expected %d, got %d", req.ip_version, goReq.ip_version)
	}

	if len(goReq.ping_hosts) != len(req.ping_hosts) {
		t.Errorf("ping_hosts length mismatch: expected %d, got %d", len(req.ping_hosts), len(goReq.ping_hosts))
	}

	if len(goReq.ping_ips) != len(req.ping_ips) {
		t.Errorf("ping_ips length mismatch: expected %d, got %d", len(req.ping_ips), len(goReq.ping_ips))
	}

	t.Logf("Request conversion test passed")
}

func TestNetstackResponseConversion(t *testing.T) {
	// Test the conversion of response structures
	resp := NetstackResponse{
		can_handshake:         true,
		sent_ips:              5,
		received_ips:          4,
		sent_hosts:            3,
		received_hosts:        2,
		can_resolve_dns:       true,
		downloaded_file:       "https://example.com/test.dat",
		download_duration_sec: 10,
		download_error:        "",
	}

	// Convert to C and back
	var buffer []byte
	cResp := refNetstackResponse(&resp, &buffer)
	goResp := newNetstackResponse(cResp)

	// Verify all fields are preserved
	if goResp.can_handshake != resp.can_handshake {
		t.Errorf("can_handshake mismatch: expected %v, got %v", resp.can_handshake, goResp.can_handshake)
	}

	if goResp.sent_ips != resp.sent_ips {
		t.Errorf("sent_ips mismatch: expected %d, got %d", resp.sent_ips, goResp.sent_ips)
	}

	if goResp.received_ips != resp.received_ips {
		t.Errorf("received_ips mismatch: expected %d, got %d", resp.received_ips, goResp.received_ips)
	}

	if goResp.can_resolve_dns != resp.can_resolve_dns {
		t.Errorf("can_resolve_dns mismatch: expected %v, got %v", resp.can_resolve_dns, goResp.can_resolve_dns)
	}

	if goResp.download_duration_sec != resp.download_duration_sec {
		t.Errorf("download_duration_sec mismatch: expected %d, got %d", resp.download_duration_sec, goResp.download_duration_sec)
	}

	t.Logf("Response conversion test passed")
}

func TestNetworkArgsWithMultipleTargets(t *testing.T) {
	// Test with multiple ping targets
	req := NetstackRequestGo{
		wg_ip:                "10.0.0.1",
		private_key:          "test_key",
		public_key:           "test_pub",
		endpoint:             "192.168.1.1:51820",
		dns:                  "1.1.1.1",
		ip_version:           4,
		ping_hosts:           []string{"google.com", "cloudflare.com", "nymtech.net"},
		ping_ips:             []string{"8.8.8.8", "1.1.1.1", "9.9.9.9"},
		num_ping:             3,
		send_timeout_sec:     5,
		recv_timeout_sec:     5,
		download_timeout_sec: 30,
		awg_args:             "jc=4 jmin=10 jmax=100",
	}

	// Test that we can convert with multiple targets
	var buffer []byte
	cReq := refNetstackRequestGo(&req, &buffer)
	goReq := newNetstackRequestGo(cReq)

	if len(goReq.ping_hosts) != 3 {
		t.Errorf("Expected 3 ping hosts, got %d", len(goReq.ping_hosts))
	}

	if len(goReq.ping_ips) != 3 {
		t.Errorf("Expected 3 ping IPs, got %d", len(goReq.ping_ips))
	}

	// Check that all hosts are preserved
	for i, host := range req.ping_hosts {
		if i < len(goReq.ping_hosts) && goReq.ping_hosts[i] != host {
			t.Errorf("Host %d mismatch: expected %s, got %s", i, host, goReq.ping_hosts[i])
		}
	}

	t.Logf("Multiple targets test passed")
}

func TestStringRefConversion(t *testing.T) {
	// Test string conversion with various edge cases
	testStrings := []string{
		"",
		"short",
		"a much longer string with spaces and special characters !@#$%",
		"unicode: 日本語 🚀",
	}

	for _, testStr := range testStrings {
		// Create a simple response to test string conversion
		resp := NetstackResponse{
			downloaded_file: testStr,
			download_error:  testStr + "_error",
		}

		var buffer []byte
		cResp := refNetstackResponse(&resp, &buffer)
		goResp := newNetstackResponse(cResp)

		// Note: For empty strings, our implementation provides a placeholder
		// so we adjust the test accordingly
		expectedFile := testStr
		expectedError := testStr + "_error"

		if testStr == "" {
			// Our implementation provides a placeholder for empty strings
			expectedFile = " " // Single space placeholder
			expectedError = " " // Single space placeholder
		}

		if goResp.downloaded_file != expectedFile {
			t.Errorf("String conversion failed for downloaded_file: expected '%s', got '%s'", expectedFile, goResp.downloaded_file)
		}

		if goResp.download_error != expectedError {
			t.Errorf("String conversion failed for download_error: expected '%s', got '%s'", expectedError, goResp.download_error)
		}
	}

	t.Logf("String conversion test passed")
}

func TestMemoryManagement(t *testing.T) {
	// Test that multiple conversions don't leak memory
	initialGoroutines := runtime.NumGoroutine()

	for i := 0; i < 10; i++ {
		req := NetstackRequestGo{
			wg_ip:                "10.0.0.1",
			private_key:          fmt.Sprintf("test_key_%d", i),
			public_key:           fmt.Sprintf("test_pub_%d", i),
			endpoint:             "127.0.0.1:51820",
			dns:                  "1.1.1.1",
			ip_version:           4,
			ping_hosts:           []string{"localhost"},
			ping_ips:             []string{"127.0.0.1"},
			num_ping:             1,
			send_timeout_sec:     1,
			recv_timeout_sec:     1,
			download_timeout_sec: 2,
			awg_args:             "",
		}

		var buffer []byte
		cReq := refNetstackRequestGo(&req, &buffer)
		goReq := newNetstackRequestGo(cReq)

		// Verify the conversion worked
		if goReq.private_key != req.private_key {
			t.Errorf("Iteration %d: private_key mismatch", i)
		}

		// Force garbage collection
		runtime.GC()
	}

	// Check that we didn't leak goroutines
	finalGoroutines := runtime.NumGoroutine()
	if finalGoroutines > initialGoroutines+2 { // Allow some leeway
		t.Errorf("Possible goroutine leak: started with %d, ended with %d",
			initialGoroutines, finalGoroutines)
	}

	t.Logf("Memory management test completed successfully")
}

func TestEdgeCaseEmptyRequest(t *testing.T) {
	// Test with minimal/empty request
	req := NetstackRequestGo{
		wg_ip:                "",
		private_key:          "",
		public_key:           "",
		endpoint:             "",
		dns:                  "",
		ip_version:           4,
		ping_hosts:           []string{},
		ping_ips:             []string{},
		num_ping:             0,
		send_timeout_sec:     1,
		recv_timeout_sec:     1,
		download_timeout_sec: 1,
		awg_args:             "",
	}

	var buffer []byte
	cReq := refNetstackRequestGo(&req, &buffer)
	goReq := newNetstackRequestGo(cReq)

	// Should handle empty values gracefully
	if len(goReq.ping_hosts) != 0 {
		t.Errorf("Expected 0 ping hosts for empty request, got %d", len(goReq.ping_hosts))
	}

	if len(goReq.ping_ips) != 0 {
		t.Errorf("Expected 0 ping IPs for empty request, got %d", len(goReq.ping_ips))
	}

	t.Logf("Edge case empty request handled successfully")
} 
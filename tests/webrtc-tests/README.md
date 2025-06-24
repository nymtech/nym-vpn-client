# Nym VPN WebRTC Leak Detection

WebRTC leak testing for Nym VPN connections. Detects if your real IP addresses (IPv4/IPv6) leak through WebRTC while connected to Nym VPN

## What it Tests

WebRTC can bypass VPN tunnels and expose your real IP through:
- Browser connections to STUN servers
- Local network interface enumeration
- IPv6 connectivity when IPv4 is protected

## Testing Method

Uses headless browser automation to create real WebRTC connections and detect IP leaks:
- Tests IPv4 and IPv6 separately with multiple STUN servers
- mDNS (.local) address detection
- TURN server testing for more coverage
- Data channel connectivity testing
- Mobile device simulation support (iPhone 15 Pro, Pixel 7)
- Compares results against pre-VPN baseline
- Categorizes local vs public IP exposure

## Usage

**Prerequisites**: Nym VPN must be running and connected before running this test (caveat  - you can also run this locally as a pre-test without any vpn connection)

### Local Testing
Run WebRTC test with Nym VPN connected:

```
cd webrtc-tests
npx playwright install chromium
node webrtc-leak-test.js

# Test mobile device simulation
WEBRTC_MOBILE=true node webrtc-leak-test.js

# Test Android device simulation
WEBRTC_ANDROID=true node webrtc-leak-test.js
```

### Docker Testing
Build and run the WebRTC test in Docker:

```
cd webrtc-tests
docker build -t webrtc-leak-test .

# Run with host network access (to detect VPN on host)
docker run --rm --network host -v $(pwd)/../results:/app/results webrtc-leak-test

# Test mobile device simulation
docker run --rm --network host -e WEBRTC_MOBILE=true -v $(pwd)/../results:/app/results webrtc-leak-test

# Test Android device simulation
docker run --rm --network host -e WEBRTC_ANDROID=true -v $(pwd)/../results:/app/results webrtc-leak-test
```

**Note**: Using `--network host` ensures the Docker container can detect the Nym VPN connection running on your host machine.

## Results

Test results show detected IPs and leak status:
- **No Leak**: Only Nym VPN IPs detected
- **Leak**: Original IP detected through WebRTC

Results saved to JSON format in `./results/webrtc/`

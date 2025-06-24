#!/usr/bin/env node

const { chromium, devices } = require('playwright');
const fs = require('fs');
const path = require('path');

const DEFAULT_OUTPUT_FILE = './results/webrtc-test-results.json';

class WebRTCLeakTester {
    constructor(options = {}) {
        this.outputFile = options.outputFile || DEFAULT_OUTPUT_FILE;
        this.timeout = options.timeout || 30000;
        this.baselineIP = null;
        this.results = {
            timestamp: new Date().toISOString(),
            test_type: "webrtc_leak_detection",
            baseline_ip: null,
            baseline_ipv6: null,
            detected_ips: {
                ipv4: [],
                ipv6: [],
                all: []
            },
            local_ips: {
                ipv4: [],
                ipv6: []
            },
            public_ips: {
                ipv4: [],
                ipv6: []
            },
            leaked: false,
            leak_details: [],
            test_duration_ms: 0,
            browser_user_agent: null
        };
    }

    async loadBaseline() {
        try {
            const baselinePath = path.resolve(__dirname, '../results/baseline.json');
            if (fs.existsSync(baselinePath)) {
                const baseline = JSON.parse(fs.readFileSync(baselinePath, 'utf8'));
                this.baselineIP = baseline.public_ip;
                this.baselineIPv6 = baseline.ipv6_address;
                this.results.baseline_ip = this.baselineIP;
                this.results.baseline_ipv6 = this.baselineIPv6;
                console.log(`[INFO] Loaded baseline IPv4: ${this.baselineIP}`);
                if (this.baselineIPv6 && this.baselineIPv6 !== 'none') {
                    console.log(`[INFO] Loaded baseline IPv6: ${this.baselineIPv6}`);
                }
            }
        } catch (error) {
            console.warn(`[WARN] Could not load baseline: ${error.message}`);
        }
    }

    async createWebRTCTestPage(page) {
        const webrtcScript = `
            (async function() {
                const ipv4s = new Set();
                const ipv6s = new Set();
                
                // Test both IPv4 and IPv6 with separate RTCPeerConnections
                async function testIPVersion(iceServers, isIPv6 = false) {
                    const pc = new RTCPeerConnection({
                        iceServers: iceServers
                    });

                    const dataChannel = pc.createDataChannel('test');
                    
                    // Test data channel connectivity for real usage simulation
                    dataChannel.onopen = () => {
                        console.log(\`Data channel opened (\${isIPv6 ? 'IPv6' : 'IPv4'})\`);
                        try {
                            dataChannel.send('test-message');
                        } catch (e) {
                            console.log('Data channel send failed:', e.message);
                        }
                    };
                    
                    dataChannel.onmessage = (event) => {
                        console.log(\`Data channel message (\${isIPv6 ? 'IPv6' : 'IPv4'}):\`, event.data);
                    };
                    
                    return new Promise((resolve) => {
                        let timeout;
                        const detectedIPs = new Set();
                        
                        pc.onicecandidate = (event) => {
                            if (event.candidate) {
                                const candidate = event.candidate.candidate;
                                console.log(\`ICE Candidate (\${isIPv6 ? 'IPv6' : 'IPv4'}):\`, candidate);
                                
                                let ipRegex, ips;
                                
                                if (isIPv6) {
                                    // IPv6 regex - to avoid partial matches
                                    ipRegex = /(?:[0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}|(?:[0-9a-fA-F]{1,4}:){1,7}:|(?:[0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|(?:[0-9a-fA-F]{1,4}:){1,5}(?::[0-9a-fA-F]{1,4}){1,2}|(?:[0-9a-fA-F]{1,4}:){1,4}(?::[0-9a-fA-F]{1,4}){1,3}|(?:[0-9a-fA-F]{1,4}:){1,3}(?::[0-9a-fA-F]{1,4}){1,4}|(?:[0-9a-fA-F]{1,4}:){1,2}(?::[0-9a-fA-F]{1,4}){1,5}|[0-9a-fA-F]{1,4}:(?::[0-9a-fA-F]{1,4}){1,6}|:(?::[0-9a-fA-F]{1,4}){1,7}|::(?:[0-9a-fA-F]{1,4}:){0,6}[0-9a-fA-F]{1,4}|::1|::/g;
                                    ips = candidate.match(ipRegex);
                                    
                                    // filter out incomplete addresses
                                    if (ips) {
                                        ips = ips.filter(ip => {
                                            // Must contain at least 2 colons for IPv6 (except ::1 and ::)
                                            const colonCount = (ip.match(/:/g) || []).length;
                                            if (colonCount < 2 && ip !== '::1' && ip !== '::') return false;
                                            
                                            // minimum reasonable IPv6 length
                                            if (ip.length < 4) return false;
                                            
                                            return /^[0-9a-fA-F:]+$/.test(ip);
                                        });
                                    }
                                    
                                    // Check for mDNS (.local) addresses
                                    const mdnsRegex = /[a-zA-Z0-9-]+\\.local/g;
                                    const mdnsAddresses = candidate.match(mdnsRegex);
                                    if (mdnsAddresses) {
                                        console.log('mDNS address detected:', mdnsAddresses);
                                        ips = (ips || []).concat(mdnsAddresses);
                                    }
                                } else {
                                    // IPv4 regex
                                    ipRegex = /(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)/g;
                                    ips = candidate.match(ipRegex);
                                    
                                    // Check for mDNS (.local) addresses
                                    const mdnsRegex = /[a-zA-Z0-9-]+\\.local/g;
                                    const mdnsAddresses = candidate.match(mdnsRegex);
                                    if (mdnsAddresses) {
                                        console.log('mDNS address detected:', mdnsAddresses);
                                        ips = (ips || []).concat(mdnsAddresses);
                                    }
                                }
                                
                                // Check candidate type for additional analysis
                                if (candidate.includes('typ host')) {
                                    console.log('Host candidate detected (direct IP exposure):', candidate);
                                } else if (candidate.includes('typ relay')) {
                                    console.log('Relay candidate detected (via TURN):', candidate);
                                } else if (candidate.includes('typ srflx')) {
                                    console.log('Server reflexive candidate detected (via STUN):', candidate);
                                }
                                
                                if (ips) {
                                    ips.forEach(ip => {
                                        if (isIPv6) {
                                            // Filter out loopback and link-local IPv6, and validate format
                                            if (!ip.startsWith('::1') && 
                                                !ip.startsWith('fe80:') && 
                                                !ip.startsWith('::ffff:') && 
                                                ip.includes(':') &&
                                                ip.length >= 6 && // Minimum reasonable IPv6 length
                                                (ip.match(/:/g) || []).length >= 2) { // At least 2 colons
                                                detectedIPs.add(ip);
                                                ipv6s.add(ip);
                                                console.log('Detected IPv6:', ip);
                                            }
                                        } else {
                                            // Filter out loopback and link-local IPv4
                                            if (!ip.startsWith('127.') && 
                                                !ip.startsWith('169.254.') && 
                                                !ip.includes(':') &&
                                                ip.split('.').length === 4) {
                                                detectedIPs.add(ip);
                                                ipv4s.add(ip);
                                                console.log('Detected IPv4:', ip);
                                            }
                                        }
                                    });
                                }
                            }
                        };

                        pc.onicegatheringstatechange = () => {
                            console.log(\`ICE Gathering State (\${isIPv6 ? 'IPv6' : 'IPv4'}):\`, pc.iceGatheringState);
                            if (pc.iceGatheringState === 'complete') {
                                clearTimeout(timeout);
                                resolve(Array.from(detectedIPs));
                            }
                        };

                        pc.createOffer().then(offer => {
                            return pc.setLocalDescription(offer);
                        }).catch(console.error);

                        // Timeout after 10 seconds per test
                        timeout = setTimeout(() => {
                            console.log(\`WebRTC \${isIPv6 ? 'IPv6' : 'IPv4'} test timeout\`);
                            resolve(Array.from(detectedIPs));
                        }, 10000);
                    });
                }
                
                // IPv4 STUN servers
                const ipv4StunServers = [
                    { urls: 'stun:stun.l.google.com:19302' },
                    { urls: 'stun:stun1.l.google.com:19302' },
                    { urls: 'stun:stun.cloudflare.com:3478' },
                    { urls: 'stun:stun.stunprotocol.org:3478' },
                    { urls: 'stun:stun.ekiga.net:3478' },
                    { urls: 'stun:stun.ideasip.com:3478' }
                ];
                
                // IPv6 STUN servers
                const ipv6StunServers = [
                    { urls: 'stun:stun.l.google.com:19302' },
                    { urls: 'stun:[2001:4860:4860::8888]:19302' },
                    { urls: 'stun:[2606:4700:4700::1111]:3478' },
                    { urls: 'stun:[2001:4860:4860::8844]:19302' }
                ];
                
                // Test TURN servers for additional leak vectors (with dummy credentials)
                const turnServers = [
                    { urls: 'turn:stun.l.google.com:19302', username: 'test', credential: 'test' }
                ];
                
                // Test IPv4 with STUN
                console.log('Starting IPv4 WebRTC test...');
                const ipv4Results = await testIPVersion(ipv4StunServers, false);
                
                // Test IPv6 with STUN
                console.log('Starting IPv6 WebRTC test...');
                const ipv6Results = await testIPVersion(ipv6StunServers, true);
                
                // Test with TURN servers for additional coverage
                console.log('Testing TURN servers...');
                try {
                    await testIPVersion(turnServers, false);
                } catch (e) {
                    console.log('TURN test failed (expected with dummy credentials):', e.message);
                }
                
                return {
                    ipv4: Array.from(ipv4s),
                    ipv6: Array.from(ipv6s),
                    all: [...Array.from(ipv4s), ...Array.from(ipv6s)]
                };
            })();
        `;

        return await page.evaluate(webrtcScript);
    }

    categorizeIPs(detectedIPs) {
        const localIPs = { ipv4: [], ipv6: [] };
        const publicIPs = { ipv4: [], ipv6: [] };

        detectedIPs.ipv4.forEach(ip => {
            if (this.isPrivateIP(ip)) {
                localIPs.ipv4.push(ip);
            } else {
                publicIPs.ipv4.push(ip);
            }
        });

        detectedIPs.ipv6.forEach(ip => {
            if (this.isPrivateIPv6(ip)) {
                localIPs.ipv6.push(ip);
            } else {
                publicIPs.ipv6.push(ip);
            }
        });

        return { localIPs, publicIPs };
    }

    isPrivateIP(ip) {
        const parts = ip.split('.').map(Number);
        
        // Private IP ranges:
        // 10.0.0.0/8
        if (parts[0] === 10) return true;
        
        // 172.16.0.0/12
        if (parts[0] === 172 && parts[1] >= 16 && parts[1] <= 31) return true;
        
        // 192.168.0.0/16
        if (parts[0] === 192 && parts[1] === 168) return true;
        
        return false;
    }

    isPrivateIPv6(ip) {
        // IPv6 private/local ranges
        const lower = ip.toLowerCase();
        
        // Link-local addresses (fe80::/10)
        if (lower.startsWith('fe80:')) return true;
        
        // Unique local addresses (fc00::/7)
        if (lower.startsWith('fc') || lower.startsWith('fd')) return true;
        
        // Site-local addresses (deprecated, fec0::/10)
        if (lower.startsWith('fec0:')) return true;
        
        // Loopback (::1)
        if (lower === '::1') return true;
        
        // IPv4-mapped IPv6 addresses (::ffff:0:0/96)
        if (lower.startsWith('::ffff:')) return true;
        
        return false;
    }

    analyzeLeaks(detectedIPs) {
        const { localIPs, publicIPs } = this.categorizeIPs(detectedIPs);
        
        this.results.local_ips = localIPs;
        this.results.public_ips = publicIPs;
        this.results.detected_ips = detectedIPs;

        let leaked = false;
        const leakDetails = [];

        if (this.baselineIP) {
            publicIPs.ipv4.forEach(ip => {
                if (ip === this.baselineIP) {
                    leaked = true;
                    leakDetails.push(`IPv4 public IP leak detected: ${ip} matches baseline IP`);
                }
            });
        }

        if (this.baselineIPv6 && this.baselineIPv6 !== 'none') {
            publicIPs.ipv6.forEach(ip => {
                if (ip === this.baselineIPv6) {
                    leaked = true;
                    leakDetails.push(`IPv6 public IP leak detected: ${ip} matches baseline IPv6`);
                }
            });
        }

        if (localIPs.ipv4.length > 0) {
            leakDetails.push(`Local IPv4 address(es) exposed via WebRTC: ${localIPs.ipv4.join(', ')}`);
        }
        
        if (localIPs.ipv6.length > 0) {
            leakDetails.push(`Local IPv6 address(es) exposed via WebRTC: ${localIPs.ipv6.join(', ')}`);
        }

        // Report different public IPs (expected for VPN)
        if (publicIPs.ipv4.length > 0 && this.baselineIP) {
            publicIPs.ipv4.forEach(ip => {
                if (ip !== this.baselineIP) {
                    leakDetails.push(`Different IPv4 public IP detected: ${ip} (expected if using VPN)`);
                }
            });
        }
        
        if (publicIPs.ipv6.length > 0 && this.baselineIPv6 && this.baselineIPv6 !== 'none') {
            publicIPs.ipv6.forEach(ip => {
                if (ip !== this.baselineIPv6) {
                    leakDetails.push(`Different IPv6 public IP detected: ${ip} (expected if using VPN)`);
                }
            });
        }

        this.results.leaked = leaked;
        this.results.leak_details = leakDetails;

        return { leaked, leakDetails, localIPs, publicIPs };
    }

    async runTest() {
        const startTime = Date.now();
        console.log('[INFO] Starting WebRTC leak detection test...');

        await this.loadBaseline();

        let browser;
        try {
            browser = await chromium.launch({
                headless: true,
                args: [
                    '--no-sandbox',
                    '--disable-setuid-sandbox',
                    '--disable-dev-shm-usage',
                    '--disable-background-timer-throttling',
                    '--disable-backgrounding-occluded-windows',
                    '--disable-renderer-backgrounding',
                    '--disable-web-security',
                    '--allow-running-insecure-content'
                ]
            });

            // Mobile device simulation support
            const isMobile = process.env.WEBRTC_MOBILE === 'true';
            const isAndroid = process.env.WEBRTC_ANDROID === 'true';
            
            let device = null;
            if (isMobile) {
                device = devices['iPhone 15 Pro'];
            } else if (isAndroid) {
                device = devices['Pixel 7'];
            }

            const contextOptions = {
                permissions: ['camera', 'microphone'],
                ...(device && device)
            };

            if ((isMobile || isAndroid) && !device) {
                if (isAndroid) {
                    contextOptions.userAgent = 'Mozilla/5.0 (Linux; Android 14; Pixel 7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36';
                    contextOptions.viewport = { width: 412, height: 915 };
                } else {
                    contextOptions.userAgent = 'Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1';
                    contextOptions.viewport = { width: 393, height: 852 };
                }
                contextOptions.isMobile = true;
                contextOptions.hasTouch = true;
            }

            const context = await browser.newContext(contextOptions);

            const page = await context.newPage();
            
            this.results.browser_user_agent = await page.evaluate(() => navigator.userAgent);

            console.log('[INFO] Running WebRTC IP detection...');
            
            const detectedIPs = await this.createWebRTCTestPage(page);
            
            console.log(`[INFO] Detected IPv4 addresses (${detectedIPs.ipv4.length}):`, detectedIPs.ipv4);
            console.log(`[INFO] Detected IPv6 addresses (${detectedIPs.ipv6.length}):`, detectedIPs.ipv6);

            const analysis = this.analyzeLeaks(detectedIPs);
            
            this.results.test_duration_ms = Date.now() - startTime;

            console.log('[INFO] WebRTC test completed');
            console.log(`[INFO] Local IPv4 IPs: ${analysis.localIPs.ipv4.join(', ') || 'None'}`);
            console.log(`[INFO] Local IPv6 IPs: ${analysis.localIPs.ipv6.join(', ') || 'None'}`);
            console.log(`[INFO] Public IPv4 IPs: ${analysis.publicIPs.ipv4.join(', ') || 'None'}`);
            console.log(`[INFO] Public IPv6 IPs: ${analysis.publicIPs.ipv6.join(', ') || 'None'}`);
            console.log(`[INFO] Leaked: ${analysis.leaked}`);

            if (analysis.leaked) {
                console.log('[WARN] WebRTC leaks detected!');
                analysis.leakDetails.forEach(detail => console.log(`[WARN] ${detail}`));
            }

        } catch (error) {
            console.error('[ERROR] WebRTC test failed:', error.message);
            this.results.error = error.message;
        } finally {
            if (browser) {
                await browser.close();
            }
        }

        return this.results;
    }

    async saveResults() {
        try {
            const dir = path.dirname(this.outputFile);
            if (!fs.existsSync(dir)) {
                fs.mkdirSync(dir, { recursive: true });
            }
            
            fs.writeFileSync(this.outputFile, JSON.stringify(this.results, null, 2));
            console.log(`[INFO] Results saved to: ${this.outputFile}`);
        } catch (error) {
            console.error('[ERROR] Failed to save results:', error.message);
        }
    }
}

async function main() {
    const args = process.argv.slice(2);
    const outputFile = args.find(arg => arg.startsWith('--output-file='))?.split('=')[1];
    
    const tester = new WebRTCLeakTester({ outputFile });
    
    try {
        const results = await tester.runTest();
        await tester.saveResults();
        
        process.exit(results.leaked ? 1 : 0);
    } catch (error) {
        console.error('[ERROR] Test failed:', error.message);
        process.exit(2);
    }
}

if (require.main === module) {
    main();
}

module.exports = WebRTCLeakTester; 
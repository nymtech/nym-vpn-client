#!/usr/bin/env python3
"""Minimal HTTP stub for local E2E of fairUsage.dataUnavailable.

Serves plain HTTP (no TLS) so the client does not need mitmproxy CA trust.

Usage:
  python3 dev_mock_vpn_api_data_unavailable.py [port]

Automated coverage lives in `nym-vpn-api-client/tests/account_summary_data_unavailable.rs` (wiremock). Use this script only for manual debugging against a real HTTP client.

Default port 18080. Override with STUB_PORT or first CLI arg.

Paths (base URL must end with /api/, e.g. http://127.0.0.1:18080/api/):
  GET .../public/v1/health
  GET .../public/v1/account/<id>/summary
  GET .../public/v1/account/<id>/device/<key>/summary

Any other GET returns 404.

Mitmproxy: for HTTPS APIs you would instead intercept the same paths and rewrite
the JSON body; reqwest typically honors HTTPS_PROXY and system trust after you
install mitmproxy's CA.
"""

from __future__ import annotations

import json
import sys
from http.server import BaseHTTPRequestHandler, HTTPServer


def summary_body() -> dict:
    """Shape matches NymVpnAccountSummaryWithDeviceResponse (flattened fields).

    Nested subscription uses snake_case keys to match serde on ``NymVpnSubscription``.
    Omit optional ``pending`` entirely when absent (same as typical API payloads).
    """
    return {
        "account": {
            "created_on_utc": "2026-05-04T11:15:14.681Z",
            "last_updated_utc": "2026-05-04T11:15:14.681Z",
            "account_addr": "n1stubstubstubstubstubstubstubstubst",
            "status": "active",
            "canonical_account_addr": "n1stubstubstubstubstubstubstubstubst",
            "auth_methods": [
                {
                    "id": "stub",
                    "pubkey": "stub",
                    "kind": "user_generated_secp256k1",
                    "label": "Stub",
                    "status": "active",
                    "created": "2026-05-04T11:15:14.727Z",
                }
            ],
        },
        "subscription": {
            "isActive": True,
            "isStacked": False,
            "active": {
                "created_on_utc": "2026-05-04T11:15:51.388Z",
                "last_updated_utc": "2026-05-04T11:15:58.581Z",
                "id": "stub-sub",
                "valid_until_utc": "2026-06-04T11:15:51.000Z",
                "valid_from_utc": "2026-05-04T11:15:51.000Z",
                "status": "active",
                "kind": "one_month",
                "isRecurring": True,
            },
        },
        "devices": {"active": 1, "max": 10, "remaining": 9},
        "fairUsage": {
            "usedGB": 0,
            "limitGB": 0,
            "dataUnavailable": True,
        },
        "activeDevice": {
            "created_on_utc": "2026-05-04T15:11:56.013Z",
            "last_updated_utc": "2026-05-04T15:11:56.013Z",
            "device_identity_key": "FJDUECYAeosXhNGjxf8w5MJM7N2DfDwQznvWwTxJz6ft",
            "status": "active",
        },
    }


class Handler(BaseHTTPRequestHandler):
    def log_message(self, fmt: str, *args: object) -> None:
        sys.stderr.write("%s - %s\n" % (self.address_string(), fmt % args))

    def do_GET(self) -> None:  # noqa: N802
        path = self.path.split("?", 1)[0]
        if path.rstrip("/").endswith("/health"):
            body = json.dumps(
                {"status": "ok", "timestampUtc": "2026-05-04T12:00:00.000Z"},
            )
        elif "/account/" in path and path.rstrip("/").endswith("/summary"):
            body = json.dumps(summary_body())
        else:
            self.send_response(404)
            self.end_headers()
            self.wfile.write(b'{"error":"stub: path not supported"}\n')
            return

        data = body.encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)


def main() -> None:
    port = 18080
    if len(sys.argv) > 1:
        port = int(sys.argv[1])
    env_p = __import__("os").environ.get("STUB_PORT")
    if env_p:
        port = int(env_p)
    server = HTTPServer(("127.0.0.1", port), Handler)
    print(f"stub listening on http://127.0.0.1:{port}/api/", flush=True)
    server.serve_forever()


if __name__ == "__main__":
    main()

#!/usr/bin/env bash
# Drop guest egress UDP to addr:port (IPv4/IPv6). Same CLI shape as ip_block.sh.
# Linux/iptables only - callers must target Linux guests (see bridge_tests target_os).

set -euo pipefail

if ! command -v iptables >/dev/null 2>&1; then
    printf "udp_block.sh requires iptables (Linux). Not found on PATH.\n" >&2
    exit 1
fi

function block_addr_port_v4 {
    local addr="$1"
    local port="$2"
    printf "b udp %s %s\n" "$addr" "$port"
    iptables -A OUTPUT -p udp -d "$addr" --dport "$port" -j DROP
}

function block_addr_port_v6 {
    local addr="$1"
    local port="$2"
    printf "b udp %s %s\n" "$addr" "$port"
    ip6tables -A OUTPUT -p udp -d "$addr" --dport "$port" -j DROP
}

function unblock_addr_port_v4 {
    local addr="$1"
    local port="$2"
    printf "u udp %s %s\n" "$addr" "$port"
    # Tolerate missing rules so cleanup after a partial block attempt can finish.
    iptables -D OUTPUT -p udp -d "$addr" --dport "$port" -j DROP 2>/dev/null || true
}

function unblock_addr_port_v6 {
    local addr="$1"
    local port="$2"
    printf "u udp %s %s\n" "$addr" "$port"
    ip6tables -D OUTPUT -p udp -d "$addr" --dport "$port" -j DROP 2>/dev/null || true
}

function process_socket_addresses {
    local action=$1
    shift
    for socket_addr in "$@"; do
        if [[ $socket_addr =~ ^\[(.+)\]:([0-9]+)$ ]]; then
            local addr="${BASH_REMATCH[1]}"
            local port="${BASH_REMATCH[2]}"
            if [[ $action == "block" ]]; then
                block_addr_port_v6 "$addr" "$port"
            elif [[ $action == "unblock" ]]; then
                unblock_addr_port_v6 "$addr" "$port"
            fi
        elif [[ $socket_addr =~ ^([^:]+):([0-9]+)$ ]]; then
            local addr="${BASH_REMATCH[1]}"
            local port="${BASH_REMATCH[2]}"
            if [[ $action == "block" ]]; then
                block_addr_port_v4 "$addr" "$port"
            elif [[ $action == "unblock" ]]; then
                unblock_addr_port_v4 "$addr" "$port"
            fi
        else
            printf "Invalid socket address format: %s\n" "$socket_addr" >&2
            return 1
        fi
    done
}

case $1 in
    "block" )
        if [[ $# -gt 1 ]]; then
            shift
            process_socket_addresses "block" "$@"
        else
            printf "usage: udp_block.sh block <addr:port>...\n" >&2
            exit 1
        fi
        ;;
    "unblock" )
        if [[ $# -gt 1 ]]; then
            shift
            process_socket_addresses "unblock" "$@"
        else
            printf "usage: udp_block.sh unblock <addr:port>...\n" >&2
            exit 1
        fi
        ;;
    * )
        printf "usage: udp_block.sh block|unblock <addr:port>...\n" >&2
        exit 1
        ;;
esac

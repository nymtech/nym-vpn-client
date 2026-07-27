#!/usr/bin/env bash

# Allow connections for the IP address of `validator.nymtech.net` to send 36kB then drop traffic after.
# Keep stdout quiet: noisy script output is fine for ExecResult, but avoid anything that could
# confuse operators correlating with serial mux logs. Errors stay on stderr.

function block_addr_port_v4 {
    local addr="$1"
    local port="$2"

    iptables -A OUTPUT -p tcp -d "$addr" --dport "$port" -j DROP
    iptables -A INPUT -p tcp --sport "$port" -s "$addr" -m connbytes --connbytes 0:36000 --connbytes-mode bytes --connbytes-dir reply -j ACCEPT
    iptables -A INPUT -p tcp --sport "$port" -s "$addr" -j DROP

    return 0
}

function block_addr_port_v6 {
    local addr="$1"
    local port="$2"

    ip6tables -A OUTPUT -p tcp -d "$addr" --dport "$port" -j DROP
    ip6tables -A INPUT -p tcp --sport "$port" -s "$addr" -m connbytes --connbytes 0:36000 --connbytes-mode bytes --connbytes-dir reply -j ACCEPT
    ip6tables -A INPUT -p tcp --sport "$port" -s "$addr" -j DROP

    return 0
}

function unblock_addr_port_v4 {
    local addr="$1"
    local port="$2"

    iptables -D OUTPUT -p tcp -d "$addr" --dport "$port" -j DROP
    iptables -D INPUT -p tcp --sport "$port" -s "$addr" -m connbytes --connbytes 0:36000 --connbytes-mode bytes --connbytes-dir reply -j ACCEPT
    iptables -D INPUT -p tcp --sport "$port" -s "$addr" -j DROP

    return 0
}

function unblock_addr_port_v6 {
    local addr="$1"
    local port="$2"

    ip6tables -D OUTPUT -p tcp -d "$addr" --dport "$port" -j DROP
    ip6tables -D INPUT -p tcp --sport "$port" -s "$addr" -m connbytes --connbytes 0:36000 --connbytes-mode bytes --connbytes-dir reply -j ACCEPT
    ip6tables -D INPUT -p tcp --sport "$port" -s "$addr" -j DROP

    return 0
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
            printf "Invalid socket address format: %s (expected IPv4:port or [IPv6]:port)\n" "$socket_addr" >&2
            return 1
        fi
    done

    return 0
}

case $1 in
    "block" )
        if [[ $# -gt 1 ]]; then
            process_socket_addresses "$@"
        fi
        ;;

    "unblock" )
        if [[ $# -gt 1 ]]; then
            process_socket_addresses "$@"
        fi
        ;;
    *)
        printf "unknown arg %s\n" "$1" >&2
        exit 1
        ;;
esac

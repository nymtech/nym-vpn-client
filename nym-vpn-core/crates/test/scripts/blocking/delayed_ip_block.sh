#!/usr/bin/env bash

# Allow connections for the IP address of `validator.nymtech.net` to send 36kB then drop traffic after


function block_addr_port_v4 {
    local addr="$1"
    local port="$2"
    
    printf "b %s %s\n" "$addr" "$port"
    iptables -A OUTPUT -p tcp -d "$addr" --dport "$port" -j DROP
	iptables -A INPUT -p tcp --sport "$port" -s "$addr"  -m connbytes --connbytes 0:36000 --connbytes-mode bytes --connbytes-dir reply -j ACCEPT
	iptables -A INPUT -p tcp --sport "$port" -s "$addr"  -j DROP
	
	return 0
}

function block_addr_port_v6 {
    local addr="$1"
    local port="$2"
    
    printf "b %s %s\n" "$addr" "$port"
	ip6tables -A INPUT -p tcp --sport "$port" -s "$addr"  -m connbytes --connbytes 0:36000 --connbytes-mode bytes --connbytes-dir reply -j ACCEPT
	ip6tables -A INPUT -p tcp --sport "$port" -s "$addr"  -j DROP
	
	return 0
}

function unblock_addr_port_v4 {
    local addr="$1"
    local port="$2"
    
    printf "u %s %s\n" "$addr" "$port"
	iptables -D INPUT -p tcp --sport "$port" -s "$addr"  -m connbytes --connbytes 0:36000 --connbytes-mode bytes --connbytes-dir reply -j ACCEPT
	iptables -D INPUT -p tcp --sport "$port" -s "$addr"  -j DROP
	
	return 0
}

function unblock_addr_port_v6 {
    local addr="$1"
    local port="$2"
    
    printf "u %s %s\n" "$addr" "$port"
    ip6tables -D OUTPUT -p tcp -d "$addr" --dport "$port" -j DROP
	ip6tables -D INPUT -p tcp --sport "$port" -s "$addr"  -m connbytes --connbytes 0:36000 --connbytes-mode bytes --connbytes-dir reply -j ACCEPT
	ip6tables -D INPUT -p tcp --sport "$port" -s "$addr"  -j DROP
	
	return 0
}

function process_socket_addresses {
    local action=$1
    shift # Remove the first argument (action)
    
    for socket_addr in "$@"; do
        # Check if it's IPv6 format [address]:port
        if [[ $socket_addr =~ ^\[(.+)\]:([0-9]+)$ ]]; then
            local addr="${BASH_REMATCH[1]}"
            local port="${BASH_REMATCH[2]}"
            printf "Processing IPv6 socket address: %s:%s\n" "$addr" "$port"
            if [[ $action == "block" ]]; then
                block_addr_port_v6 "$addr" "$port"
            elif [[ $action == "unblock" ]]; then
                unblock_addr_port_v6 "$addr" "$port"
            fi
        # Check if it's IPv4 format address:port
        elif [[ $socket_addr =~ ^([^:]+):([0-9]+)$ ]]; then
            local addr="${BASH_REMATCH[1]}"
            local port="${BASH_REMATCH[2]}"
            printf "Processing IPv4 socket address: %s:%s\n" "$addr" "$port"
            if [[ $action == "block" ]]; then
                block_addr_port_v4 "$addr" "$port"
            elif [[ $action == "unblock" ]]; then
                unblock_addr_port_v4 "$addr" "$port"
            fi
        else
            printf "Invalid socket address format: %s (expected IPv4:port or [IPv6]:port)\n" "$socket_addr"
            return 1
        fi
    done
    
    return 0
}

case $1 in
    "block" )
        if [[ $# -gt 1 ]]; then
            printf "blocking specified addresses... \n"
            process_socket_addresses "$@"
        fi
        ;;

    "unblock" )
        if [[ $# -gt 1 ]]; then
            printf "unblocking specified addresses... \n"
            process_socket_addresses "$@"
        fi
        ;;
    *)
        printf "unknown arg $1\n"; ;;
esac


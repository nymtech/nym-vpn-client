#!/usr/bin/env bash

function block_addr_port_v4 {
    printf "b %s %s\n" $1 $2
    iptables -A OUTPUT -p tcp -d $1 --dport $2 -j DROP
}

function block_addr_port_v6 {
    printf "b %s %s\n" $1 $2
    ip6tables -A OUTPUT -p tcp -d $1 --dport $2 -j DROP
}

function unblock_addr_port_v4 {
    printf "u %s %s\n" $1 $2
    iptables -D OUTPUT -p tcp -d $1 --dport $2 -j DROP
}

function unblock_addr_port_v6 {
    printf "u %s %s\n" $1 $2         
    ip6tables -D OUTPUT -p tcp -d $1 --dport $2 -j DROP
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
        fi
    done
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


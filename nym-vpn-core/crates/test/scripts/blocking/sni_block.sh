#!/usr/bin/env bash

function block_sni {
    local domain=$1
    local port=$2
    
    if [[ -n "$port" ]]; then
        printf "b %s %s\n" "$domain" "$port"
        iptables -A OUTPUT -p tcp --dport "$port" -m string --string "$domain" --algo kmp -j DROP
        ip6tables -A OUTPUT -p tcp --dport "$port" -m string --string "$domain" --algo kmp -j DROP
    else
        printf "b %s (all ports)\n" "$domain"
        iptables -A OUTPUT -p tcp -m string --string "$domain" --algo kmp -j DROP
        ip6tables -A OUTPUT -p tcp -m string --string "$domain" --algo kmp -j DROP
    fi
    
    return 0
}

function unblock_sni {
    local domain=$1
    local port=$2
    
    if [[ -n "$port" ]]; then
        printf "u %s %s\n" "$domain" "$port"
        iptables -D OUTPUT -p tcp --dport "$port" -m string --string "$domain" --algo kmp -j DROP
        ip6tables -D OUTPUT -p tcp --dport "$port" -m string --string "$domain" --algo kmp -j DROP
    else
        printf "u %s (all ports)\n" "$domain"
        iptables -D OUTPUT -p tcp -m string --string "$domain" --algo kmp -j DROP
        ip6tables -D OUTPUT -p tcp -m string --string "$domain" --algo kmp -j DROP
    fi
    
    return 0
}

function process_domain_addresses {
    local action=$1
    shift # Remove the first argument (action)
    
    for domain_addr in "$@"; do
        # Check if it contains a port (domain:port format)
        if [[ $domain_addr =~ ^([^:]+):([0-9]+)$ ]]; then
            local domain="${BASH_REMATCH[1]}"
            local port="${BASH_REMATCH[2]}"
            printf "Processing domain with port: %s:%s\n" "$domain" "$port"
            if [[ $action == "block" ]]; then
                block_sni "$domain" "$port"
            elif [[ $action == "unblock" ]]; then
                unblock_sni "$domain" "$port"
            fi
        # Domain without port - no port restriction
        elif [[ $domain_addr =~ ^[a-zA-Z0-9.-]+$ ]]; then
            local domain="$domain_addr"
            printf "Processing domain (any port): %s\n" "$domain"
            if [[ $action == "block" ]]; then
                block_sni "$domain" ""
            elif [[ $action == "unblock" ]]; then
                unblock_sni "$domain" ""
            fi
        else
            printf "Invalid domain format: %s (expected domain.com or domain.com:port)\n" "$domain_addr"
            return 1
        fi
    done
    
    return 0
}

case $1 in
    "block" )
        if [[ $# -gt 1 ]]; then
            printf "blocking specified domains... \n"
            process_domain_addresses "$@"
        else
            printf "No domains specified for blocking\n"
        fi
        ;;

    "unblock" )
        if [[ $# -gt 1 ]]; then
            printf "unblocking specified domains... \n"
            process_domain_addresses "$@"
        else
            printf "No domains specified for unblocking\n"
        fi
        ;;
    *)
        printf "Usage: $0 {block|unblock} domain1[:port] [domain2[:port] ...]\n"
        printf "Examples:\n"
        printf "  $0 block example.com\n"
        printf "  $0 block example.com:443 domain.example.com\n"
        printf "  $0 unblock example.org:8443\n"
        ;;
esac
#!/bin/bash
# Network setup script for slideshow-stack
# This creates the bridge network required for inter-service communication

set -e

NETWORK_NAME="slideshow-net"
NETWORK_SUBNET="10.0.100.0/24"
ZENOH_PORT="7447"

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*"
}

check_network() {
    if ip link show "$NETWORK_NAME" &>/dev/null; then
        local state=$(ip -brief link show "$NETWORK_NAME" 2>/dev/null | awk '{print $2}')
        if [[ "$state" == "UP" ]]; then
            local ip_addr=$(ip -4 addr show "$NETWORK_NAME" 2>/dev/null | grep -oP 'inet \K[\d.]+' | head -1)
            if [[ -n "$ip_addr" ]]; then
                return 0
            fi
        fi
    fi
    return 1
}

create_network() {
    if check_network; then
        log "Network '$NETWORK_NAME' is already operational"
        return 0
    fi

    log "Creating bridge network: $NETWORK_NAME ($NETWORK_SUBNET)"

    if ip link show "$NETWORK_NAME" &>/dev/null; then
        log "Cleaning up existing interface $NETWORK_NAME"
        ip link set "$NETWORK_NAME" down 2>/dev/null || true
        ip link delete "$NETWORK_NAME" type bridge 2>/dev/null || true
        sleep 1
    fi

    ip link add name "$NETWORK_NAME" type bridge 2>/dev/null
    ip addr add "$NETWORK_SUBNET" dev "$NETWORK_NAME" 2>/dev/null
    ip link set "$NETWORK_NAME" up

    sleep 1

    if check_network; then
        local ip_addr=$(ip -4 addr show "$NETWORK_NAME" | grep -oP 'inet \K[\d.]+' | head -1)
        log "Bridge created successfully: $NETWORK_NAME at $ip_addr"

        sysctl -w net.ipv4.conf.$NETWORK_NAME.forwarding=1 2>/dev/null || true
        return 0
    else
        log "ERROR: Failed to create operational bridge"
        return 1
    fi
}

destroy_network() {
    if ip link show "$NETWORK_NAME" &>/dev/null; then
        log "Destroying bridge network: $NETWORK_NAME"
        ip link set "$NETWORK_NAME" down 2>/dev/null || true
        ip link delete "$NETWORK_NAME" type bridge 2>/dev/null || true
        log "Bridge destroyed"
    else
        log "Network '$NETWORK_NAME' does not exist"
    fi
}

status() {
    if check_network; then
        local ip_addr=$(ip -4 addr show "$NETWORK_NAME" | grep -oP 'inet \K[\d.]+' | head -1)
        echo "Network '$NETWORK_NAME' is UP with IP $ip_addr"
        ip addr show "$NETWORK_NAME"
        echo ""
        echo "Testing Zenoh connectivity..."
        if timeout 2 bash -c "echo > /dev/udp/$ip_addr/$ZENOH_PORT" 2>/dev/null; then
            echo "Zenoh port $ZENOH_PORT is reachable"
        else
            echo "Warning: Zenoh port $ZENOH_PORT not reachable (may be normal if no service listening)"
        fi
    else
        echo "Network '$NETWORK_NAME' is not operational"
        return 1
    fi
}

case "$1" in
    start)
        create_network
        ;;
    stop)
        destroy_network
        ;;
    restart)
        destroy_network
        sleep 1
        create_network
        ;;
    status)
        status
        ;;
    *)
        echo "Usage: $0 {start|stop|restart|status}"
        exit 1
        ;;
esac
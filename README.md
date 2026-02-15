# Ping Implementation

A custom ping implementation written in Rust that demonstrates low-level network programming using raw sockets and ICMP protocol.

## Features

### Implemented
- **ICMP Echo Request**: Sends ICMP echo request packets from scratch
- **IPv4 Support**: Full IPv4 address support
- **Round-Trip Time (RTT) Measurement**: Calculates and displays response time in milliseconds
- **TTL Extraction**: Shows Time-To-Live value from received packets
- **Packet Statistics**: Displays sent, received, and lost packet counts with loss percentage
- **Cross-Platform**: Supports both Windows and Unix-like systems
- **Configurable Packet Count**: Specify number of packets to send via CLI

### Future Features (Potential)
- **IPv6 Support**: Add ICMPv6 echo request support
- **Continuous Ping Mode**: Add option for continuous ping (like traditional `ping -t`)
- **Payload Size Configuration**: Allow custom payload sizes
- **Interval Configuration**: Configurable time between packets
- **Timeout Configuration**: Custom timeout settings
- **Statistics Aggregation**: Min/Max/Avg RTT calculations
- **Timestamp Precision**: High-precision timestamps using `Instant`
- **Graceful Handling**: Handle permission errors more gracefully (raw sockets require admin/root)

## Building

```bash
cargo build --release
```

## Usage

```bash
# Ping an IP address (default: 8.8.8.8)
cargo run --release -- 192.168.1.1

# Specify number of packets
cargo run --release -- -n 10 8.8.8.8
```

## Requirements

- **Windows**: Administrator privileges (raw sockets)
- **Unix/Linux**: Root or CAP_NET_RAW capability

## How It Works

1. Creates a raw socket using Windows API (WinSock) on Windows, or libc on Unix
2. Builds ICMP echo request packets manually (type=8, code=0)
3. Calculates ICMP checksum
4. Sends packets and waits for responses
5. Parses IP header to extract TTL and source address
6. Calculates RTT using high-resolution timers
7. Reports statistics at the end

## Architecture

- `src/main.rs`: CLI argument parsing
- `src/protocol.rs`: ICMP packet construction and checksum calculation
- `src/sys/mod.rs`: Platform-specific module routing
- `src/sys/win.rs`: Windows raw socket implementation
- `src/sys/unix.rs`: Unix raw socket implementation

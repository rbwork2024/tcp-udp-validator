# TCP-UDP-validator

This crate is designed to test a client and server sending data to each other through TCP or UDP. A warning will be thrown if the SHA256 checksum fails on the recipient, and a `NACK` will be sent back to the sender. The expected response is an `ACK`.

## Building

### Windows, MacOS, Linux x86

```shell
# For Windows, MacOS, Linux, etc.
cargo build --release
```

### Cross compiling for Armv7 embedded

#### Cross method

```shell
# For embedded systems, use cross for simplicity
cargo install cross
# 32-bit armv7, for 64-bit remove the hf at the end
cross build --release --target armv7-unknown-linux-gnueabihf
```

#### Non-cross (haven't tested)

```shell
# Add target, -gnueabi without hf for 64-bit
rustup target add armv7-unknown-linux-gnueabihf
sudo apt-get install gcc-arm-linux-gnueabihf
cargo build --release --target armv7-unknown-linux-gnueabihf
```

## Usage (TCP)

### On the client

```shell
# Use the IP address you'd like to attempt connecting to
usb-validator tcp client 127.0.0.1:8080
```

### On the server
```shell
# Use the IP address you'd like to bind to
tcp-validator tcp server 0.0.0.0:8080
```

## Usage (UDP)

> **NOTE:** UDP is expected to have more data integrity failures since it is a stateless connection-less protocol. TCP has built-in checksums and ordered delivery guarantees. Since the protocol doesn't check for reliable delivery, out-of-order packets and dropped packets can occur! This program currently does not check for out-of-order or dropped packets, but the checksums will check the integrity of the packets themselves.

> **WARNING:** Start the client first! Or else you might get an error like `Error: An existing connection was forcibly closed by the remote host. (os error 10054)`

### On the client

```shell
# Provide the IP address you'd like to bind to
usb-validator udp client 0.0.0.0:8081
```

### On the server
```shell
# Provide the IP address you'd like to bind to, AND the IP address you'd like to connect to
tcp-validator udp server 0.0.0.0:8080 127.0.0.1:8081
```

## Additional arguments

> **Note:** Run `tcp-validator --help` to see the latest arguments, this is probably not out of date but just in case!

```
Simple program to validate data sent through TCP or UDP

Usage: tcp-udp-validator.exe [OPTIONS] <COMMAND>

Commands:
  tcp
  udp
  help  Print this message or the help of the given subcommand(s)

Options:
      --log-level <LOG_LEVEL>  Define a log level (default=info) [possible values: warn, info, debug, trace]
      --abort-on-fail          Abort on failure
  -h, --help                   Print help
  -V, --version                Print version
```

### Help (UDP)

```
Usage: tcp-udp-validator.exe udp <UNIT> <BIND_ADDRESS> [SEND_ADDRESS]

Arguments:
  <UNIT>          Whether to run as server or client (UDP) [possible values: server, client]
  <BIND_ADDRESS>  Bind address for the server/client Example(server): 0.0.0.0:8080, Example(client): 0.0.0.0:8081     
  [SEND_ADDRESS]  Send address for the server. Will be unused for client Example(server): 127.0.0.1:8081

Options:
  -h, --help  Print help
```

### Help (TCP)

```
Usage: tcp-udp-validator.exe tcp <UNIT> <ADDRESS>

Arguments:
  <UNIT>     Whether to run as server or client (TCP) [possible values: server, client]
  <ADDRESS>  Bind address for the server, and connection address for the client Example(server): 0.0.0.0:8080, Example(client): 127.0.0.1:8080

Options:
  -h, --help  Print help
```

## Issues

Report issues at <https://github.com/rbwork2024/tcp-udp-validator/issues/>!
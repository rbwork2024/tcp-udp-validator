# TCP-validator

This crate is designed to test a client and server sending data to each other through TCP. A warning will be thrown if the SHA256 checksum fails on the recipient, and a `NACK` will be sent back to the sender. The expected response is an `ACK`.

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

## Usage

### On the client

```shell
# Use the IP address you'd like to attempt connecting to
usb-validator client 127.0.0.1:8080
```

### On the server
```shell
# Use the IP address you'd like to bind to
tcp-validator server 0.0.0.0:8080
```

## Additional arguments

> **Note:** Run `tcp-validator --help` to see the latest arguments, this is probably not out of date but just in case!

```
Simple program to validate data sent through TCP

Usage: tcp-validator.exe [OPTIONS] <UNIT> <ADDRESS>        

Arguments:
  <UNIT>     Whether to run as server or client [possible values: server, client]
  <ADDRESS>  Bind address for the server, and connection address for the client Example(server): 0.0.0.0:8080, Example(client): 127.0.0.1:8080

Options:
      --log-level <LOG_LEVEL>  Define a log level (default=Warn) [possible values: warn, info, debug, trace]
      --abort-on-fail          Abort on failure
  -h, --help                   Print help
  -V, --version                Print version
```
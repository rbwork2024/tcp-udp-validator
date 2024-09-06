# TCP-validator

This crate is designed to test a client and server sending data to each other through TCP. A warning will be thrown if the SHA256 checksum fails on the recipient, and a `NACK` will be sent back to the sender. The expected response is an `ACK`.

## Usage

### On the recipient

```shell
# Use the IP address you'd like to attempt connecting to
usb-validator receiver 127.0.0.1:8080
```

### On the sender
```shell
# Use the IP address you'd like to bind to
tcp-validator sender 0.0.0.0:8080
```

## Additional arguments

> **Note:** Run `tcp-validator --help` to see the latest arguments, this is probably not out of date but just in case!

```
Simple program to validate data sent through TCP

Usage: tcp-validator.exe [OPTIONS] <UNIT> <ADDRESS>        

Arguments:
  <UNIT>     Whether to run as sender or receiver [possible values: sender, receiver]
  <ADDRESS>  Bind address for the sender, and connection address for the receiver Example(sender): 0.0.0.0:8080, Example(receiver): 127.0.0.1:8080

Options:
      --log-level <LOG_LEVEL>  Define a log level (default=Warn) [possible values: warn, info, debug, trace]
      --abort-on-fail          Abort on failure
  -h, --help                   Print help
  -V, --version                Print version
```
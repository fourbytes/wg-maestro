# wg-maestro

## Info
*This does not work yet, at all.*

## Goals
### Core Functionality
 * Every node has a link-local IPv6 address generated from it's public key. This is fixed and will always return the same IP from a given public key.
 * Support multiple Wireguard implementations, possibly boringtun as a default fallback for devices without the Wireguard kernel module.
 * Centralised server setup, the only information each client should need is; the server's address, the server's public key, it's own private key, and optionally a pre-shared key.
 * All communication should be done over the Wireguard interface.
 * Live reloading of config (on SIGHUP).

### Additional Functionality
 * Admin API, ability to dynamically modify server config.
 * Ansible module for communicating with admin API.
 * Send metrics to Prometheus (connected peers, in, out, latency, etc).


## Development
Since communicating with the Wireguard API requires admin access, testing must generally be done using sudo:
```sh
# Running the server
$ cargo build && sudo ./target/debug/wg-maestro -vv server

# Running the client
$ cargo build && sudo ./target/debug/wg-maestro -vv client
```

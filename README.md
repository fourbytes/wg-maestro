# wg-maestro
## Goals
 * Support multiple Wireguard implementations, possibly boringtun as a default fallback for devices without the Wireguard kernel module.  Centralised server setup, the only information each client should need is; the server's address, the server's public key, it's own private key, and optionally a pre-shared key.
 * Every node has a link-local IPv6 address generated from it's public key. This is fixed and will always return the same IP from a given public key.
 * All communication should be done over the Wireguard interface.
 * Send metrics to Prometheus (connected peers, in, out, latency, etc).

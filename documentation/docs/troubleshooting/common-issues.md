# Common Issues

Solutions for frequently encountered problems with SolanaVault.

---

## Installation Issues

### Build fails with missing dependencies

**Error:**

```
error: linker `cc` not found
```

**Solution:**

Install build tools for your platform:

=== "Ubuntu/Debian"

    ```bash
    sudo apt-get update
    sudo apt-get install -y build-essential pkg-config libssl-dev
    ```

=== "macOS"

    ```bash
    xcode-select --install
    brew install openssl pkg-config
    ```

=== "Fedora"

    ```bash
    sudo dnf install gcc gcc-c++ openssl-devel pkgconfig
    ```

---

### OpenSSL errors

**Error:**

```
Could not find directory of OpenSSL installation
```

**Solution:**

Set OpenSSL path:

```bash
# macOS (Homebrew)
export OPENSSL_DIR=$(brew --prefix openssl)

# Linux
export OPENSSL_DIR=/usr/local/ssl
```

---

### Rust version too old

**Error:**

```
error: package requires rustc 1.70.0 or newer
```

**Solution:**

Update Rust:

```bash
rustup update stable
rustc --version  # Should be 1.70.0+
```

---

## Connection Issues

### Cannot connect to gateway

**Error:**

```
Error: Failed to connect to gateway: Connection refused
```

**Solutions:**

1. **Check gateway endpoint**:
   ```bash
   # Verify endpoint is reachable
   nc -zv gateway.solanavault.com 5050
   ```

2. **Check firewall**:
   ```bash
   # Ensure outbound connections are allowed
   sudo ufw status
   ```

3. **Try alternative gateways**:
   ```bash
   vault-light-client gateways list
   vault-light-client start --gateway tcp://gateway2.solanavault.com:5050
   ```

4. **Check network connectivity**:
   ```bash
   ping gateway.solanavault.com
   ```

---

### Connection timeout

**Error:**

```
Error: Request timeout after 30000ms
```

**Solutions:**

1. **Increase timeout**:
   ```toml
   [network]
   request_timeout_ms = 60000
   ```

2. **Check network latency**:
   ```bash
   ping gateway.solanavault.com
   ```

3. **Use closer gateway**:
   ```bash
   vault-light-client gateways list --sort-by latency
   ```

---

### Peer discovery fails

**Error:**

```
Error: No peers found after discovery
```

**Solutions:**

1. **Check bootstrap nodes**:
   ```toml
   [network]
   bootstrap_nodes = [
       "tcp://bootstrap1.solanavault.com:4040",
       "tcp://bootstrap2.solanavault.com:4040"
   ]
   ```

2. **Verify P2P port is open**:
   ```bash
   # For incoming connections
   sudo ufw allow 4040/tcp
   ```

3. **Check logs for details**:
   ```bash
   RUST_LOG=vault_core::network::discovery=debug vault-node ...
   ```

---

## Balance and Payment Issues

### Insufficient balance

**Error:**

```
Error: Insufficient balance for request (required: 500, available: 100)
```

**Solutions:**

1. **Add more tokens**:
   ```bash
   vault-light-client balance add 50000
   ```

2. **Enable caching to reduce costs**:
   ```toml
   [cache]
   enabled = true
   max_entries = 100000
   ```

3. **Check spending**:
   ```bash
   vault-light-client balance history --days 1
   ```

---

### Payment rejected

**Error:**

```
Error: Payment rejected by gateway
```

**Solutions:**

1. **Check balance**:
   ```bash
   vault-light-client balance
   ```

2. **Verify gateway accepts your payment method**:
   ```bash
   vault-light-client gateways test tcp://gateway:5050
   ```

3. **Try different gateway**:
   ```bash
   vault-light-client start --gateway tcp://gateway2:5050
   ```

---

## Sync Issues

### Sync stuck

**Symptoms:**

- Sync progress not advancing
- Same block height for extended period

**Solutions:**

1. **Check peer connections**:
   ```bash
   vault-node peers list
   ```

2. **Restart sync**:
   ```bash
   vault-node sync reset
   ```

3. **Use warp sync**:
   ```toml
   [sync]
   mode = "warp"
   ```

4. **Check disk space**:
   ```bash
   df -h /data/solanavault
   ```

---

### Sync behind

**Error:**

```
Warning: Node is 1000 blocks behind network
```

**Solutions:**

1. **Check network bandwidth**:
   ```bash
   # Monitor network usage
   nethogs
   ```

2. **Increase parallel downloads**:
   ```toml
   [sync]
   parallel_downloads = 16
   ```

3. **Check CPU usage**:
   ```bash
   htop
   ```

---

## Consensus Issues

### Not participating in consensus

**Error:**

```
Warning: Not selected for consensus participation
```

**Solutions:**

1. **Verify stake**:
   ```bash
   vault-node stake status
   ```

2. **Check minimum stake**:
   ```bash
   # Stake must meet minimum requirement
   vault-node stake deposit 10000
   ```

3. **Check uptime**:
   - Ensure node has been running consistently
   - Check for recent restarts

---

### Slashing warning

**Error:**

```
Warning: Potential slashing condition detected
```

**Solutions:**

1. **Check for double voting**:
   ```bash
   vault-node consensus status
   ```

2. **Ensure single node per key**:
   - Never run two nodes with the same key

3. **Enable slashing protection**:
   ```toml
   [consensus]
   enable_slashing_protection = true
   ```

---

## Performance Issues

### High latency

**Symptoms:**

- Slow RPC responses
- Request timeouts

**Solutions:**

1. **Enable caching**:
   ```toml
   [cache]
   enabled = true
   max_size_mb = 1024
   ```

2. **Use NVMe storage**:
   - SSD is minimum, NVMe recommended

3. **Increase memory**:
   ```toml
   [storage]
   cache_size_mb = 8192
   ```

4. **Check system resources**:
   ```bash
   htop
   iostat -x 1
   ```

---

### Low cache hit rate

**Symptoms:**

- Cache hit rate below 80%
- High gateway costs

**Solutions:**

1. **Increase cache size**:
   ```toml
   [cache]
   max_entries = 500000
   max_size_mb = 2048
   ```

2. **Enable persistent cache**:
   ```toml
   [cache]
   persistent = true
   persistent_path = "~/.solanavault/cache"
   ```

3. **Adjust TTL**:
   ```toml
   [cache]
   ttl_seconds = 7200
   ```

---

### High memory usage

**Symptoms:**

- OOM errors
- System slowdown

**Solutions:**

1. **Limit cache size**:
   ```toml
   [cache]
   max_size_mb = 512
   ```

2. **Reduce connection pool**:
   ```toml
   [network]
   connection_pool = 50
   ```

3. **Check for memory leaks**:
   ```bash
   RUST_LOG=debug vault-node ... 2>&1 | grep -i memory
   ```

---

## Storage Issues

### Disk full

**Error:**

```
Error: No space left on device
```

**Solutions:**

1. **Check disk usage**:
   ```bash
   df -h /data/solanavault
   du -sh /data/solanavault/*
   ```

2. **Run garbage collection**:
   ```bash
   vault-node storage gc --force
   ```

3. **Reduce storage**:
   ```toml
   [storage]
   max_storage_gb = 500  # Reduce capacity
   gc_interval_hours = 6  # More frequent cleanup
   ```

---

### Data corruption

**Error:**

```
Error: Data integrity check failed
```

**Solutions:**

1. **Verify data**:
   ```bash
   vault-node storage verify
   ```

2. **Restore from backup**:
   ```bash
   vault-node backup restore /backup/latest.tar.gz
   ```

3. **Reset and resync**:
   ```bash
   vault-node sync reset --full
   ```

---

## Logging and Debugging

### Enable debug logging

```bash
# All debug logs
RUST_LOG=debug vault-node ...

# Specific module
RUST_LOG=vault_core::network=debug vault-node ...

# Multiple modules
RUST_LOG=vault_core::network=debug,vault_core::consensus=trace vault-node ...
```

### Common log locations

```bash
# Systemd journal
journalctl -u solanavault-node -f

# Custom log file
vault-node ... 2>&1 | tee /var/log/solanavault.log
```

### Get stack traces

```bash
RUST_BACKTRACE=1 vault-node ...
RUST_BACKTRACE=full vault-node ...  # Full traces
```

---

## Getting Help

If these solutions don't resolve your issue:

1. **Check the FAQ**: [FAQ](faq.md)

2. **Search GitHub Issues**:
   ```
   https://github.com/solanavault/solanavault/issues
   ```

3. **Open a new issue** with:
   - SolanaVault version (`vault-cli --version`)
   - Operating system
   - Full error message
   - Relevant logs
   - Steps to reproduce

4. **Join Discord/Community**:
   - Real-time help from community
   - Share experiences with others

---

## Next Steps

- [FAQ](faq.md) - Frequently asked questions
- [Configuration Reference](../reference/configuration.md) - All options
- [Quick Start](../guides/quickstart.md) - Getting started

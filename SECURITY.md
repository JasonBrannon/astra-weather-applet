# Security Policy

**Supported:** 1.0.x | **Reporting:** Email maintainer (see GitLab profile), response within 48h | **DO NOT** open public issues for vulnerabilities

## API Key Storage

API keys are stored using the freedesktop.org Secret Service D-Bus API (primary) with encrypted persistence via system keyrings (gnome-keyring, KWallet). The implementation uses Rust's `keyring` crate with the `sync-secret-service` feature for cross-platform compatibility. If Secret Service is unavailable, a secure fallback writes to `~/.config/cosmic/com.slagmine.astra/config.toml` with 600 permissions (owner read/write only). API key values are never logged—only storage success/failure events are recorded.

```bash
# System dependencies (compilation)
sudo apt install libdbus-1-dev pkg-config libssl-dev

# Check active storage method
RUST_LOG=info astra 2>&1 | grep "API key"
# "Stored API key in system keyring" (✅ secure) OR "Stored API key in config file" (⚠️ fallback)
```

```rust
// src/security.rs - Primary: Secret Service D-Bus API (keyring crate, sync-secret-service feature)
// Service: com.slagmine.astra, Username: api-key
// Backends: gnome-keyring, KWallet, any Secret Service provider
// ✅ Encrypted, persistent (reboots), system auth protected, auto-unlocked on login

keyring::Entry::new("com.slagmine.astra", "api-key")
    .set_password(&api_key)
    .unwrap_or_else(|_| save_to_config_file(&api_key))  // Fallback if D-Bus unavailable

// Fallback: ~/.config/cosmic/com.slagmine.astra/config.toml
// Permissions: 600 (owner only), ⚠️ plaintext, ⚠️ visible in backups
// API key values NEVER logged (only storage success/failure)
```

## Network Security

```rust
// All API calls: HTTPS/WSS only, TLS 1.2+, cert validation, no HTTP fallback
// REST: https://swd.weatherflow.com/swd/rest/*
// WebSocket: wss://ws.weatherflow.com/swd/data
// API key: request headers (not URLs), always encrypted
```

## Best Practices

```bash
# Verify Secret Service available
dbus-send --session --print-reply --dest=org.freedesktop.secrets \
  /org/freedesktop/secrets org.freedesktop.DBus.Properties.Get \
  string:org.freedesktop.Secret.Service string:Collections 2>/dev/null && echo "Available"

# User: Strong login password (Secret Service encryption), full-disk encryption, lock screen
# User: Exclude ~/.config/cosmic/ from cloud backups, encrypt local backups
# User: Get API key from tempestwx.com/settings/tokens, don't share, regenerate if compromised

# Developer checklist
# [ ] No API keys in code/logs
# [ ] Test keyring + fallback paths
# [ ] Config file permissions: 600
# [ ] All network: HTTPS/WSS
# [ ] cargo audit before release

RUST_LOG=debug cargo run  # Verify keyring storage
cargo audit               # Check vulnerabilities
```

## Limitations & Privacy

```bash
# Known limitations
# 1. Fallback: plaintext config if Secret Service unavailable
# 2. Memory: API key visible in process memory while running
# 3. Swap: may be written to swap (use encrypted swap)

# Privacy
# ✅ No telemetry, analytics, tracking, third parties
# ✅ Local processing only, direct WeatherFlow API communication
# ✅ Debug logs to stdout only (cleared on exit)

# Dependencies (SBOM tracking)
just sbom                                # Generate SBOM
jq '.packages | length' sbom.spdx.json  # Current: 721 dependencies
cargo audit                              # Security scanning
```

---

**Contact:** Email maintainer (GitLab profile) | GPG available on request

**Made for the COSMIC Desktop community with ❤️** | GPL-3.0-or-later


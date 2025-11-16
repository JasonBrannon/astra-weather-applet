# Contributing to Astra Weather Applet

## Issues

[Open an issue](https://github.com/JasonBrannon/astra-weather-applet/issues) with: description, repro steps, expected vs actual behavior, system info (distro, COSMIC version, Rust version), API key status, screenshots. Check for duplicates first.

## Pull Requests

**Before submitting:**
- Bug fixes: Always welcome (reference issue #)
- Features/UI changes: Discuss in issue tracker first
- Translations: Always welcome (5 languages must stay 100% synchronized)
- Infrastructure: Discuss with maintainers first

**Code quality checks (all must pass):**
```bash
cargo build --release 2>&1 | grep "^warning:" | wc -l  # Must be 0
just check-headers                                       # GPL-3.0 headers required
cargo test                                               # All tests pass
```

**Commits:** Descriptive messages (explain "why"), logical splits, squash related commits only

**PR format:** Clear title/description, before/after screenshots for UI changes, reference issue numbers, explain reasoning

**Engagement required:** Respond to review comments or PR will be closed

## Translations

5 languages must stay **100% synchronized** (200 lines each): en, de, es, gd, ja

```bash
# Files: i18n/{language}/astra.ftl
# 1. Update ALL 5 language files when adding keys
wc -l i18n/*/astra.ftl                # 2. Verify line counts match
# 3. Parameter names must match English ({$value}, {$time})
LANG=de cargo run                     # 4. Test with target language
```

## Development Setup

```bash
# System dependencies (Pop!_OS/Ubuntu/Debian)
sudo apt install libdbus-1-dev pkg-config libssl-dev  # D-Bus keyring, build tools, TLS

# Clone and build
git clone https://github.com/JasonBrannon/astra-weather-applet
cd astra-weather-applet
just build-release
cargo run -- --test                   # Test without GUI
just install                          # Install locally

# Testing modes
RUST_LOG=debug cargo run              # Debug logging
# DEV_MODE in src/constants.rs        # Mock stations (10 fake stations for UI testing)
```

**Code style:** `cargo fmt`, zero warnings, GPL-3.0 SPDX headers, `tracing` (not `println!`), Tokio async.

## Resources

[Issue Tracker](https://github.com/JasonBrannon/astra-weather-applet/issues) | [FEATURES.md](FEATURES.md)| [SBOM.md](SBOM.md)

---

**Made for the COSMIC Desktop community with ❤️** | GPL-3.0-or-later

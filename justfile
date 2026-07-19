name := 'astra'
appid := 'com.slagmine.astra'

rootdir := ''
prefix := env_var('HOME') + '/.local'

base-dir := absolute_path(clean(rootdir / prefix))

bin-src := 'target' / 'release' / name
bin-dst := base-dir / 'bin' / name

desktop := appid + '.desktop'
desktop-src := 'resources' / desktop
desktop-dst := clean(rootdir / prefix) / 'share' / 'applications' / desktop

metainfo := appid + '.metainfo.xml'
metainfo-src := 'resources' / metainfo
metainfo-dst := clean(rootdir / prefix) / 'share' / 'metainfo' / metainfo

icons-src := 'resources' / 'icons' / 'hicolor'
icons-dst := clean(rootdir / prefix) / 'share' / 'icons' / 'hicolor'

icon-sizes := '128 256 512'

# Default recipe which runs `just build-release`
default: build-release

# Runs `cargo clean`
clean:
    cargo clean

# Removes vendored dependencies
clean-vendor:
    rm -rf .cargo vendor vendor.tar

# `cargo clean` and removes vendored dependencies
clean-dist: clean clean-vendor

# Compiles with debug profile
build-debug *args:
    cargo build {{args}}

# Compiles with release profile
build-release *args: (build-debug '--release' args)

# Compiles release profile with vendored dependencies
build-vendored *args: vendor-extract (build-release '--frozen --offline' args)

# Runs a clippy check
check *args:
    cargo clippy --all-features {{args}} -- -W clippy::pedantic

# Runs a clippy check with JSON message format
check-json: (check '--message-format=json')

# Check if all source files have GPL-3.0 headers
check-headers:
    #!/usr/bin/env bash
    echo "🔍 Checking for GPL-3.0-or-later headers..."
    missing=0
    incorrect=0
    for file in $(find src/ build.rs -name "*.rs" -type f 2>/dev/null); do
        if ! head -n 1 "$file" | grep -q "^// SPDX-License-Identifier: GPL-3.0-or-later"; then
            if head -n 1 "$file" | grep -q "^// SPDX-License-Identifier:"; then
                echo "❌ Incorrect header: $file"
                head -n 1 "$file"
                ((incorrect++))
            else
                echo "❌ Missing header: $file"
                ((missing++))
            fi
        fi
    done
    if [ $missing -eq 0 ] && [ $incorrect -eq 0 ]; then
        echo "✅ All source files have correct GPL-3.0-or-later headers!"
        exit 0
    else
        echo ""
        echo "Found $missing missing and $incorrect incorrect headers."
        echo "Run 'just fix-headers' to add/fix them."
        exit 1
    fi

# Add GPL-3.0 headers to all source files
fix-headers:
    #!/usr/bin/env bash
    echo "🔧 Adding GPL-3.0-or-later headers to source files..."
    header="// SPDX-License-Identifier: GPL-3.0-or-later"
    for file in $(find src/ build.rs -name "*.rs" -type f 2>/dev/null); do
        if head -n 1 "$file" | grep -q "^// SPDX-License-Identifier:"; then
            # Replace existing SPDX header
            tail -n +2 "$file" > "${file}.tmp"
            echo "$header" > "${file}.new"
            cat "${file}.tmp" >> "${file}.new"
            mv "${file}.new" "$file"
            rm "${file}.tmp"
            echo "  ✓ Updated: $file"
        else
            # Add new header
            echo "$header" > "${file}.new"
            echo "" >> "${file}.new"
            cat "$file" >> "${file}.new"
            mv "${file}.new" "$file"
            echo "  ✓ Added: $file"
        fi
    done
    echo "✅ All headers updated!"

# Run the application for testing purposes
run *args:
    env RUST_BACKTRACE=full cargo run --release {{args}}

# Installs files
install:
    #!/usr/bin/env bash
    killall astra 2>/dev/null || true
    install -Dm0755 {{bin-src}} {{bin-dst}}
    install -Dm0644 {{desktop-src}} {{desktop-dst}}
    install -Dm0644 {{metainfo-src}} {{metainfo-dst}}
    for size in {{icon-sizes}}; do
        install -Dm0644 "{{icons-src}}/${size}x${size}/apps/{{appid}}.png" "{{icons-dst}}/${size}x${size}/apps/{{appid}}.png"
    done
    gtk-update-icon-cache -f -t {{icons-dst}} 2>/dev/null || true

# Uninstalls installed files
uninstall:
    #!/usr/bin/env bash
    killall astra 2>/dev/null || true
    rm -f {{bin-dst}} {{desktop-dst}} {{metainfo-dst}}
    for size in {{icon-sizes}}; do
        rm -f "{{icons-dst}}/${size}x${size}/apps/{{appid}}.png"
    done

# Uninstalls everything including config, cache, and keyring
uninstall-all: uninstall
    #!/usr/bin/env bash
    echo "🗑️  Removing configuration and cache..."
    # Remove COSMIC config directory
    rm -rf ~/.config/cosmic/com.slagmine.astra
    # Remove old config directories from previous branding
    rm -rf ~/.config/cosmic/com.astra.tempest
    rm -rf ~/.config/cosmic/com.slagmine.tempest
    rm -rf ~/.config/com.slagmine.astra
    rm -rf ~/.cache/com.slagmine.astra
    echo "✅ Configuration and cache removed"
    echo ""
    echo "🔑 Attempting to remove API key from keyring..."
    # Try secret-tool first (gnome-keyring)
    if command -v secret-tool &> /dev/null; then
        secret-tool clear service com.slagmine.astra username api-key 2>/dev/null && echo "✅ API key removed from keyring" || echo "ℹ️  No API key found in keyring (or already removed)"
    # Fallback to Python secretstorage
    elif command -v python3 &> /dev/null; then
        python3 -c "import secretstorage; bus = secretstorage.dbus_init(); collection = secretstorage.get_default_collection(bus); items = collection.search_items({'service': 'com.slagmine.astra', 'username': 'api-key'}); [item.delete() for item in items]; print('✅ API key removed from keyring')" 2>/dev/null || echo "ℹ️  No API key found in keyring"
    else
        echo "⚠️  No keyring tools available - keyring entry not removed"
        echo "   You can manually remove it using your keyring manager"
        echo "   Service: com.slagmine.astra, Username: api-key"
    fi
    echo ""
    echo "✅ Complete uninstall finished!"

# Vendor dependencies locally
vendor:
    #!/usr/bin/env bash
    mkdir -p .cargo
    cargo vendor --sync Cargo.toml | head -n -1 > .cargo/config.toml
    echo 'directory = "vendor"' >> .cargo/config.toml
    echo >> .cargo/config.toml
    echo '[env]' >> .cargo/config.toml
    if [ -n "${SOURCE_DATE_EPOCH}" ]
    then
        source_date="$(date -d "@${SOURCE_DATE_EPOCH}" "+%Y-%m-%d")"
        echo "VERGEN_GIT_COMMIT_DATE = \"${source_date}\"" >> .cargo/config.toml
    fi
    if [ -n "${SOURCE_GIT_HASH}" ]
    then
        echo "VERGEN_GIT_SHA = \"${SOURCE_GIT_HASH}\"" >> .cargo/config.toml
    fi
    tar pcf vendor.tar .cargo vendor
    rm -rf .cargo vendor

# Extracts vendored dependencies
vendor-extract:
    rm -rf vendor
    tar pxf vendor.tar

# Generate SPDX SBOM from dependencies (requires: cargo install cargo-sbom)
sbom:
    #!/usr/bin/env bash
    if ! command -v cargo-sbom &> /dev/null; then
        echo "❌ cargo-sbom not found. Installing..."
        cargo install cargo-sbom
    fi
    echo "📦 Generating SPDX SBOM..."
    cargo sbom --output-format spdx_json_2_3 > sbom.spdx.json
    echo "✅ SBOM generated: sbom.spdx.json"
    echo "   Format: SPDX 2.3 JSON"
    echo "   Size: $(wc -c < sbom.spdx.json | numfmt --to=iec-i --suffix=B --format="%.1f")"

# Generate both SPDX and CycloneDX SBOMs
sbom-all:
    #!/usr/bin/env bash
    if ! command -v cargo-sbom &> /dev/null; then
        echo "❌ cargo-sbom not found. Installing..."
        cargo install cargo-sbom
    fi
    echo "📦 Generating SBOM files..."
    cargo sbom --output-format spdx_json_2_3 > sbom.spdx.json
    cargo sbom --output-format cyclone_dx_json_1_4 > sbom.cdx.json
    echo "✅ SBOM files generated:"
    echo "   - sbom.spdx.json (SPDX 2.3)"
    echo "   - sbom.cdx.json (CycloneDX 1.4)"

# Generate cargo-sources.json for Flatpak offline builds
flatpak-sources:
    #!/usr/bin/env bash
    if ! command -v flatpak-cargo-generator &> /dev/null; then
        echo "❌ flatpak-cargo-generator not found. Installing..."
        cargo install flatpak-cargo-generator
    fi
    echo "📦 Generating cargo-sources.json for Flatpak..."
    flatpak-cargo-generator Cargo.lock -o cargo-sources.json
    echo "✅ cargo-sources.json generated"
    echo "   Size: $(wc -c < cargo-sources.json | numfmt --to=iec-i --suffix=B --format="%.1f")"
    echo "   This file is required for Flatpak offline builds"

# Verify SBOM dependencies match Cargo.lock (useful for CI/CD)
check-sbom:
    #!/usr/bin/env bash
    if ! command -v cargo-sbom &> /dev/null; then
        echo "❌ cargo-sbom not installed. Run: cargo install cargo-sbom"
        exit 1
    fi
    echo "🔍 Checking SBOM against Cargo.lock..."
    if [ -f sbom.spdx.json ]; then
        # Extract package count from existing SBOM (excluding timestamps)
        EXISTING_COUNT=$(jq '.packages | length' sbom.spdx.json 2>/dev/null || echo "0")

        # Generate fresh SBOM and count packages
        cargo sbom --output-format spdx_json_2_3 > sbom.spdx.json.tmp 2>/dev/null
        CURRENT_COUNT=$(jq '.packages | length' sbom.spdx.json.tmp)

        if [ "$EXISTING_COUNT" -eq "$CURRENT_COUNT" ]; then
            echo "✅ SBOM package count matches ($CURRENT_COUNT packages)"
            rm sbom.spdx.json.tmp
            exit 0
        else
            echo "⚠️  SBOM package count mismatch: $EXISTING_COUNT (existing) vs $CURRENT_COUNT (current)"
            echo "   Dependencies may have changed. Run 'just sbom' to update."
            rm sbom.spdx.json.tmp
            exit 1
        fi
    else
        echo "❌ sbom.spdx.json not found! Run 'just sbom'"
        exit 1
    fi


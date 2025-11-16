# Software Bill of Materials (SBOM)

This project generates SPDX-compliant Software Bills of Materials (SBOMs) for transparency and security compliance.

## Quick Start

### Generate SBOM Locally

```bash
# Generate SPDX 2.3 SBOM
just sbom

# Generate both SPDX and CycloneDX formats
just sbom-all

# Verify SBOM is up to date
just check-sbom
```

## What is an SBOM?

A Software Bill of Materials is a complete inventory of all software components, dependencies, and metadata in your application. It's crucial for:

- **Security**: Track vulnerable dependencies
- **Compliance**: Meet regulatory requirements (e.g., NTIA, EU Cyber Resilience Act)
- **Transparency**: Know exactly what's in your software
- **Supply Chain**: Verify software integrity and provenance

## SBOM Formats

This project supports two industry-standard formats:

1. **SPDX 2.3 JSON** (`sbom.spdx.json`)
   - ISO/IEC 5962:2021 standard
   - Comprehensive license and relationship data
   - Preferred for open-source compliance

2. **CycloneDX 1.4 JSON** (`sbom.cdx.json`)
   - OWASP standard
   - Optimized for security use cases
   - Integrates well with vulnerability scanners

## Automated Generation

### GitHub Actions

SBOMs are automatically generated on:

- **Every release** - Attached as release assets with attestation
- **Cargo.lock changes** - Available as artifacts
- **Manual trigger** - Via GitHub Actions UI

The workflow (`.github/workflows/sbom.yml`) performs:

1. ✅ Generates SPDX and CycloneDX SBOMs
2. ✅ Creates signed attestations (GitHub Attestations)
3. ✅ Uploads to release (if release event)
4. ✅ Archives as artifacts (if non-release)

### Attestation

For releases, GitHub automatically signs and attests the SBOM using:

- **Sigstore** - Keyless signing with OpenID Connect
- **Rekor** - Transparency log for verification
- **GitHub Attestations API** - Built-in artifact verification

Users can verify the SBOM authenticity:

```bash
gh attestation verify sbom.spdx.json --owner <username> --repo astra
```

## SBOM Contents

### Current Stats

- **Total Dependencies**: ~721 packages (as of last generation)
- **License Coverage**: 100% (all dependencies have license metadata)
- **Format**: SPDX 2.3 JSON
- **File Size**: ~797 KB

### What's Included

Each SBOM contains:

- Package names and versions
- License identifiers (SPDX format)
- Package checksums (SHA1)
- Dependency relationships
- Creation timestamp
- Creator information

### Example Package Entry

```json
{
  "SPDXID": "SPDXRef-serde-1.0.228",
  "name": "serde",
  "versionInfo": "1.0.228",
  "licenseConcluded": "MIT OR Apache-2.0",
  "downloadLocation": "https://crates.io/crates/serde/1.0.228",
  "checksums": [
    {
      "algorithm": "SHA1",
      "checksumValue": "..."
    }
  ]
}
```

## Integration

### CI/CD Pipelines

Add SBOM validation to your CI:

```yaml
- name: Verify SBOM
  run: just check-sbom
```

### Vulnerability Scanning

Use SBOMs with security scanners:

```bash
# With Grype (Anchore)
grype sbom:sbom.spdx.json

# With Trivy (Aqua Security)
trivy sbom sbom.spdx.json

# With OSV-Scanner (Google)
osv-scanner --sbom=sbom.spdx.json
```

### Supply Chain Security

SBOMs integrate with SLSA (Supply Chain Levels for Software Artifacts):

1. **SBOM Generation** - Part of build process
2. **Attestation** - Signed with GitHub's identity
3. **Verification** - Users can verify SBOM authenticity
4. **Provenance** - Links SBOM to specific build

## License Compliance

The SBOM includes complete license information for all dependencies:

```bash
# Extract all unique licenses
jq -r '.packages[].licenseConcluded' sbom.spdx.json | sort -u

# Common licenses in this project:
# - MIT
# - Apache-2.0
# - MIT OR Apache-2.0
# - GPL-3.0-or-later (this project)
```

## Resources

- [SPDX Specification](https://spdx.github.io/spdx-spec/)
- [CycloneDX Specification](https://cyclonedx.org/)
- [GitHub Attestations](https://docs.github.com/en/actions/security-guides/using-artifact-attestations-to-establish-provenance-for-builds)
- [NTIA SBOM Requirements](https://www.ntia.gov/page/software-bill-materials)
- [cargo-sbom Documentation](https://crates.io/crates/cargo-sbom)

## Troubleshooting

### cargo-sbom not found

```bash
cargo install cargo-sbom
```

### SBOM out of date

```bash
just sbom
```

### Missing dependencies in SBOM

Ensure `Cargo.lock` is up to date:

```bash
cargo update
just sbom
```

## Security Contact

If you discover a security vulnerability in any dependency listed in the SBOM, please report it via:

- GitHub Security Advisories
- Email: [Your security contact]

---

**Last Updated**: 2025-11-16
**SBOM Tool**: cargo-sbom v0.10.0
**SPDX Version**: 2.3
**CycloneDX Version**: 1.4

---

**Made for the COSMIC Desktop community with ❤️** | GPL-3.0-or-later
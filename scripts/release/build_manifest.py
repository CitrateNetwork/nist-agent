#!/usr/bin/env python3
"""Build a release.manifest.toml from a staged bundle directory.

Walks the staging dir, hashes every file (excluding the manifest
itself), classifies by path prefix, and emits the manifest TOML
in the shape `nist_agent_release::ReleaseManifest` decodes.

The signature block is initialized to the pre-signature form
(zeroed signature_hex, empty public_key_hex); the next workflow
step (`sign-manifest`) fills it in.

Why Python and not Rust? Pure-stdlib (no deps to fetch in CI),
deterministic output, easy to inspect on the air-gap host.
"""
import argparse
import hashlib
import os
import sys
from pathlib import Path

# Pre-signature form (128 hex zeros = 64 bytes zero) per
# `SignatureEnvelope::PRE_SIGNATURE_SIGNATURE_HEX`.
PRE_SIGNATURE_HEX = "0" * 128


def classify(rel_path: str) -> str:
    """Map a relative path inside the staging dir to a manifest
    artifact `kind`. Matches `ArtifactKind` variants from
    nist-agent-release."""
    p = rel_path.lower()
    if p.startswith("bin/"):
        return "daemon"
    if p.startswith("models/") and p.endswith(".gguf"):
        return "model"
    if p.startswith("models/") and "license" in p:
        return "model-license"
    if p.startswith("capsules/"):
        return "capsule"
    if p.startswith("sbom/") or p.endswith(".spdx.json") or p.endswith(".cdx.json"):
        return "sbom"
    if p.startswith("runbooks/") or p.endswith("RUNBOOK.md"):
        return "runbook"
    return "other"


def sha256_of(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def main(argv=None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--version", required=True)
    ap.add_argument("--git-rev", required=True)
    ap.add_argument("--staging", required=True, type=Path)
    ap.add_argument("--out", required=True, type=Path)
    args = ap.parse_args(argv)

    staging: Path = args.staging.resolve()
    if not staging.is_dir():
        print(f"staging dir not found: {staging}", file=sys.stderr)
        return 2

    entries = []
    for root, _dirs, files in os.walk(staging):
        # Deterministic order: sort dirs + files.
        for fname in sorted(files):
            abs_path = Path(root) / fname
            rel_path = abs_path.relative_to(staging).as_posix()
            # Skip the manifest itself (we're producing it).
            if rel_path == args.out.relative_to(staging).as_posix():
                continue
            kind = classify(rel_path)
            sha = sha256_of(abs_path)
            entries.append((rel_path, kind, sha))

    # Stable sort by path so the manifest is reproducible.
    entries.sort(key=lambda e: e[0])

    lines = []
    lines.append(f'version = "{args.version}"')
    lines.append(f'git_rev = "{args.git_rev}"')
    lines.append("")
    lines.append("[signature]")
    lines.append('algorithm = "ed25519"')
    lines.append('public_key_hex = ""')
    lines.append(f'signature_hex = "{PRE_SIGNATURE_HEX}"')
    lines.append("")
    for path, kind, sha in entries:
        lines.append("[[artifact]]")
        lines.append(f'kind = "{kind}"')
        lines.append(f'path = "{path}"')
        lines.append(f'sha256 = "0x{sha}"')
        lines.append("")

    out_path = args.out
    out_path.write_text("\n".join(lines), encoding="utf-8")
    print(f"wrote {out_path} ({len(entries)} artifact(s))")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

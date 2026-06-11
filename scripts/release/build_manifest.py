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
    ap.add_argument(
        "--expect",
        action="append",
        default=[],
        metavar="PATH=SHA256",
        help=(
            "require the staged file at PATH (relative to --staging) to "
            "hash to SHA256 (hex, optional 0x prefix); repeatable. Used "
            "by the release workflow to bind the manifest to the "
            "reproducibility-gate hash — a mismatch or missing file "
            "aborts before any manifest is written (FUA-NIST-AGENT-01)."
        ),
    )
    args = ap.parse_args(argv)

    expected: dict[str, str] = {}
    for spec in args.expect:
        path_part, sep, sha_part = spec.partition("=")
        sha_norm = sha_part.lower().removeprefix("0x")
        if not sep or not path_part or len(sha_norm) != 64:
            print(f"bad --expect (want PATH=SHA256-hex): {spec}", file=sys.stderr)
            return 2
        expected[path_part] = sha_norm

    staging: Path = args.staging.resolve()
    if not staging.is_dir():
        print(f"staging dir not found: {staging}", file=sys.stderr)
        return 2

    # Resolve --out so relative_to(staging) works regardless of
    # whether the caller passed an absolute or relative path.
    # The output may live outside the staging dir (e.g. in CI it
    # lives at release-staging/release.manifest.toml relative to
    # the workflow's CWD, which resolves to the same absolute
    # path as `staging`).
    out_abs = args.out.resolve() if args.out.is_absolute() else (Path.cwd() / args.out).resolve()
    try:
        out_rel_to_staging = out_abs.relative_to(staging).as_posix()
    except ValueError:
        # --out is outside --staging; nothing to skip.
        out_rel_to_staging = None

    entries = []
    for root, _dirs, files in os.walk(staging):
        # Deterministic order: sort dirs + files.
        for fname in sorted(files):
            abs_path = Path(root) / fname
            rel_path = abs_path.relative_to(staging).as_posix()
            # Skip the manifest itself (we're producing it).
            if out_rel_to_staging is not None and rel_path == out_rel_to_staging:
                continue
            kind = classify(rel_path)
            sha = sha256_of(abs_path)
            entries.append((rel_path, kind, sha))

    # Stable sort by path so the manifest is reproducible.
    entries.sort(key=lambda e: e[0])

    # Hash binding: every --expect must name a staged file whose
    # sha256 matches exactly. Fail BEFORE writing any manifest so a
    # binary that drifted from the reproducibility gate can never be
    # carried by a signed manifest (FUA-NIST-AGENT-01).
    by_path = {path: sha for path, _kind, sha in entries}
    for path, want in sorted(expected.items()):
        got = by_path.get(path)
        if got is None:
            print(f"--expect {path}: file not present in staging dir", file=sys.stderr)
            return 3
        if got != want:
            print(
                f"--expect {path}: hash mismatch (staged sha256 {got}, expected {want})",
                file=sys.stderr,
            )
            return 3

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

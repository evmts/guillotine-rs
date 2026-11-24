#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."
metadata=$(cargo metadata --offline --locked --no-deps --format-version 1)
package_info=$(python3 -c '
import json, sys
data = json.load(sys.stdin)
package = next(p for p in data["packages"] if p["name"] == "guillotine-rs")
print(data["target_directory"])
print(f"{package['"'"'name'"'"']}-{package['"'"'version'"'"']}")
print(next(d["req"] for d in package["dependencies"] if d["name"] == "revm"))
' <<< "$metadata")
target_dir=$(sed -n '1p' <<< "$package_info")
package=$(sed -n '2p' <<< "$package_info")
revm=$(sed -n '3p' <<< "$package_info")

cargo package --locked --allow-dirty

scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT
tar -xzf "$target_dir/package/$package.crate" -C "$scratch"
mkdir -p "$scratch/consumer/tests"
cp "$scratch/$package/tests/e2e.rs" "$scratch/consumer/tests/e2e.rs"
cp "$scratch/$package/Cargo.lock" "$scratch/consumer/Cargo.lock"
cat > "$scratch/consumer/Cargo.toml" <<MANIFEST
[package]
name = "guillotine-package-consumer"
version = "0.0.0"
edition = "2021"

[dependencies]
guillotine-rs = { path = "../$package" }
revm = "$revm"

[[test]]
name = "e2e"
path = "tests/e2e.rs"
MANIFEST
CARGO_TARGET_DIR="$target_dir" cargo test --offline --manifest-path "$scratch/consumer/Cargo.toml"

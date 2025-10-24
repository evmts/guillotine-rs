# Publishing Guide for guillotine-rs

## Current Status

guillotine-rs **cannot be published to crates.io** in its current form due to git submodule dependencies.

## Problem

Cargo does not include git submodules when packaging crates. The `cargo package` command creates a tarball that excludes the `lib/guillotine-mini` submodule, causing the build to fail during verification.

## Evidence

```bash
$ cargo package --allow-dirty
   Packaging guillotine-rs v0.1.0
   Packaged 22 files  # <- lib/guillotine-mini is NOT included
   Verifying guillotine-rs v0.1.0
   Compiling guillotine-rs v0.1.0
error: Failed to execute zig build command: No such file or directory
```

## Current Recommended Installation

Users must install via git with submodules:

```bash
# Clone with submodules
git clone --recursive https://github.com/evmts/guillotine-rs.git
cd guillotine-rs
cargo build

# Or via Cargo.toml
[dependencies]
guillotine-rs = { git = "https://github.com/evmts/guillotine-rs", submodules = true }
```

## Future Solutions for crates.io Publishing

### Option 1: Vendor guillotine-mini (Recommended)

Copy guillotine-mini source code directly into the repository instead of using a submodule:

1. Remove submodule: `git submodule deinit lib/guillotine-mini`
2. Copy files: `cp -r lib/guillotine-mini lib/guillotine-mini-vendor`
3. Add to git: `git add lib/guillotine-mini-vendor`
4. Update build.rs to point to new path
5. Test packaging: `cargo package --allow-dirty`

**Pros:**
- Works with crates.io
- Single repository
- No submodule complexity

**Cons:**
- Harder to sync updates from upstream guillotine-mini
- Larger repository size
- Duplicated code

### Option 2: Publish guillotine-mini as separate crate

Publish guillotine-mini as a Rust crate that includes the Zig source:

1. Create `guillotine-mini-sys` crate with Zig sources
2. Add as dependency: `guillotine-mini-sys = "0.1"`
3. Remove submodule from guillotine-rs

**Pros:**
- Modular design
- Follows Rust conventions for -sys crates
- Can version guillotine-mini independently

**Cons:**
- Requires maintaining separate crate
- More complex build setup
- Zig sources still need to be vendored in -sys crate

### Option 3: Build-time download (Not Recommended)

Download guillotine-mini in build.rs:

**Pros:**
- No vendoring needed

**Cons:**
- Network dependency during build
- Violates crates.io policy
- Security concerns
- Build fails offline

## Build Script Enhancements

The build.rs has been enhanced with:

✅ Zig version check (0.15.1+ required)
✅ Helpful error messages with installation instructions
✅ Git availability check
✅ Automatic submodule initialization
✅ Clear error reporting for build failures

## Package Metadata

Cargo.toml has been updated with:

✅ Description
✅ Repository URL
✅ Documentation URL
✅ Homepage
✅ Keywords
✅ Categories
✅ Exclude patterns for large test directories

## Testing Checklist

Before attempting to publish:

- [ ] Choose vendoring strategy (Option 1 or 2 above)
- [ ] Test `cargo package` succeeds
- [ ] Test `cargo package --list` includes all necessary files
- [ ] Test package builds in isolation: `cargo install --path .`
- [ ] Test in Docker container without zig (verify error message)
- [ ] Test in Docker container with zig (verify successful build)
- [ ] Update README.md to remove "git-only" warnings
- [ ] Update version in Cargo.toml
- [ ] Create git tag for release
- [ ] Dry run: `cargo publish --dry-run`
- [ ] Publish: `cargo publish`

## Current Package Stats

```bash
$ cargo package --list | wc -l
22 files

$ cargo package --allow-dirty 2>&1 | grep Packaged
Packaged 22 files, 202.6KiB (47.4KiB compressed)
```

**Missing:** lib/guillotine-mini/ submodule (~500+ files)

## Conclusion

**For now:** guillotine-rs remains a **git-only** distribution.

**For crates.io:** Implement Option 1 (vendoring) or Option 2 (separate -sys crate).

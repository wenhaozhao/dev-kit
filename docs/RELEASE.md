# Release process

## Version source

`[workspace.package].version` in the root `Cargo.toml` is the canonical version.
`dkui/package.json` and `dkui/src-tauri/tauri.conf.json` must carry the same value,
which is enforced by `./scripts/check-version.sh`.

## Assets and platforms

Release artifacts use the `devkit` name. The release workflow builds CLI archives for:

- macOS Apple Silicon (`aarch64-apple-darwin`)
- macOS Intel (`x86_64-apple-darwin`)
- Linux x86_64 (`x86_64-unknown-linux-gnu`)
- Windows x86_64 (`x86_64-pc-windows-msvc`)

Tauri bundles are produced for the matching supported runners. Every uploaded asset has
a SHA-256 checksum. A release tag must be `v<workspace version>`.

Pushing a matching tag runs `.github/workflows/release.yml`. It runs the quality
gate, builds the CLI archives and Tauri bundles, then creates a GitHub Release
with `SHA256SUMS.txt` and generated release notes.

## Required checks

Before a release, run:

```bash
./scripts/check-version.sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test -p dev-kit
cargo build --workspace
(cd dkui && npm ci && npm run build)
```

## Homebrew Tap

Create `wenhaozhao/homebrew-dev-kit`, then configure
`HOMEBREW_TAP_REPOSITORY=wenhaozhao/homebrew-dev-kit` and a write-capable
`HOMEBREW_TAP_TOKEN` secret. The release workflow renders the Formula from the
macOS CLI checksums and commits it to the configured Tap.

```bash
brew tap wenhaozhao/dev-kit
brew install devkit
devkit --version
```

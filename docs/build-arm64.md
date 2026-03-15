# Build for arm64

This repository already supports `arm64` via the Rust `aarch64` targets used in CI and release builds.

## Linux GNU (`aarch64-unknown-linux-gnu`)

Use this when you want a dynamically linked Linux `arm64` binary.

### Build on a native arm64 Linux machine

1. Install system dependencies.

```shell
sudo apt-get update
sudo apt-get install -y protobuf-compiler perl
```

2. Install Rust if it is not already present.

```shell
curl https://sh.rustup.rs -sSf | sh
. "$HOME/.cargo/env"
```

3. Build the release binary.

```shell
cargo build --target aarch64-unknown-linux-gnu --release --locked
```

4. Find the binary at:

```text
target/aarch64-unknown-linux-gnu/release/gurk
```

5. Build the release archive used by the release workflow, if needed.

```shell
GURK_TARGET=aarch64-unknown-linux-gnu cargo xtask dist
```

6. Find the archive at:

```text
dist/gurk-aarch64-unknown-linux-gnu.tar.gz
```

### Cross-compile from x86_64 Linux

The repository already sets `aarch64-linux-gnu-gcc` as the linker in [`.cargo/config.toml`](../.cargo/config.toml).

1. Install system dependencies.

```shell
sudo apt-get update
sudo apt-get install -y protobuf-compiler perl gcc-aarch64-linux-gnu
```

2. Add the Rust target.

```shell
rustup target add aarch64-unknown-linux-gnu
```

3. Build the release binary.

```shell
cargo build --target aarch64-unknown-linux-gnu --release --locked
```

4. Build the release archive, if needed.

```shell
GURK_TARGET=aarch64-unknown-linux-gnu cargo xtask dist
```

## Linux musl (`aarch64-unknown-linux-musl`)

Use this when you want a mostly static Linux `arm64` binary.

The repository already sets the required linker configuration in [`.cargo/config.toml`](../.cargo/config.toml).

### Build from Alpine Linux

1. Install system dependencies.

```shell
apk add --no-cache musl-dev lld protoc bash clang llvm make perl
```

2. Add the Rust target.

```shell
rustup target add aarch64-unknown-linux-musl
```

3. Build the release binary.

```shell
cargo build --target aarch64-unknown-linux-musl --release --locked
```

4. Build the release archive, if needed.

```shell
GURK_TARGET=aarch64-unknown-linux-musl cargo xtask dist
```

5. Find the outputs at:

```text
target/aarch64-unknown-linux-musl/release/gurk
dist/gurk-aarch64-unknown-linux-musl.tar.gz
```

If you specifically want to reproduce the current CI container job from an `x86_64` Alpine host, use:

```shell
rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl
GURK_TARGET=aarch64-unknown-linux-musl cargo run -p xtask --target x86_64-unknown-linux-musl -- dist
```

## macOS arm64 (`aarch64-apple-darwin`)

Build this target on an Apple Silicon Mac.

1. Install Xcode command line tools and `protoc`.

```shell
xcode-select --install
brew install protobuf
```

2. Add the Rust target.

```shell
rustup target add aarch64-apple-darwin
```

3. Build the release binary.

```shell
cargo build --target aarch64-apple-darwin --release --locked
```

4. Build the release archive, if needed.

```shell
GURK_TARGET=aarch64-apple-darwin cargo xtask dist
```

5. Find the outputs at:

```text
target/aarch64-apple-darwin/release/gurk
dist/gurk-aarch64-apple-darwin.tar.gz
```

# Build for arm64

This repository already supports `arm64` via the Rust `aarch64` targets used in CI and release builds.

## Linux GNU (`aarch64-unknown-linux-gnu`)

Use this when you want a dynamically linked Linux `arm64` binary.

If `cargo build --target aarch64-unknown-linux-gnu` fails with `can't find crate for core`
or `can't find crate for std`, your current Rust installation does not include the standard
library for that target. In practice this usually means Rust was installed from a distro
package instead of `rustup`, so `rustup target add aarch64-unknown-linux-gnu` is not
available. Fix that by either:

1. installing Rust through `rustup` and then adding the target, or
2. building for your host target instead of `aarch64-unknown-linux-gnu`.

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

### Build a UBports click package

Use this when you want an Ubuntu Touch package that installs on an `arm64` device and opens `gurk` inside a terminal window.
The build defaults to the validated `ubuntu-sdk-20.04.1` framework, which matches current Ubuntu Touch devices more reliably than `ubuntu-sdk-24.04`.

1. Install the cross-build and click packaging dependencies.

```shell
sudo apt-get update
sudo apt-get install -y protobuf-compiler perl gcc-aarch64-linux-gnu click
```

2. Add the Rust target.

```shell
rustup target add aarch64-unknown-linux-gnu
```

3. Build the click package.

```shell
cargo xtask click
```

To target a different framework explicitly, set `GURK_CLICK_FRAMEWORK` before building. For example:

```shell
GURK_CLICK_FRAMEWORK=ubuntu-sdk-24.04 cargo xtask click
```

If that fails with `click: command not found`, install the Ubuntu Click CLI and rerun the command:

```shell
sudo apt-get update
sudo apt-get install -y click
```

4. Find the output in `dist/`. The generated file name follows this pattern:

```text
dist/gurk.boxdot_<version>_arm64.click
```

5. Copy the click package to the device and install it.

```shell
adb push dist/gurk.boxdot_<version>_arm64.click /home/phablet/
adb shell pkcon install-local --allow-untrusted /home/phablet/gurk.boxdot_<version>_arm64.click
```

6. Launch `Gurk` from the app drawer. Ubuntu Touch opens it in a terminal window because the packaged desktop file sets `Terminal=true`.

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

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

Use this when you want an Ubuntu Touch package that installs on an `arm64` device.
The build uses `clickable` to assemble the final package and now defaults to the supported `ubuntu-touch-24.04-1.x` framework. The generated Click follows the Ubuntu Touch 24.04 packaging requirements by leaving the manifest framework as `@CLICK_FRAMEWORK@` and the AppArmor policy version as `@APPARMOR_POLICY@` for Clickable to fill at build time. The packaged app opens a GTK window with an embedded terminal running the standard `gurk` UI, and falls back to the platform terminal app if GTK/VTE bindings are unavailable at runtime.

1. Install the cross-build and Clickable packaging dependencies.

```shell
sudo apt-get update
sudo apt-get install -y protobuf-compiler perl gcc-aarch64-linux-gnu pipx
pipx install clickable-ut
```

2. Add the Rust target.

```shell
rustup target add aarch64-unknown-linux-gnu
```

3. Build the click package.

```shell
cargo xtask click
```

Clickable 8.4.0 or newer is required.

To target a different framework explicitly, set `CLICKABLE_FRAMEWORK` before building. `GURK_CLICK_FRAMEWORK` is still accepted as a backwards-compatible fallback. For example:

```shell
CLICKABLE_FRAMEWORK=ubuntu-touch-24.04-1.x cargo xtask click
```

If that fails with `clickable: command not found`, install Clickable and rerun the command:

```shell
sudo apt-get update
sudo apt-get install -y pipx
pipx install clickable-ut
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

6. Launch `Gurk` from the app drawer.

### Debug when no window appears on the device

If tapping `Gurk` does nothing, debug the packaged launcher directly on the device first. The click package starts `gurk-launch`, which tries the GTK/VTE frontend first and then falls back to `lomiri-terminal-app`, `ubuntu-terminal-app`, or another terminal if GTK/VTE is unavailable.

1. Open a shell on the device.

```shell
adb shell
```

2. Find the installed click app directory.

```shell
find /opt/click.ubuntu.com -maxdepth 3 -type f -name gurk-launch 2>/dev/null
```

You should see a path ending in `.../gurk.boxdot/current/gurk-launch`.

3. Change into that directory and confirm the packaged files are present.

```shell
cd /opt/click.ubuntu.com/gurk.boxdot/current
ls -l
```

At minimum, verify that `gurk`, `gurk-launch`, `gurk.desktop`, and `gurk-gtk-frontend.py` exist.

4. Launch the packaged app from the shell and keep all output visible.

```shell
./gurk-launch
```

If the app exits immediately, run the launcher in verbose mode and enable trace logging for the Rust backend.

```shell
RUST_LOG=gurk=trace,presage=trace,libsignal=trace ./gurk-launch --verbose
```

`gurk --verbose` writes `gurk.log` in the current working directory, so when started from the package directory the log file will be:

```text
/opt/click.ubuntu.com/gurk.boxdot/current/gurk.log
```

5. Inspect the backend log.

```shell
sed -n '1,200p' gurk.log
tail -n 100 gurk.log
```

6. If the GTK window itself may be the problem, bypass `gurk-launch` and start the frontend script directly.

```shell
python3 ./gurk-gtk-frontend.py ./gurk
```

For maximum detail:

```shell
RUST_LOG=gurk=trace,presage=trace,libsignal=trace python3 ./gurk-gtk-frontend.py ./gurk --verbose
```

This isolates whether the failure is in the shell launcher, the GTK/VTE frontend, or the Rust backend.

7. If neither command shows a window, verify the GTK dependencies that the launcher checks before using the embedded frontend.

```shell
python3 -c 'import gi; gi.require_version("Gtk", "3.0"); gi.require_version("Vte", "2.91"); from gi.repository import Gtk, Vte; print("gtk-vte-ok")'
which lomiri-terminal-app
which ubuntu-terminal-app
```

8. When you need a host-side copy of the device log, pull it back with `adb`.

```shell
exit
adb pull /opt/click.ubuntu.com/gurk.boxdot/current/gurk.log .
```

Common outcomes:

- If `python3 ./gurk-gtk-frontend.py ./gurk` fails before the Rust app starts, the issue is in the packaged GTK/PyGObject/VTE path.
- If the frontend opens but `gurk.log` shows startup failures, the issue is in `gurk` itself, usually config, linking, passphrase, or storage startup.
- If the frontend path is unavailable but `lomiri-terminal-app` exists, `./gurk-launch` should still open a terminal window; if it does not, focus on the terminal-app fallback on that device image.

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

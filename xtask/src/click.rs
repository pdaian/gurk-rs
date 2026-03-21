use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use toml::Table;
use xshell::{cmd, Shell};

use crate::{flags, project_root};

const TARGET: &str = "aarch64-unknown-linux-gnu";
const DEFAULT_FRAMEWORK: &str = "ubuntu-sdk-24.04";
const DEFAULT_POLICY_VERSION: &str = "24.04";
const PACKAGE_NAME: &str = "gurk.boxdot";
const APP_NAME: &str = "gurk";
const CLICK_INSTALL_HINT: &str =
    "install the Ubuntu Click CLI first, for example with `sudo apt-get install -y click`";

impl flags::Click {
    pub(crate) fn run(self, sh: &Shell) -> Result<()> {
        let version = package_version()?;
        let dist_dir = project_root().join("dist");
        let stage_dir = dist_dir.join("ubports-click");

        ensure_click_available(sh)?;

        sh.create_dir(&dist_dir)?;
        sh.remove_path(&stage_dir)?;
        sh.create_dir(&stage_dir)?;

        build_binary(sh)?;
        stage_package(&stage_dir, &version)?;

        let _pushd = sh.push_dir(&dist_dir);
        cmd!(sh, "click build ubports-click")
            .run()
            .with_context(|| format!("failed to run `click build`; {CLICK_INSTALL_HINT}"))?;

        Ok(())
    }
}

fn ensure_click_available(sh: &Shell) -> Result<()> {
    let _ = sh;

    match Command::new("click").arg("--help").status() {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => {
            bail!("`click --help` exited with status {status}; verify the Click CLI installation")
        }
        Err(err) if err.kind() == ErrorKind::NotFound => {
            Err(err).with_context(|| format!("failed to find `click`; {CLICK_INSTALL_HINT}"))
        }
        Err(err) => Err(err).context("failed to run `click --help`"),
    }
}

fn build_binary(sh: &Shell) -> Result<()> {
    cmd!(sh, "cargo build --target {TARGET} --release --locked").run()?;
    Ok(())
}

fn stage_package(stage_dir: &Path, version: &str) -> Result<()> {
    let packaging_dir = project_root().join("packaging/ubports-click");
    let framework = click_framework()?;
    let policy_version = policy_version(&framework);

    copy_file(
        &packaging_dir.join("gurk.apparmor"),
        &stage_dir.join("gurk.apparmor"),
    )?;
    copy_file(
        &packaging_dir.join("gurk.desktop"),
        &stage_dir.join("gurk.desktop"),
    )?;
    copy_file(
        &packaging_dir.join("gurk-launch"),
        &stage_dir.join("gurk-launch"),
    )?;
    copy_file(&packaging_dir.join("icon.svg"), &stage_dir.join("icon.svg"))?;

    let launcher = stage_dir.join("gurk-launch");
    set_executable(&launcher)?;

    let binary = Path::new("target")
        .join(TARGET)
        .join("release")
        .join(APP_NAME);
    copy_file(&binary, &stage_dir.join(APP_NAME))?;
    set_executable(&stage_dir.join(APP_NAME))?;

    let manifest = manifest(version, &framework);
    fs::write(stage_dir.join("manifest.json"), manifest)?;
    fs::write(
        stage_dir.join("gurk.apparmor"),
        apparmor_policy(&policy_version),
    )?;

    Ok(())
}

fn package_version() -> Result<String> {
    let cargo_toml = fs::read_to_string(project_root().join("Cargo.toml"))?;
    let value: Table = toml::from_str(&cargo_toml)?;
    value
        .get("package")
        .and_then(toml::Value::as_table)
        .and_then(|package| package.get("version"))
        .and_then(toml::Value::as_str)
        .map(ToOwned::to_owned)
        .context("failed to read package.version from Cargo.toml")
}

fn click_framework() -> Result<String> {
    let requested = std::env::var("GURK_CLICK_FRAMEWORK")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(|value| value.trim().to_owned());
    let available = available_frameworks()?;

    if let Some(requested) = requested {
        if available.iter().any(|framework| framework == &requested) {
            return Ok(requested);
        }

        bail!(
            "requested click framework `{requested}` is not installed. Available frameworks: {}. \
             Install the matching UBports framework or set GURK_CLICK_FRAMEWORK to one of the installed values.",
            format_frameworks(&available),
        );
    }

    if available.iter().any(|framework| framework == DEFAULT_FRAMEWORK) {
        return Ok(DEFAULT_FRAMEWORK.to_owned());
    }

    if available.len() == 1 {
        return Ok(available[0].clone());
    }

    bail!(
        "no usable default click framework found. Preferred framework `{DEFAULT_FRAMEWORK}` is not installed. \
         Available frameworks: {}. Install `{DEFAULT_FRAMEWORK}` or set GURK_CLICK_FRAMEWORK explicitly.",
        format_frameworks(&available),
    );
}

fn available_frameworks() -> Result<Vec<String>> {
    let output = Command::new("click")
        .args(["framework", "list"])
        .output()
        .context("failed to run `click framework list`")?;

    if !output.status.success() {
        bail!(
            "`click framework list` exited with status {}; verify the Click CLI installation",
            output.status
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect())
}

fn format_frameworks(frameworks: &[String]) -> String {
    if frameworks.is_empty() {
        "none".to_owned()
    } else {
        frameworks.join(", ")
    }
}

fn policy_version(framework: &str) -> String {
    framework
        .strip_prefix("ubuntu-sdk-")
        .map(str::to_owned)
        .unwrap_or_else(|| DEFAULT_POLICY_VERSION.to_owned())
}

fn manifest(version: &str, framework: &str) -> String {
    format!(
        concat!(
            "{{\n",
            "  \"name\": \"{PACKAGE_NAME}\",\n",
            "  \"version\": \"{version}\",\n",
            "  \"architecture\": \"arm64\",\n",
            "  \"title\": \"Gurk\",\n",
            "  \"description\": \"Signal messenger client for terminal\",\n",
            "  \"maintainer\": \"boxdot <d@zerovolt.org>\",\n",
            "  \"framework\": \"{framework}\",\n",
            "  \"hooks\": {{\n",
            "    \"gurk\": {{\n",
            "      \"apparmor\": \"gurk.apparmor\",\n",
            "      \"desktop\": \"gurk.desktop\"\n",
            "    }}\n",
            "  }}\n",
            "}}\n"
        ),
        PACKAGE_NAME = PACKAGE_NAME,
        version = version,
        framework = framework,
    )
}

fn apparmor_policy(policy_version: &str) -> String {
    format!(
        concat!(
            "{{\n",
            "  \"policy_version\": {policy_version},\n",
            "  \"policy_groups\": [\n",
            "    \"networking\"\n",
            "  ]\n",
            "}}\n"
        ),
        policy_version = policy_version,
    )
}

fn copy_file(src: &Path, dst: &Path) -> Result<()> {
    if !src.exists() {
        bail!("missing required file: {}", src.display());
    }

    fs::copy(src, dst)
        .with_context(|| format!("failed to copy {} to {}", src.display(), dst.display()))?;
    Ok(())
}

#[cfg(unix)]
fn set_executable(path: &PathBuf) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)?;
    Ok(())
}

#[cfg(not(unix))]
fn set_executable(_path: &PathBuf) -> Result<()> {
    Ok(())
}

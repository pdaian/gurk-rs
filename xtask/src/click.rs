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
const CLICKABLE_INSTALL_HINT: &str = "install Clickable first, for example with `pipx install clickable-ut`";

impl flags::Click {
    pub(crate) fn run(self, sh: &Shell) -> Result<()> {
        let version = package_version()?;
        let dist_dir = project_root().join("dist");
        let stage_dir = dist_dir.join("ubports-click");

        ensure_clickable_available()?;

        sh.create_dir(&dist_dir)?;
        sh.remove_path(&stage_dir)?;
        sh.create_dir(&stage_dir)?;

        build_binary(sh)?;
        stage_package(&stage_dir, &version)?;
        build_clickable_package(&stage_dir)?;
        collect_click_artifacts(&stage_dir, &dist_dir)?;

        Ok(())
    }
}

fn ensure_clickable_available() -> Result<()> {
    match Command::new("clickable").arg("--version").status() {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => {
            bail!("`clickable --version` exited with status {status}; verify the Clickable installation")
        }
        Err(err) if err.kind() == ErrorKind::NotFound => {
            Err(err).with_context(|| format!("failed to find `clickable`; {CLICKABLE_INSTALL_HINT}"))
        }
        Err(err) => Err(err).context("failed to run `clickable --version`"),
    }
}

fn build_binary(sh: &Shell) -> Result<()> {
    cmd!(sh, "cargo build --target {TARGET} --release --locked").run()?;
    Ok(())
}

fn stage_package(stage_dir: &Path, version: &str) -> Result<()> {
    let packaging_dir = project_root().join("packaging/ubports-click");
    let framework = clickable_framework();
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
    fs::write(stage_dir.join("clickable.yaml"), clickable_config(&framework))?;

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

fn clickable_framework() -> String {
    std::env::var("GURK_CLICK_FRAMEWORK")
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_FRAMEWORK.to_owned())
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

fn clickable_config(framework: &str) -> String {
    format!(
        concat!(
            "clickable_minimum_required: 8.0.0\n",
            "builder: pure\n",
            "framework: {framework}\n",
            "arch: arm64\n"
        ),
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

fn build_clickable_package(stage_dir: &Path) -> Result<()> {
    let status = Command::new("clickable")
        .arg("build")
        .current_dir(stage_dir)
        .status()
        .with_context(|| format!("failed to run `clickable build`; {CLICKABLE_INSTALL_HINT}"))?;

    if status.success() {
        Ok(())
    } else {
        bail!("`clickable build` exited with status {status}")
    }
}

fn collect_click_artifacts(stage_dir: &Path, dist_dir: &Path) -> Result<()> {
    let mut artifacts = Vec::new();
    collect_click_files(stage_dir, &mut artifacts)?;

    if artifacts.is_empty() {
        bail!("`clickable build` finished without producing a `.click` artifact")
    }

    for artifact in artifacts {
        let file_name = artifact
            .file_name()
            .context("click artifact is missing a file name")?;
        let destination = dist_dir.join(file_name);
        if artifact != destination {
            fs::copy(&artifact, &destination).with_context(|| {
                format!(
                    "failed to copy click artifact from {} to {}",
                    artifact.display(),
                    destination.display()
                )
            })?;
        }
    }

    Ok(())
}

fn collect_click_files(dir: &Path, artifacts: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            collect_click_files(&path, artifacts)?;
            continue;
        }

        if path.extension().is_some_and(|extension| extension == "click") {
            artifacts.push(path);
        }
    }

    Ok(())
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

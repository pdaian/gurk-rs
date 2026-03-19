use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use toml::Table;
use xshell::{Shell, cmd};

use crate::{flags, project_root};

const TARGET: &str = "aarch64-unknown-linux-gnu";
const FRAMEWORK: &str = "ubuntu-sdk-24.04";
const PACKAGE_NAME: &str = "gurk.boxdot";
const APP_NAME: &str = "gurk";

impl flags::Click {
    pub(crate) fn run(self, sh: &Shell) -> Result<()> {
        let version = package_version()?;
        let dist_dir = project_root().join("dist");
        let stage_dir = dist_dir.join("ubports-click");

        sh.create_dir(&dist_dir)?;
        sh.remove_path(&stage_dir)?;
        sh.create_dir(&stage_dir)?;

        build_binary(sh)?;
        stage_package(&stage_dir, &version)?;

        let _pushd = sh.push_dir(&dist_dir);
        cmd!(sh, "click build ubports-click --no-validate")
            .run()
            .context(
                "failed to run `click build`; install the Ubuntu Click CLI first, for example with `sudo apt-get install -y click`",
            )?;

        Ok(())
    }
}

fn build_binary(sh: &Shell) -> Result<()> {
    cmd!(sh, "cargo build --target {TARGET} --release --locked").run()?;
    Ok(())
}

fn stage_package(stage_dir: &Path, version: &str) -> Result<()> {
    let packaging_dir = project_root().join("packaging/ubports-click");

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

    let manifest = manifest(version);
    fs::write(stage_dir.join("manifest.json"), manifest)?;

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

fn manifest(version: &str) -> String {
    format!(
        concat!(
            "{{\n",
            "  \"name\": \"{PACKAGE_NAME}\",\n",
            "  \"version\": \"{version}\",\n",
            "  \"architecture\": \"arm64\",\n",
            "  \"title\": \"Gurk\",\n",
            "  \"description\": \"Signal messenger client for terminal\",\n",
            "  \"maintainer\": \"boxdot <d@zerovolt.org>\",\n",
            "  \"framework\": \"{FRAMEWORK}\",\n",
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
        FRAMEWORK = FRAMEWORK,
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

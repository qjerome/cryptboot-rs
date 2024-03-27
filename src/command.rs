use anyhow::anyhow;
use std::{ffi::OsStr, path::PathBuf, process::Stdio};

use crate::Device;

pub fn command<S: AsRef<OsStr>>(program: S) -> std::process::Command {
    let mut cmd = std::process::Command::new(program);
    cmd.env_clear().env("PATH", "/bin:/usr/bin");
    cmd
}

pub fn cryptsetup_open<S: AsRef<str>>(dev: &Device, name: S) -> anyhow::Result<()> {
    if !dev.is_valid() {
        return Err(anyhow!("cryptsetup open error invalid device: {}", dev));
    }
    let status = command("cryptsetup")
        .arg("open")
        .arg(dev.full_path())
        .arg(name.as_ref())
        .status()?;

    if !status.success() {
        return Err(anyhow!("cryptsetup open failed: {}", status));
    }
    Ok(())
}

pub fn cryptsetup_close<S: AsRef<OsStr>>(name: S, silent: bool) -> anyhow::Result<()> {
    let mut cmd = command("cryptsetup");

    cmd.arg("close").arg(name.as_ref());

    if silent {
        cmd.stderr(Stdio::null()).stdout(Stdio::null());
    }

    let status = cmd.status()?;

    if !status.success() {
        return Err(anyhow!("cryptsetup close failed: {}", status));
    }
    Ok(())
}

pub fn mount(dev: &Device, mountpoint: &PathBuf) -> anyhow::Result<()> {
    if !dev.is_valid() {
        return Err(anyhow!("mount error invalid device: {}", dev));
    }
    if !mountpoint.is_dir() {
        return Err(anyhow!(
            "mount invalid mountpoint: {}",
            mountpoint.to_string_lossy()
        ));
    }
    let status = command("mount")
        .arg(dev.full_path())
        .arg(mountpoint)
        .status()?;
    if !status.success() {
        return Err(anyhow!("failed to mount {}: {}", dev, status));
    }
    Ok(())
}

pub fn sbctl<S: AsRef<str>>(cmd: S) -> anyhow::Result<()> {
    let status = command("sbctl").arg(cmd.as_ref()).status()?;
    if !status.success() {
        return Err(anyhow!("sbctl {} failed: {status}", cmd.as_ref()));
    }
    Ok(())
}

pub fn umount(mountpoint: &PathBuf, args: &[&str]) -> anyhow::Result<()> {
    let status = command("umount").args(args).arg(mountpoint).status()?;
    if !status.success() {
        return Err(anyhow!("failed to umount: {}", status));
    }
    Ok(())
}

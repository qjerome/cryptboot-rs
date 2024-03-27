use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{command, Device};

const BOOT_MAPPER_NAME: &str = "cryptboot-boot";

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Config {
    // luks device containing boot
    pub device: PathBuf,
    pub mountpoint: PathBuf,
    pub efi: Efi,
}

#[derive(Debug, Clone)]
pub struct EncryptedBoot {
    pub config: Config,
    pub name: String,
    pub umount_on_drop: bool,
}

impl Default for EncryptedBoot {
    fn default() -> Self {
        Self {
            config: Default::default(),
            name: Default::default(),
            umount_on_drop: true,
        }
    }
}

impl Drop for EncryptedBoot {
    fn drop(&mut self) {
        if self.umount_on_drop {
            self.umount().unwrap()
        }
    }
}

impl EncryptedBoot {
    pub fn from_config(config: Config) -> Self {
        Self {
            config,
            name: BOOT_MAPPER_NAME.into(),
            umount_on_drop: false,
        }
    }

    pub fn umount_on_drop(mut self) -> Self {
        self.umount_on_drop = true;
        self
    }

    pub fn mount(&mut self) -> anyhow::Result<()> {
        // we mount encrypted partition
        command::cryptsetup_open(&Device::Path(self.config.device.clone()), BOOT_MAPPER_NAME)?;
        // we mount the decrypted device
        command::mount(&Device::Mapper(self.name.clone()), &self.config.mountpoint)?;
        // we mount efi
        self.config.efi.mount()?;
        Ok(())
    }

    pub fn umount(&self) -> anyhow::Result<()> {
        // we don't care a too much if this one fails
        let _ = self.config.efi.umount(&[]);
        // we always unmount everything
        command::umount(&self.config.mountpoint, &["-R"])?;
        command::cryptsetup_close(BOOT_MAPPER_NAME, false)
    }

    pub fn reset(&self) {
        // we don't care a too much if this one fails
        let _ = self.config.efi.umount(&["-qR"]);
        // we always unmount everything
        let _ = command::umount(&self.config.mountpoint, &["-qR"]);
        let _ = command::cryptsetup_close(BOOT_MAPPER_NAME, true);
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Efi {
    pub device: PathBuf,
    pub mountpoint: PathBuf,
}

impl Efi {
    fn mount(&self) -> anyhow::Result<()> {
        command::mount(&Device::Path(self.device.clone()), &self.mountpoint)
    }

    fn umount(&self, args: &[&str]) -> anyhow::Result<()> {
        command::umount(&self.mountpoint, args)
    }
}

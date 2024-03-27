use std::fs;

use crate::{boot, command::command};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};

const MODULES: &[&str] = &[
    // common modules
    "all_video",
    "boot",
    "btrfs",
    "cat",
    "chain",
    "configfile",
    "echo",
    "efifwsetup",
    "efinet",
    "ext2",
    "fat",
    "font",
    "gettext",
    "gfxmenu",
    "gfxterm",
    "gfxterm_background",
    "gzio",
    "halt",
    "help",
    "hfsplus",
    "iso9660",
    "jpeg",
    "keystatus",
    "loadenv",
    "loopback",
    "linux",
    "ls",
    "lsefi",
    "lsefimmap",
    "lsefisystab",
    "lssal",
    "memdisk",
    "minicmd",
    "normal",
    "ntfs",
    "part_apple",
    "part_msdos",
    "part_gpt",
    "password_pbkdf2",
    "png",
    "probe",
    "reboot",
    "regexp",
    "search",
    "search_fs_uuid",
    "search_fs_file",
    "search_label",
    "sleep",
    "smbios",
    "squash4",
    "test",
    "true",
    "video",
    "xfs",
    "zfs",
    "zfscrypt",
    "zfsinfo",
    // support for crypto
    "cryptodisk",
    "gcry_arcfour",
    "gcry_blowfish",
    "gcry_camellia",
    "gcry_cast5",
    "gcry_crc",
    "gcry_des",
    "gcry_dsa",
    "gcry_idea",
    "gcry_md4",
    "gcry_md5",
    "gcry_rfc2268",
    "gcry_rijndael",
    "gcry_rmd160",
    "gcry_rsa",
    "gcry_seed",
    "gcry_serpent",
    "gcry_sha1",
    "gcry_sha256",
    "gcry_sha512",
    "gcry_tiger",
    "gcry_twofish",
    "gcry_whirlpool",
    "luks",
    "lvm",
    "mdraid09",
    "mdraid1x",
    "raid5rec",
    "raid6rec",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Config {
    // Grub target
    pub target: String,
    pub bootloader_id: String,
    // additional modules
    pub add_modules: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            target: "x86_64-efi".into(),
            bootloader_id: "GRUB".into(),
            add_modules: vec![],
        }
    }
}

impl From<Config> for Grub {
    fn from(value: Config) -> Self {
        Self(value)
    }
}

pub(crate) struct Grub(Config);

impl Grub {
    pub fn from_config(config: Config) -> Self {
        config.into()
    }

    pub fn modules_for_target(&self, target: &str) -> Vec<String> {
        let mut modules: Vec<String> = MODULES.iter().map(|&s| String::from(s)).collect();

        match target {
            "x86_64-efi" | "i386-efi" => {
                modules.push("cpuid".into());
                modules.push("play".into());
                modules.push("tpm".into());
            }
            _ => {}
        }

        for add_mod in self.0.add_modules.iter() {
            if !modules.contains(add_mod) {
                modules.push(add_mod.clone())
            }
        }

        modules
    }

    pub fn mkconfig(&self, cfg: &boot::Config) -> anyhow::Result<()> {
        let grub_dir = cfg.mountpoint.join("grub");
        // create grub directory if it does not exists
        if !grub_dir.exists() {
            fs::create_dir(&grub_dir)?;
        }
        let status = command("grub-mkconfig")
            .arg("-o")
            .arg(grub_dir.join("grub.cfg"))
            .status()?;

        if !status.success() {
            return Err(anyhow!("grub-mkconfig failed: {}", status));
        }

        Ok(())
    }

    pub fn install(&self, cfg: &boot::Config) -> anyhow::Result<()> {
        let esp = &cfg.efi.mountpoint;

        if !esp.is_dir() {
            return Err(anyhow!(
                "esp directory not found: {}",
                esp.to_string_lossy()
            ));
        }

        let status = command("grub-install")
            .arg(format!("--target={}", self.0.target))
            .arg(format!("--efi-directory={}", esp.to_string_lossy()))
            .arg(format!("--bootloader-id={}", self.0.bootloader_id))
            .arg(format!(
                "--modules={}",
                self.modules_for_target(&self.0.target).join(" ")
            ))
            .arg("--disable-shim-lock")
            .status()?;

        if !status.success() {
            return Err(anyhow!("grub-install failed: {}", status));
        }
        Ok(())
    }
}

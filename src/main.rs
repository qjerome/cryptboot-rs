use std::{
    fs,
    os::unix::{self, fs::FileTypeExt},
    path::PathBuf,
};

use anyhow::anyhow;
use boot::EncryptedBoot;
use clap::{builder::styling, CommandFactory, FromArgMatches, Parser};
use fs_extra::dir::CopyOptions;
use grub::Grub;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

mod boot;
mod command;
mod grub;

#[allow(dead_code)]
enum Device {
    Path(PathBuf),
    PartUuid(Uuid),
    Mapper(String),
}

impl std::fmt::Display for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.full_path().to_string_lossy())
    }
}

impl Device {
    fn full_path(&self) -> PathBuf {
        match self {
            Self::Path(p) => p.clone(),
            Self::PartUuid(u) => {
                PathBuf::from("/dev/disk/by-partuuid").join(u.hyphenated().to_string())
            }
            Self::Mapper(s) => PathBuf::from("/dev/mapper").join(s),
        }
    }

    fn is_valid(&self) -> bool {
        match fs::metadata(self.full_path()) {
            Ok(m) => m.file_type().is_block_device(),
            _ => false,
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
struct Config {
    boot: boot::Config,
    grub: grub::Config,
}

struct Cryptboot(Config);

impl Cryptboot {
    fn from_config(config: Config) -> Self {
        Self(config)
    }

    fn mount(&self) -> anyhow::Result<EncryptedBoot> {
        let mut m = EncryptedBoot::from_config(self.0.boot.clone());
        m.reset();
        m.mount()?;
        Ok(m)
    }

    fn move_sbctl(&self) -> anyhow::Result<()> {
        let sbctl_dir = PathBuf::from("/usr/share/secureboot").canonicalize()?;
        let dst = self.0.boot.mountpoint.join("secureboot");

        let m = self.mount()?.umount_on_drop();

        // nothing to do
        if sbctl_dir == dst {
            return Ok(());
        }

        fs_extra::dir::move_dir(&sbctl_dir, &self.0.boot.mountpoint, &CopyOptions::new())?;
        // we create a symlink
        unix::fs::symlink(dst, sbctl_dir)?;

        drop(m);
        Ok(())
    }

    fn umount(&self) -> anyhow::Result<()> {
        let m = EncryptedBoot::from_config(self.0.boot.clone());
        m.umount()
    }

    fn grub_install(&self, o: GrubInstallOptions) -> anyhow::Result<()> {
        let m = self.mount()?.umount_on_drop();

        let grub = Grub::from_config(self.0.grub.clone());
        // update grub configuration
        grub.mkconfig(&self.0.boot)?;
        // install grub
        grub.install(&self.0.boot)?;

        // we sign all files
        if !o.no_sign {
            command::sbctl("sign-all")?;
        }

        drop(m);
        Ok(())
    }

    fn run(&self, o: RunOptions) -> anyhow::Result<()> {
        let m = self.mount()?.umount_on_drop();

        if !o.command_line.is_empty() {
            let program = &o.command_line[0];
            let mut cmd = command::command(program);
            if o.command_line.len() > 1 {
                cmd.args(&o.command_line[1..]);
            }
            let status = cmd.status()?;
            if !status.success() {
                return Err(anyhow!("failed to run {program}: {status}"));
            }
        }

        if o.sign_all {
            command::sbctl("sign-all")?;
        }

        drop(m);
        Ok(())
    }
}

#[derive(Debug, Parser)]
pub struct Args {
    /// configuration file
    #[clap(short, long, default_value_t = String::from("/etc/cryptboot/config.toml"))]
    config: String,
    #[clap(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Parser)]
enum Command {
    /// Create a configuration from command line
    Configure(ConfigureOption),
    /// Mount encrypted boot partition
    Mount,
    /// Unmount encrypted boot partition
    Umount,
    /// Install Grub in EFI mountpoint
    GrubInstall(GrubInstallOptions),
    /// Move sbctl files to encrypted boot partition and creates a symlink to /usr/share/secureboot
    MoveSbctl,
    /// Mount encrypted boot partition, run command and unmount
    Run(RunOptions),
}

#[derive(Debug, Parser)]

struct GrubInstallOptions {
    /// Do not sign grub after installation
    #[clap(long)]
    no_sign: bool,
}

#[derive(Debug, Parser)]
struct ConfigureOption {
    /// Path to a LUKS formated device used to store boot files
    #[clap(long)]
    boot_device: PathBuf,
    /// Path where boot partition will be mounted
    #[clap(long, default_value_t = String::from("/boot"))]
    boot_mountpoint: String,
    /// Path to the device holding your efi partition (accessible by UEFI)
    #[clap(long)]
    efi_device: PathBuf,
    /// Path where efi partition will be mounted
    #[clap(long, default_value_t= String::from("/boot/efi"))]
    efi_mountpoint: String,
}

impl From<ConfigureOption> for Config {
    fn from(value: ConfigureOption) -> Self {
        let mut c = Self {
            ..Default::default()
        };
        c.boot.device = value.boot_device;
        c.boot.mountpoint = value.boot_mountpoint.into();

        c.boot.efi.device = value.efi_device;
        c.boot.efi.mountpoint = value.efi_mountpoint.into();
        c
    }
}

#[derive(Debug, Parser)]
struct RunOptions {
    /// Run sbctl sign-all before unmounting (useful when running a system update)
    #[clap(short = 's', long)]
    sign_all: bool,
    /// Command line to run
    command_line: Vec<String>,
}

fn get_current_uid() -> libc::uid_t {
    unsafe { libc::getuid() }
}

fn main() -> Result<(), anyhow::Error> {
    let styles = styling::Styles::styled()
        .header(styling::AnsiColor::Green.on_default() | styling::Effects::BOLD)
        .usage(styling::AnsiColor::Green.on_default() | styling::Effects::BOLD)
        .literal(styling::AnsiColor::Blue.on_default() | styling::Effects::BOLD)
        .placeholder(styling::AnsiColor::Cyan.on_default());

    let a = Args::command().styles(styles).get_matches();
    let args = Args::from_arg_matches(&a)?;

    if let Some(Command::Configure(o)) = args.command {
        let c: Config = o.into();
        print!("{}", toml::to_string(&c)?);
        return Ok(());
    }

    if get_current_uid() != 0 && !matches!(args.command, Some(Command::Configure(_))) {
        return Err(anyhow!("this program needs to run as root"));
    }

    let config: Config = toml::from_str(
        &fs::read_to_string(&args.config)
            .map_err(|e| anyhow!("failed to read configuration file {}: {e}", &args.config))?,
    )?;

    let cryptboot = Cryptboot::from_config(config);

    if let Some(command) = args.command {
        match command {
            Command::Configure(_) => {}
            Command::Mount => cryptboot.mount().map(|_| ())?,
            Command::Umount => cryptboot.umount()?,
            Command::GrubInstall(o) => cryptboot.grub_install(o)?,
            Command::MoveSbctl => cryptboot.move_sbctl()?,
            Command::Run(o) => cryptboot.run(o)?,
        }
    }

    Ok(())
}

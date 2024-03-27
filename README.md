# Cryptboot

[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/qjerome/cryptboot-rs/rust.yml?style=for-the-badge)](https://github.com/qjerome/cryptboot-rs/actions/workflows/rust.yml)


Cryptboot helps managing Linux setup using a dedicated encrypted boot partition.
It aims at being the successor of [cryptboot (bash)](https://github.com/xmikos/cryptboot).

Boot partition does not need to be accessible all the time, there are actually a few moments (mainly boot and kernel update) where it needs to be mounted.
However, it might be a burden to always manage partition decryption, mount (`/boot` and `/boot/efi`) and unmount task manually. This is where cryptboot comes into play.
You can see it as a tool doing all the boring work for you, the only thing you'll need is providing your decryption password whenever you need
to do something on your encrypted boot partition.

Its main features are:
* easy mount/umount encrypted boot partitions
* grub installation supporting secure boot (bundling all the needed grub modules)
* allow to run commands on temporarily mounted encrypted boot partition (to be used for system updates)
* integrated with [sbctl](https://github.com/Foxboron/sbctl) to manage **secure boot** setup
  
Advantages of using a dedicated encrypted boot partition:
* only efi stub is accessible, all the rest (kernel, initramfs, grub configuration ...) is hidden in encrypted boot
* boot partition can be used as a vault to store secure boot signing keys

## Installation

```
git clone https://github.com/qjerome/cryptboot-rs
cd cryptboot-rs
./install.sh
```

## Usage

```
Usage: cryptboot [OPTIONS] [COMMAND]

Commands:
  configure     Create a configuration from command line
  mount         Mount encrypted boot partition
  umount        Unmount encrypted boot partition
  grub-install  Install Grub in EFI mountpoint
  harden-sbctl  Move sbctl /usr/share/secureboot directory to encrypted boot partition and creates a symlink to it. After that command sbctl will work only through cryptboot
  run           Mount encrypted boot partition, run command then unmount
  help          Print this message or the help of the given subcommand(s)

Options:
  -c, --config <CONFIG>  Path of configuration file [default: /etc/cryptboot/config.toml]
  -h, --help             Print help
```

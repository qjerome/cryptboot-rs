# Cryptboot

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
  
Advantages of using a dedicated encrypted boot partition:
* only efi stub is accessible, all the rest (kernel, initramfs, grub configuration ...) is hidden in encrypted boot
* boot partition can be used as a vault to store secure boot signing keys

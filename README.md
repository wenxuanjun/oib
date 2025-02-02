# OIB

OS Image Builder? It is just a simple `GPT+FAT` image builder.

## Usage

```bash
./oib config.toml
```

## Config

```toml
output = "output.img"

[[files]]
source = "build/kernel"
dest = "kernel"

[[files]]
source = "assets/BOOTX64.EFI"
dest = "efi/boot/bootx64.efi"

[[files]]
source = "assets/limine.conf"
dest = "limine.conf"

[[folders]]
source = "assets/static"
dest = "static"
```

## Acknowledgement

The code is based on [bootloader](https://github.com/rust-osdev/bootloader), a great pure-rust x86 bootloader.

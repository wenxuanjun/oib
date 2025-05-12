# OIB

OS Image Builder? It is just a simple `GPT+FAT` image builder.

## Usage

You can pass a config file:

```bash
./oib -c config.toml
```

Or pass your arguments directly:

```bash
./oib -o output.img -f build/kernel:kernel -d assets/static:static

# Combine config file with additional command line options
./oib -c config.toml -f additional_file.txt:extra.txt -d test/dir_name:dir_name
```

### Available Options

- `-o, --output`: Output image path
- `-c, --config`: Config file path
- `-f, --file`: Add a file to the image (format: source:destination)
- `-d, --dir`: Add a folder to the image (format: source:destination)

Command line arguments take precedence over configuration file settings when both are provided.

### Example Config

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

### Acknowledgement

The code is based on [bootloader](https://github.com/rust-osdev/bootloader), a great pure-rust x86 bootloader.

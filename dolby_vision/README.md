Library to read & write Dolby Vision metadata.  
Comes as a Rust crate and C compatible library.  

See [changelog](CHANGELOG.md) for API changes.

&nbsp;

### Toolchain

The minimum Rust version to use `dolby_vision` is 1.60.0.

&nbsp;

### `libdovi`, C-API

Packages
- **Arch Linux**: available on the AUR, `libdovi` or `libdovi-git`.

&nbsp;

#### Building the library

`libdovi` comes as a C compatible library.  
To build and install it you can use [cargo-c](https://crates.io/crates/cargo-c):

```sh
cargo install cargo-c
cargo cinstall --release
```

#### Running the C-API example
```sh
cd examples
gcc capi_rpu_file.c -ldovi -o capi_example.o
./capi_example.o
```

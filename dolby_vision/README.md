Library to read & write Dolby Vision metadata.  
Comes as a Rust crate and C compatible library.  

See [changelog](CHANGELOG.md) for API changes.

&nbsp;

### Toolchain

The minimum Rust version to use `dolby_vision` is 1.56.0.

&nbsp;

### Building the C-API

To build and install it you can use [cargo-c](https://crates.io/crates/cargo-c):

```sh
cargo install cargo-c
cargo cinstall --release
```

### Running the C-API example
```sh
cd examples
gcc capi.c -ldovi -o capi_example.o
./capi_example.o
```

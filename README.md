# Rust memfs

This branch adds in memory file system for `unknown` target. This is needed mostly for WebAssembly to populate file system in runtime, like this:

```rust
    File::create(file).expect(&format!("Unable to create {}", file))
        .write(include_bytes!("../file"))
        .expect(&format!("Unable to write in {}", file));
``` 

## Pre-built binaries

Building can take long time, but if your host `x86_64-unknown-linux-gnu` then you can download pre-built binaries in [Releases](https://github.com/caiiiycuk/rust-memfs/releases) section.

## Building

You need to build rustc compiler it self, basically it can be done with this command:

```
./x.py build -i --keep-stage 0 library/std --target wasm32-unknown-unknown --target x86_64-unknown-linux-gnu
```

**It's important** to pass two targets `wasm32-unknown-unknown` and your **host target**, because wasm32 can't be built without host tools.

If you have any problem, please consult with rustc dev [documentation](https://rustc-dev-guide.rust-lang.org/building/how-to-build-and-run.html)

## Setup

When rustc is compiled you should add it as toolchain with rustup:

```sh
rustup toolchain link memfs build/<host-triple>/stage1
```

Next you should override toolchain for project where you want to use `memfs`:

```sh
cd <project>
rustup override set memfs
```

After that you can build your porject for `wasm32-unknown-unknown` and create/write/read files.

## Discalimer

The implemenation if memory FS is not complete I created it only for [vange-rs](https://github.com/kvark/vange-rs) project.

# mini-starcoinvm
a standalone starcoin vm used in layer2

## Get Started

By default, data will be pulled from the remote, and written to the files
```shell
cargo build --release
```

Then, you can validate the data in file mode
```shell
cargo build --release --no-default-features
```

you can get more details by typing -h:
```shell
cargo run -- -h
```

## Cross Compile

1. Add mips-unknown-linux-musl supports:
```shell
rustup target add mips-unknown-linux-musl
```
2. Download musl toolchain from [musl.cc](https://musl.cc): mips-linux-musl-cross
3. Release:
```shell
cargo build --target mips-unknown-linux-musl --release --no-default-features
```
4. Find the bin:
```text
target\mips-unknown-linux-musl\release
```
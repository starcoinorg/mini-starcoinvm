# mini-starcoinvm
a standalone starcoin vm used in layer2

## Get Started

By default, data will be pulled from the remote, and written to the files
```shell
cargo build --release
```

Then, you can validate the data in file mode
```shell
cargo build --release --no-default-features --features from_file
```

you can get more details by typing -h:
```shell
cargo run -- -h
```

You can make cross-compile to mips like this:

```shell
cross build --release -v --no-default-features --features from_file
```

And find the bin at:
```text
target\mips-unknown-linux-musl\release
```
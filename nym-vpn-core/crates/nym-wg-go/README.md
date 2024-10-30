# Rust Wireguard-go Wrapper

This library wraps `wireguard-go` making it avaiable for transparent use to other
rust crates.

## Usage

Using this crate requires building the `libwg.a` library file and `libwg.h` header
file. 

```toml
nym-wg-go.workspace = true
```

```sh
# In the root of the repo
make build-wireguard

# build this library (or downstream projects)
cargo build -p nym-wg-go
```

### Amnezia

To use the AmneziaWG version we need to adjust the `libwg,a` file that gets built
to use the altered golang wrapper. Also you will need to enable the `amnezia` feature
in this crate.

```toml
nym-wg-go = { workspace=true, features=["amnezia"]}
```

```sh
# In the root of the repo
make build-wireguard-amnezia

# build this library (or downstream projects) in the same way as before.
cargo build -p nym-wg-go
```

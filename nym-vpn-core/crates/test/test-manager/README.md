# Nym preface

Code in this crate and accompanying crates
- test-manager
- test-rpc (library)
- test-runner


has been adapter to work with nym-vpnd instead of mullvad VPN daemon.

Structure of original code has been kept to a large extent, for several reasons
- in case of issues, to more easily compare our forked code to upstream and refer to it
- for further maintenance, to keep up-to-date with the fork


## TODO list
- `mullvad-management-interface` should be replaced with nym-vpn-proto as a dependency
    - currently Nym code in `mullvad-management-interface` was copied from
    `nym-vpn-proto` because of dependency hell trying to integrate it
    - eliminating mullvad code parts in that crate could help alleviate those
    dependency issues, helping us integrate it properly as a dependency
- bootstrap scripts for launching the test suite could be parametrized with a
desired nym-vpn published version so that it could be modified in one place and "just work"
- nym code should be built in a docker for reproducibility, as mullvad code currently is
- docker image used to build test-manager could be replaced with open source / in-house one


# Writing tests for [MullvadVPN App](https://github.com/mullvad/mullvadvpn-app/)

The `test-manager` crate is where end-to-end tests for the [MullvadVPN
App](https://github.com/mullvad/mullvadvpn-app/) resides. The tests are located
in different modules under `test-manager/src/tests/`.

## Getting started

Tests are regular Rust functions! Except that they are also `async` and marked
with the `#[test_function]` attribute

```rust
#[test_function]
pub async fn test(
    rpc: ServiceClient,
    mut mullvad_client: mullvad_management_interface::MullvadProxyClient,
) -> Result<(), Error> {
    Ok(())
}
```

The `test_function` macro allows you to write tests for the MullvadVPN App in a
format which is very similiar to [standard Rust unit
tests](https://doc.rust-lang.org/book/ch11-01-writing-tests.html). A more
detailed writeup on how the `#[test_function]` macro works is given as a
doc-comment in [test_macro::test_function](./test_macro/src/lib.rs).

If a new module is created, make sure to add it in
`test-manager/src/tests/mod.rs`.

### UI/Graphical tests

It is possible to write tests for asserting graphical properties in the app, but
this is a slightly more involved process. GUI tests are written in `Typescript`,
and reside in the `desktop/packages/mullvad-vpn/test/e2e` folder in the app repository.
Packaging of these tests is also done from the `desktop/packages/mullvad-vpn/` folder.

Assuming that a graphical test `gui-test.spec` has been bundled correctly, it
can be invoked from any Rust function by calling
`test_manager::tests::ui::run_test(rpc:
.., params: ..) -> Result<ExecResult,
Error>`

```rust
// Run a UI test. Panic if any assertion in it fails!
test_manager::tests::ui::run_test(&rpc, &["gui-test.spec"]).await.unwrap()
```

# Configuring `test-manager`

`test-manager` uses a configuration file to keep track of available virtual machines it can use for testing purposes.

More details can be found in [this configuration format document](./docs/config.md).

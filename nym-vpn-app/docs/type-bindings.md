## Type bindings

[ts-rs](https://github.com/Aleph-Alpha/ts-rs) is used to generate
TypeScript type definitions from Rust types

The UI web app (Js) is consuming typed data from tauri (Rust).
In order to enforce end-to-end typing across boundary, TypeScript
types are generated from Rust (eg. `struct`s and `enum`s).

To generate ts types run

```shell
npm run tsgen
# or
cd src-tauri && cargo test
```

This will generate all the types in `src/types/tauri.ts`. This
file is git tracked. Be sure to commit new changes.

### During dev iterations

See how to [annotate](https://github.com/Aleph-Alpha/ts-rs/blob/main/example/src/lib.rs)
Rust types.

When introducing changes in Rust side impacting shared types,
you must re-generate them, then handle any breaking.

For example, run the following commands to check for any ts errors

```shell
npm run tsgen
npm run tscheck
```

### CI

In CI a check is made to ensure that ts code is not broken,
including the generated types.
If CI is red on this check -> UI is broken!

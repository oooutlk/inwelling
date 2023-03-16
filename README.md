# Project Goal

To provide a mechanism for upstream crates to collect information from
downstream crates.

# Use cases

## Case 1, anonymous struct

Rust does not provide anonymous struct yet. To simulate this feature, we want to
define the struct's fields and access to its value at the same place. The
upstream crate providing anonymous struct simulation must know all the ad-hoc
`struct`s among its downstream crates source code, to define them before
downstream crates start to compile.

## Case 2, get rid of the multiple `-sys` crates

It would be nice if we have a unified method of compiling APIs from
"the C world" instead of maintain multiple `-sys` crates. And same APIs which
are defined in the same header files may be separated in different `-sys`
crates, e.g. `tcl_sys::TCL_OK` and `tk_sys::TCL_OK`.  Some crate such as
[cib](https://crates.io/crates/clib) tries provide a unified method to compile C
libraries and provides a unified namespace `clib::`.

# Information collected from downstream crates

Invoking `collect_downstream()` will collect the following information from
crates which called `register()` in its `build.rs`.

- Package name.

- Metadata defined in `Cargo.toml`.

- Manifest paths of `Cargo.toml`.

- Source file paths(optional). Call `collect_downstream()` with the argument
`inwelling::Opt::dump_rs_paths == true` to collect.

# Quickstart

1. The upstream crate e.g. `crate foo` calls `inwelling::collect_downstream()`
in its `build.rs` and do whatever it want to generate APIs for downstream.

2. The downstream crate e.g. `crate bar` calls `inwelling::register()` in its
`build.rs`.

```rust
// build.rs
fn main() { inwelling::register(); }
```

To send some metadata to upstream, encode them in `Cargo.toml`'s package metadata.

```toml
[package.metadata.inwelling.foo]
answer = { type = "integer", value = "42" }
```

# License

Under Apache License 2.0 or MIT License, at your will.

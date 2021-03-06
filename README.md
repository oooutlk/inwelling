# Problem To Solve

Sometimes a crate needs to gather information from its downstream users.

Frequently used mechanisms:

- Cargo features.

  The are friendly to cargo tools but not applicable for passing free contents
  because they are predefined options.

- Environment variables.

  They can pass free contents, but are not friendly to cargo tools.

# Project Goal

To provide a mechanism that is both friendly to cargo tools and able to pass
free contents.

# Library Overview

This library helps to send metadata through the hierarchy of crates, from
downstream crates to one of their common ancestor.

The main API is `inwelling()`, which is expected to be called in `build.rs` of
the common ancestor crate.

```text
.      +--------------> [topmost crate]
.      |      3            |       ^
.      |                  4|       |8
.      |                   |       |
.      |                 [dependencies]
.      |2                  |       |
.      |                   |       |
.      |        (metadata) |5     7| (API)
.      |                   |       |
.      |        1          v   6   |
. inwelling() <---- build.rs ----> bindings.rs
.[inwelling crate]     [common ancestor]
```

The information in section `[package.metadata.inwelling.{common ancestor}.*]`
in downstream crates' Cargo.toml files will be collected by `inwelling()`.

# Examples

See this [demo](https://github.com/oooutlk/inwelling/tree/main/examples/)
for more.

The `echo` crate has build-dependency of inwelling crate:

```toml
[build-dependencies]
inwelling = { path = "../.." }
```

And provides `echo()` which simply returns what it recieves as strings.

In `build.rs`:

```rust
use inwelling::*;

use std::{env, fs, path::PathBuf};

fn main() {
    let metadata_from_downstream = inwelling( Opts::default() )
        .sections
        .into_iter()
        .fold( String::new(), |acc, section|
            format!( "{}{:?} <{}>: {}\n"
                , acc
                , section.manifest
                , section.pkg
                , section.metadata.to_string() ));

    let out_path = PathBuf::from( env::var( "OUT_DIR" )
        .expect( "$OUT_DIR should exist." )
    ).join( "metadata_from_downstream" );

    fs::write(
        out_path,
        metadata_from_downstream
    ).expect( "metadata_from_downstream generated." );
}
```

In `lib.rs`:

```rust
pub fn echo() -> String {
    include_str!( concat!( env!( "OUT_DIR" ), "/metadata_from_downstream" ))
        .to_owned()
}
```

The gamma crate depends on alpha crate and conditionally depends on beta crate.
The beta crate depends on alpha crate. The alpha, beta and gamma ccrates all
depend on echo crate.

```text
.      +---------------> [gamma crate]    gamma=true
.      |                   .       ^           ^
.      |       gamma=true  .       |           |
.      |                   .       |           |
.      |            [beta crate]   |       beta=true
.      |                   |       |           |
.      |        beta=true  |       |           |
.      |                   |       |           |
.      |                 [alpha crate]    alpha=true
.      |                   |       |           |
.      |       alpha=true  |       |           |
.      |                   v       |           |
. inwelling() <---- build.rs ----> `echo()`----+
.[inwelling crate]       [echo crate]
```

In alpha crate's test code:

```rust,no_run
pub fn test() {
    let metadata = echo::echo();
    assert!( metadata.find("<alpha>: {\"alpha\":true}\n").is_some() );
}
```

# Optional Metadata

Cargo features can control whether to send metadata or not. in section
`[package.metadata.inwelling-{common ancestor}]`, a value of `feature = blah`
means that the metadata will be collected by inwelling if and only if blah
feature is enabled. See beta crate in examples for more.

# Other information collected from downstream crates

The following information are also collected:

- Package names.

- Cargo.toml files' paths.

- Optional .rs file paths. Call `inwelling()` with the argument
`inwelling::Opt::dump_rs_paths == true` to collect.

# Caveat

## Reverse Dependency

Collecting metadata from downstream and utilizing it in build process makes a
crate depending on its downstream crates. Unfortunately this kind of
reverse-dependency is not known to cargo. As a result, the changing of feature
set will not cause recompilation of the crate collecting metadata, which it
should.

To address this issue, simply do `cargo clean`, or more precisely,
`cargo clean --package {crate-collecting-metadata}` before running
`cargo build`. Substitute `{crate-collecting-metadata}` with actual crate name,
e.g. `cargo clean --package echo` in the examples above.

## Lacking Of `PWD` Environment Variable On Windows

Without official support from cargo, this library requires environment variable
such as `PWD` to locate topmost crate's Cargo.toml. Unfortunately `PWD` is
missing on Windows platform. This library will panic if it is feeling no luck to
locate Cargo.toml. However, `PWD` is not mandatory, unless `inwelling()` told
you so.

# License

Under Apache License 2.0 or MIT License, at your will.

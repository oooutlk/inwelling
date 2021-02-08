This crate demonstrates the usage of
[inwelling crate](https://crates.io/crates/inwelling). The `echo::echo()` simply
echos the metadata collected from its downstream users.

Conceptually this crate depends on its downstream crates which provide the
metadata in their manifests. Since this kind of reverse dependency is not
recognized by cargo, running `cargo clean --package echo` is required if the
manifests or cargo feature set has been changed.

The alpha, beta, gamma crates use this crate and test the echoed metadata.

This crate embeds metadata in its manifest:

```toml
[package.metadata.inwelling.echo]
gamma = true
```

which will be echoed in `echo::echo()`.

Since this crate has dependency of the alpha crate, `echo::echo()` also echoes
metadata embeded in alpha crate's manifest:

```toml
[package.metadata.inwelling.echo]
alpha = true
```

Since this crate has an optional dependency of the alpha crate, `echo::echo()`
also echoes metadata embeded in beta crate's manifest, as long as
`-all-features` or `--features beta` is provided in cargo command:

```toml
[package.metadata.inwelling.echo]
beta = true
```

Use `cargo test -- --nocapture` to check the metadata, and use
`cargo clean --package echo` then `cargo test --features beta -- --nocapture` to
check it again.

Note that "alpha = true" and "beta = true" will not be echoed twice in
`echo::echo()`, whether feature beta is on or off.

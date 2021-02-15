This crate embeds metadata in its manifest:

```toml
[package.metadata.inwelling.echo]
beta = true
```

which will be echoed in `echo::echo()`.

Since this crate has dependency of the alpha crate, `echo::echo()` also echoes
metadata embeded in alpha crate's manifest:

```toml
[package.metadata.inwelling.echo]
alpha = true
```

Use `cargo test -- --nocapture` to check the metadata.

Note that "alpha = true" will not be echoed twice in `echo::echo()`.

Use `cargo test --no-default-features` to demonstrate optional metadata.

This crate embeds metadata in its manifest:

```toml
[package.metadata.inwelling.echo]
alpha = true
```

which will be echoed in `echo::echo()`.

Use `cargo test -- --nocapture` to check the metadata.

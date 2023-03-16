This crate embeds metadata in its manifest:

```toml
[package.metadata.inwelling.echo]
beta = true
```

which will NOT be echoed in `echo::echo()`, because the lack of `build.rs`
containing `inwelling::collect_downstream()`.

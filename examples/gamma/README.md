This crate embeds metadata in its manifest:

```toml
[package.metadata.inwelling.echo]
gamma = "the third letter"
```

which will be echoed in `echo::echo()`.

Since this crate has dependency of the alpha crate, `echo::echo()` also echoes
metadata embeded in alpha crate's manifest:

```toml
[package.metadata.inwelling.echo]
alpha = "the first letter"
```

Note that "alpha = true" will not be echoed twice in `echo::echo()`, whether
feature beta is on or off.

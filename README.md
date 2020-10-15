# Tokio Compat 0.2

```toml
tokio-compat-02 = "0.1"
```

## Example

```rust
use tokio_compat_02::FutureExt;

// Run any tokio 02 future in any other runtime.
my_tokio_02_future.compat().await
```
# Tokio Compat 0.2

```toml
tokio-compat-02 = "0.1"
```

This crate includes utilities around integrating Tokio with other runtimes
by allowing the context to be attached to futures. This allows spawning
futures on other executors while still using Tokio to drive them. This
can be useful if you need to use a Tokio based library in an executor/runtime
that does not provide a Tokio context.

Be aware that the `.compat()` region allows you to use _both_ Tokio 0.2 and 0.3
features. It is _not_ the case that you opt-out of Tokio 0.3 when you are inside
a Tokio 0.2 compatibility region.

Basic usage:
```rust
use hyper::{Client, Uri};
use tokio_compat_02::FutureExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    // This will not panic because we are wrapping it in the
    // Tokio 0.2 context via the `FutureExt::compat` fn.
    client
        .get(Uri::from_static("http://tokio.rs"))
        .compat()
        .await?;

    Ok(())
}
```
Usage on async function:
```rust
use hyper::{Client, Uri};
use tokio_compat_02::FutureExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // By calling compat on the async function, everything inside it is able
    // to use Tokio 0.2 features.
    hyper_get().compat().await?;
    Ok(())
}

async fn hyper_get() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    // This will not panic because the `main` function wrapped it in the
    // Tokio 0.2 context.
    client
        .get(Uri::from_static("http://tokio.rs"))
        .await?;
}
```
Be aware that the constructors of some type require being inside the context. For
example, this includes `TcpStream` and `delay_for`.
```rust
use tokio_02::time::delay_for;
use tokio_compat_02::FutureExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Call the non-async constructor in the context.
    let time_future = async { delay_for(duration) }.compat().await;

    // Use the constructed `Delay`.
    time_future.compat().await;

    Ok(())
}
```
Of course the above would also work if the surrounding async function was called
with `.compat()`.

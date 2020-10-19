use hyper::{Client, Uri};
use tokio_compat_02::FutureExt;
use tokio_03 as tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    // This will not panic because we are wrapping it in the
    // Tokio 0.2 context.
    client
        .get(Uri::from_static("http://tokio.rs"))
        .compat()
        .await?;

    Ok(())
}

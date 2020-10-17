//! Tokio context aware futures utilities.
//!
//! This module includes utilities around integrating Tokio with other runtimes
//! by allowing the context to be attached to futures. This allows spawning
//! futures on other executors while still using tokio to drive them. This
//! can be useful if you need to use a tokio based library in an executor/runtime
//! that does not provide a tokio context.
//!
//! Be aware that the `.compat()` region allows you to use _both_ Tokio 0.2 and 0.3
//! features. It is _not_ the case that you opt-out of Tokio 0.3 when you are inside
//! a Tokio 0.2 compatibility region.
//!
//! Basic usage:
//! ```
//! use hyper::{Client, Uri};
//! use tokio_compat_02::FutureExt;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::new();
//!
//!     // This will not panic because we are wrapping it in the
//!     // Tokio 0.2 context via the `FutureExt::compat` fn.
//!     client
//!         .get(Uri::from_static("http://tokio.rs"))
//!         .compat()
//!         .await?;
//!
//!     Ok(())
//! }
//! ```
//! Usage on async function:
//! ```
//! use hyper::{Client, Uri};
//! use tokio_compat_02::FutureExt;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // By calling compat on the async function, everything inside it is able
//!     // to use Tokio 0.2 features.
//!     hyper_get().compat().await?;
//!     Ok(())
//! }
//!
//! async fn hyper_get() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::new();
//!
//!     // This will not panic because the `main` function wrapped it in the
//!     // Tokio 0.2 context.
//!     client
//!         .get(Uri::from_static("http://tokio.rs"))
//!         .await?;
//! }
//! ```
//! Be aware that the constructors of some type require being inside the context. For
//! example, this includes `TcpStream` and `delay_for`.
//! ```
//! use tokio_02::time::delay_for;
//! use tokio_compat_02::FutureExt;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Call the non-async constructor in the context.
//!     let time_future = async { delay_for(duration) }.compat().await;
//!
//!     // Use the constructed `Delay`.
//!     time_future.compat().await;
//!
//!     Ok(())
//! }
//! ```
//! Of course the above would also work if the surrounding async function was called
//! with `.compat()`.

#![doc(html_root_url = "https://docs.rs/tokio-compat-02/0.1.0")]

use once_cell::sync::OnceCell;
use pin_project_lite::pin_project;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tokio_02::runtime::{Handle, Runtime};

static RT: OnceCell<Runtime> = OnceCell::new();

pin_project! {
    /// `TokioContext` allows connecting a custom executor with the tokio runtime.
    ///
    /// It contains a `Handle` to the runtime. A handle to the runtime can be
    /// obtain by calling the `Runtime::handle()` method.
    pub struct TokioContext<F> {
        #[pin]
        inner: F,
        handle: Handle,
    }
}

impl<F: Future> Future for TokioContext<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let me = self.project();
        let handle = me.handle;
        let fut = me.inner;

        handle.enter(|| fut.poll(cx))
    }
}

/// Trait extension that simplifies bundling a `Handle` with a `Future`.
pub trait FutureExt: Future {
    /// Compat any future into a future that can run within the Tokio 0.2
    /// context.
    ///
    /// This will spawn single threaded scheduler in the background globally
    /// to allow you to call this extension trait method from anywhere. The
    /// background scheduler supports running Tokio 0.2 futures in any context
    /// with a small footprint.
    ///
    /// # Example
    ///
    /// ```rust
    /// # async fn my_future() {}
    /// # async fn foo() {
    /// use tokio_compat_02::FutureExt as _;
    ///
    /// my_future().compat().await;
    /// # }
    fn compat(self) -> TokioContext<Self>
    where
        Self: Sized;
}

impl<F: Future> FutureExt for F {
    fn compat(self) -> TokioContext<Self> {
        let handle = RT
            .get_or_init(|| {
                tokio_02::runtime::Builder::new()
                    .threaded_scheduler()
                    .core_threads(1)
                    .enable_all()
                    .build()
                    .unwrap()
            })
            .handle()
            .clone();

        TokioContext {
            inner: self,
            handle,
        }
    }
}

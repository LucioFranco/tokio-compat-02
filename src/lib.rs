//! Tokio context aware futures utilities.
//!
//! This module includes utilities around integrating tokio with other runtimes
//! by allowing the context to be attached to futures. This allows spawning
//! futures on other executors while still using tokio to drive them. This
//! can be useful if you need to use a tokio based library in an executor/runtime
//! that does not provide a tokio context.

#![doc(html_root_url = "https://docs.rs/tokio-compat-02/0.1.0")]

use once_cell::sync::OnceCell;
use pin_project_lite::pin_project;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tokio_02::runtime::{Handle, Runtime};

mod io;
pub use self::io::IoCompat;

static RT: OnceCell<Runtime> = OnceCell::new();
fn get_handle() -> Handle {
    RT
        .get_or_init(|| {
            tokio_02::runtime::Builder::new()
                .threaded_scheduler()
                .core_threads(1)
                .enable_all()
                .build()
                .unwrap()
        })
        .handle()
        .clone()
}

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
        TokioContext {
            inner: self,
            handle: get_handle(),
        }
    }
}

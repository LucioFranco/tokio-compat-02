use bytes::buf::BufMut;
use pin_project_lite::pin_project;
use std::{
    io::Result,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::{
    AsyncBufRead as AsyncBufRead1, AsyncRead as AsyncRead1, AsyncWrite as AsyncWrite1, ReadBuf,
};
use tokio_02::{
    io::{AsyncBufRead as AsyncBufRead02, AsyncRead as AsyncRead02, AsyncWrite as AsyncWrite02},
    runtime::Handle,
};
use tokio_stream::Stream;

pin_project! {
    /// `IoCompat` allows conversion between the 0.2 and 1.0 IO traits.
    ///
    /// By wrapping any Tokio IO type in this compatibility wrapper, it becomes usable
    /// with the traits of the other version of Tokio.
    pub struct IoCompat<T> {
        #[pin]
        inner: T,
        handle: Handle,
    }
}

impl<T> IoCompat<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            handle: crate::get_handle(),
        }
    }
}

impl<T: AsyncRead02> AsyncRead1 for IoCompat<T> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<()>> {
        let me = self.project();
        let handle = me.handle;
        let inner = me.inner;

        let unfilled = buf.initialize_unfilled();

        let poll = handle.enter(|| inner.poll_read(cx, unfilled));

        if let Poll::Ready(Ok(num)) = &poll {
            buf.advance(*num);
        }

        poll.map_ok(|_| ())
    }
}

impl<T: AsyncRead1> AsyncRead02 for IoCompat<T> {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context, buf: &mut [u8]) -> Poll<Result<usize>> {
        let mut read_buf = ReadBuf::new(buf);
        match self.project().inner.poll_read(cx, &mut read_buf) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(read_buf.filled().len())),
            Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_read_buf<B: BufMut>(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut B,
    ) -> Poll<Result<usize>>
    where
        Self: Sized,
    {
        let slice = buf.bytes_mut();
        let ptr = slice.as_ptr() as *const u8;
        let mut read_buf = ReadBuf::uninit(slice);
        match self.project().inner.poll_read(cx, &mut read_buf) {
            Poll::Ready(Ok(())) => {
                assert!(std::ptr::eq(ptr, read_buf.filled().as_ptr()));
                let len = read_buf.filled().len();
                unsafe {
                    buf.advance_mut(len);
                }
                Poll::Ready(Ok(len))
            }
            Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<T: AsyncWrite02> AsyncWrite1 for IoCompat<T> {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        let me = self.project();
        let handle = me.handle;
        let inner = me.inner;

        handle.enter(|| inner.poll_write(cx, buf))
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        let me = self.project();
        let handle = me.handle;
        let inner = me.inner;

        handle.enter(|| inner.poll_flush(cx))
    }
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        let me = self.project();
        let handle = me.handle;
        let inner = me.inner;

        handle.enter(|| inner.poll_shutdown(cx))
    }
}

impl<T: AsyncWrite1> AsyncWrite02 for IoCompat<T> {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        self.project().inner.poll_write(cx, buf)
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.project().inner.poll_flush(cx)
    }
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.project().inner.poll_shutdown(cx)
    }
}

impl<T: AsyncBufRead02> AsyncBufRead1 for IoCompat<T> {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<&[u8]>> {
        let me = self.project();
        let handle = me.handle;
        let inner = me.inner;

        handle.enter(|| inner.poll_fill_buf(cx))
    }
    fn consume(self: Pin<&mut Self>, amt: usize) {
        let me = self.project();
        let handle = me.handle;
        let inner = me.inner;

        handle.enter(|| inner.consume(amt))
    }
}

impl<T: AsyncBufRead1> AsyncBufRead02 for IoCompat<T> {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<&[u8]>> {
        self.project().inner.poll_fill_buf(cx)
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        self.project().inner.consume(amt)
    }
}

impl<T: Stream> Stream for IoCompat<T> {
    type Item = T::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<T::Item>> {
        let me = self.project();
        let handle = me.handle;
        let inner = me.inner;

        handle.enter(|| inner.poll_next(cx))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

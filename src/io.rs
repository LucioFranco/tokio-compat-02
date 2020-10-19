use tokio_02::runtime::Handle;
use tokio_02::io::{AsyncRead as AsyncRead02, AsyncBufRead as AsyncBufRead02, AsyncWrite as AsyncWrite02, AsyncSeek as AsyncSeek02};
use tokio_03::io::{AsyncRead as AsyncRead03, AsyncBufRead as AsyncBufRead03, AsyncWrite as AsyncWrite03, AsyncSeek as AsyncSeek03, ReadBuf};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::io::Result;
use bytes::buf::BufMut;
use pin_project_lite::pin_project;

pin_project! {
    /// `IoCompat` allows conversion between the 0.2 and 0.3 IO traits.
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

impl<T: AsyncRead02> AsyncRead03 for IoCompat<T> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>
    ) -> Poll<Result<()>> {
        let me = self.project();
        let handle = me.handle;
        let inner = me.inner;

        let unfilled = buf.initialize_unfilled();

        let num = handle.enter(|| {
            inner.poll_read(cx, unfilled).map_ok(|_num| ())
        });

        buf.advance(num);
    }
}

impl<T: AsyncRead03> AsyncRead02 for IoCompat<T> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8]
    ) -> Poll<Result<usize>> {
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
        buf: &mut B
    ) -> Poll<Result<usize>>
    where
        Self: Sized,
    {
        let slice = buf.bytes_mut();
        let mut read_buf = ReadBuf::uninit(slice);
        match self.project().inner.poll_read(cx, &mut read_buf) {
            Poll::Ready(Ok(())) => {
                assert!(std::ptr::eq(slice.as_ptr(), read_buf.filled().as_ptr()));
                let len = read_buf.filled().len();
                unsafe {
                    buf.advance_mut(len);
                }
                Poll::Ready(Ok(len))
            },
            Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<T: AsyncWrite02> AsyncWrite03 for IoCompat<T> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8]
    ) -> Poll<Result<usize>> {
        let me = self.project();
        let handle = me.handle;
        let inner = me.inner;

        handle.enter(|| {
            inner.poll_write(cx, buf)
        })
    }
    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<()>> {
        let me = self.project();
        let handle = me.handle;
        let inner = me.inner;

        handle.enter(|| {
            inner.poll_flush(cx)
        })
    }
    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<()>> {
        let me = self.project();
        let handle = me.handle;
        let inner = me.inner;

        handle.enter(|| {
            inner.poll_shutdown(cx)
        })
    }
}

impl<T: AsyncWrite03> AsyncWrite02 for IoCompat<T> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8]
    ) -> Poll<Result<usize>> {
        self.project().inner.poll_write(cx, buf)
    }
    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<()>> {
        self.project().inner.poll_flush(cx)
    }
    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<()>> {
        self.project().inner.poll_shutdown(cx)
    }
}

impl<T: AsyncBufRead02> AsyncBufRead03 for IoCompat<T> {
    fn poll_fill_buf(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>
    ) -> Poll<Result<&[u8]>> {
        let me = self.project();
        let handle = me.handle;
        let inner = me.inner;

        handle.enter(|| {
            inner.poll_fill_buf(cx)
        })
    }
    fn consume(self: Pin<&mut Self>, amt: usize) {
        let me = self.project();
        let handle = me.handle;
        let inner = me.inner;

        handle.enter(|| {
            inner.consume(amt)
        })
    }
}

impl<T: AsyncBufRead03> AsyncBufRead02 for IoCompat<T> {
    fn poll_fill_buf(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>
    ) -> Poll<Result<&[u8]>> {
        self.project().inner.poll_fill_buf(cx)
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        self.project().inner.consume(amt)
    }
}

impl<T: AsyncSeek02> AsyncSeek03 for IoCompat<T> {
    fn poll_fill_buf(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>
    ) -> Poll<Result<&[u8]>> {
        let me = self.project();
        let handle = me.handle;
        let inner = me.inner;

        handle.enter(|| {
            inner.poll_fill_buf(cx)
        })
    }
    fn consume(self: Pin<&mut Self>, amt: usize) {
        let me = self.project();
        let handle = me.handle;
        let inner = me.inner;

        handle.enter(|| {
            inner.consume(amt)
        })
    }
}

impl<T: AsyncSeek03> AsyncSeek02 for IoCompat<T> {
    fn poll_fill_buf(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>
    ) -> Poll<Result<&[u8]>> {
        self.project().inner.poll_fill_buf(cx)
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        self.project().inner.consume(amt)
    }
}

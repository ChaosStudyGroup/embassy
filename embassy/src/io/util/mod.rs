use core::cmp::min;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures::ready;

mod read;
pub use self::read::Read;

mod read_buf;
pub use self::read_buf::ReadBuf;

mod read_byte;
pub use self::read_byte::ReadByte;

mod read_exact;
pub use self::read_exact::ReadExact;

mod read_while;
pub use self::read_while::ReadWhile;

mod read_to_end;
pub use self::read_to_end::ReadToEnd;

mod skip_while;
pub use self::skip_while::SkipWhile;

mod write;
pub use self::write::Write;

mod write_all;
pub use self::write_all::WriteAll;

mod write_byte;
pub use self::write_byte::WriteByte;

#[cfg(feature = "alloc")]
mod split;
#[cfg(feature = "alloc")]
pub use self::split::{split, ReadHalf, WriteHalf};

mod copy_buf;
pub use self::copy_buf::{copy_buf, CopyBuf};

use super::error::Result;
use super::traits::{AsyncBufRead, AsyncWrite};

pub trait AsyncBufReadExt: AsyncBufRead {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize>>
    where
        Self: Unpin,
    {
        let mut this = &mut *self;
        let rbuf = ready!(Pin::new(&mut this).poll_fill_buf(cx))?;
        let n = min(buf.len(), rbuf.len());
        buf[..n].copy_from_slice(&rbuf[..n]);
        Pin::new(&mut this).consume(n);
        Poll::Ready(Ok(n))
    }

    fn read_while<'a, F: Fn(u8) -> bool>(
        &'a mut self,
        buf: &'a mut [u8],
        f: F,
    ) -> ReadWhile<'a, Self, F>
    where
        Self: Unpin,
    {
        ReadWhile::new(self, f, buf)
    }

    fn skip_while<'a, F: Fn(u8) -> bool>(&'a mut self, f: F) -> SkipWhile<'a, Self, F>
    where
        Self: Unpin,
    {
        SkipWhile::new(self, f)
    }

    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Read<'a, Self>
    where
        Self: Unpin,
    {
        Read::new(self, buf)
    }

    fn read_buf<'a>(&'a mut self) -> ReadBuf<'a, Self>
    where
        Self: Unpin,
    {
        ReadBuf::new(self)
    }

    fn read_byte<'a>(&'a mut self) -> ReadByte<'a, Self>
    where
        Self: Unpin,
    {
        ReadByte::new(self)
    }

    fn read_exact<'a>(&'a mut self, buf: &'a mut [u8]) -> ReadExact<'a, Self>
    where
        Self: Unpin,
    {
        ReadExact::new(self, buf)
    }

    fn read_to_end<'a>(&'a mut self, buf: &'a mut [u8]) -> ReadToEnd<'a, Self>
    where
        Self: Unpin,
    {
        ReadToEnd::new(self, buf)
    }
}

impl<R: AsyncBufRead + ?Sized> AsyncBufReadExt for R {}

pub async fn read_line<R: AsyncBufRead + Unpin>(r: &mut R, buf: &mut [u8]) -> Result<usize> {
    r.skip_while(|b| b == b'\r' || b == b'\n').await?;
    let n = r.read_while(buf, |b| b != b'\r' && b != b'\n').await?;
    r.skip_while(|b| b == b'\r').await?;
    //assert_eq!(b'\n', r.read_byte().await?);
    r.read_byte().await?;
    Ok(n)
}

pub trait AsyncWriteExt: AsyncWrite {
    fn write_all<'a>(&'a mut self, buf: &'a [u8]) -> WriteAll<'a, Self>
    where
        Self: Unpin,
    {
        WriteAll::new(self, buf)
    }

    fn write_byte<'a>(&'a mut self, byte: u8) -> WriteByte<'a, Self>
    where
        Self: Unpin,
    {
        WriteByte::new(self, byte)
    }
}

impl<R: AsyncWrite + ?Sized> AsyncWriteExt for R {}
#![cfg_attr(not(test), no_std)]
#![warn(missing_docs)]

//! Futures combinator to set the priority of a Future.

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use pin_project::pin_project;

/// Future to set priority.
#[pin_project]
pub struct Priority<T: Future> {
    #[pin]
    inner: T,
    prio: usize,
    waited: usize,
}

impl<T: Future> Future for Priority<T> {
    type Output = T::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let this = self.project();

        if this.waited >= this.prio {
            *this.waited = 0;
            this.inner.poll(cx)
        } else {
            *this.waited += 1;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

/// Extension trait to set priority on Futures.
pub trait PriorityExt: Future + Sized {
    /// Set the priority of a Future. A priority of 0 is equivalent to not calling this method, and
    /// higher priorities will cause a Future to be polled less often.
    fn priority(self, prio: usize) -> Priority<Self>;
}

impl<T: Future> PriorityExt for T {
    fn priority(self, prio: usize) -> Priority<Self> {
        Priority {
            inner: self,
            waited: 0,
            prio,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::future::Future;
    use core::pin::Pin;
    use core::task::{Context, Poll};
    use futures_executor::block_on;

    struct DummyFuture;

    impl Future for DummyFuture {
        type Output = ();

        fn poll(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
            Poll::Ready(())
        }
    }

    #[test]
    fn sanity_check() {
        block_on(async {
            assert_eq!(DummyFuture.await, ());
            assert_eq!(DummyFuture.priority(1).await, ());
            assert_eq!(DummyFuture.priority(2).await, ());
        });
    }
}

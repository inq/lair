use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

use hyper::body::{Bytes, Frame};

pub struct Body {
    // Our Body type is !Send and !Sync:
    _marker: PhantomData<*const ()>,
    data: Option<Bytes>,
}

impl From<&'static str> for Body {
    fn from(a: &'static str) -> Self {
        Body {
            _marker: PhantomData,
            data: Some(a.into()),
        }
    }
}

impl hyper::body::Body for Body {
    type Data = Bytes;
    type Error = hyper::Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        Poll::Ready(self.get_mut().data.take().map(|d| Ok(Frame::data(d))))
    }
}

use std::cell::RefCell;
use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

use http_body_util::combinators::UnsyncBoxBody;
use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::{Request, Response};
use tower_http::services::{ServeDir, ServeFile};
use tower_service::Service;

#[derive(Debug, Clone)]
pub struct LairService {
    counter: Rc<RefCell<i32>>,
    assets: ServeDir,
    index: ServeFile,
}

impl LairService {
    pub fn new() -> Self {
        Self {
            counter: Rc::new(RefCell::new(0)),
            assets: ServeDir::new(""),
            index: ServeFile::new("resources/index.html"),
        }
    }
}

impl hyper::service::Service<Request<Incoming>> for LairService {
    type Response = Response<UnsyncBoxBody<Bytes, std::io::Error>>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        fn mk_response(
            s: String,
        ) -> Result<Response<UnsyncBoxBody<Bytes, std::io::Error>>, Infallible> {
            Ok(Response::builder()
                .body(UnsyncBoxBody::new(Full::new(Bytes::from(s)).map_err(
                    |_| std::io::Error::new(std::io::ErrorKind::Other, "oh no!"),
                )))
                .unwrap())
        }

        if req.uri().path() != "/favicon.ico" {
            *self.counter.borrow_mut() += 1;
        }

        if req.uri().path().starts_with("/assets/") {
            let mut assets = self.assets.clone();
            return Box::pin(async move {
                let res = assets.call(req).await;
                let res = res.map(|resp| resp.map(|body| body.boxed_unsync()));
                println!("{:?}", res);
                res
            });
        }

        let res = match req.uri().path() {
            "/" => {
                let mut index = self.index.clone();
                return Box::pin(async move {
                    let res = index.call(req).await;
                    let res = res.map(|resp| resp.map(|body| body.boxed_unsync()));
                    println!("{:?}", res);
                    res
                });
            }
            _ => mk_response("not found".into()),
        };

        Box::pin(async { res })
    }
}

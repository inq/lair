use std::cell::RefCell;
use std::rc::Rc;
use sark::prelude::*;
use bytes::BytesMut;
use http;


pub struct LairHandler {
    counter: Rc<RefCell<i32>>,
}

impl Service<()> for LairHandler {
    async fn call(&self, req: Request, _: &()) -> Result<Response> {
        tracing::info!("{} {}", req.method(), req.uri().path());
        
        let result = match (req.method(), req.uri().path()) {
            (&http::Method::GET, "/") => self.index(req).await,
            (&http::Method::GET, "/count") => self.count(req).await,
            (_, path) if path.starts_with("/assets/") => self.assets(req).await,
            _ => Err(Error::NotFound),
        };
        
        if let Err(ref e) = result {
            tracing::error!("Error: {}", e);
        }
        
        result
    }
}

impl LairHandler {
    async fn index(&self, _req: Request) -> Result<Response> {
        tracing::debug!("Serving index page");
        let html_content = match std::fs::read_to_string("resources/index.html") {
            Ok(content) => content,
            Err(_) => return Err(Error::NotFound)
        };

        let mut resp = Response::ok();
        resp.headers_mut().insert(
            http::header::CONTENT_TYPE,
            http::HeaderValue::from_static("text/html")
        );
        resp.set_body_str(&html_content);
        Ok(resp)
    }

    async fn count(&self, req: Request) -> Result<Response> {
        tracing::debug!("Serving count page");
        if req.uri().path() != "/favicon.ico" {
            *self.counter.borrow_mut() += 1;
        }
        
        let mut resp = Response::ok();
        resp.set_body_str(&format!("Hello from Lair! Visit count: {}", self.counter.borrow()));
        Ok(resp)
    }

    async fn assets(&self, req: Request) -> Result<Response> {
        let path = req.uri().path();
        let asset_path = path.strip_prefix("/assets/").unwrap_or("");
        tracing::debug!("Serving asset: {}", asset_path);

        if asset_path == "fjalla-one.woff2" || asset_path == "nanum.woff2" {
            let file_path = format!("assets/{}", asset_path);
            let font_bytes = std::fs::read(file_path).map_err(|_| Error::NotFound)?;

            let mut resp = Response::ok();
            resp.headers_mut().insert(
                http::header::CONTENT_TYPE,
                http::HeaderValue::from_static("font/woff2")
            );
            resp.set_body(BytesMut::from(&font_bytes[..]));
            Ok(resp)
        } else {
            Err(Error::NotFound)
        }
    }
}

pub struct LairService;

impl LairService {
    pub fn new() -> impl Service<()> {
        let counter = Rc::new(RefCell::new(0));
        LairHandler { counter }
    }
}

use std::io::Cursor;

use actix_multipart::Multipart;
use actix_web::{http::StatusCode, HttpRequest, HttpResponse};

async fn upload(mut payload: Multipart) -> Result<HttpResponse, actix_web::Error> {
    use futures::StreamExt;

    while let Some(item) = payload.next().await {
        let mut field = item?;
        let mut chunks = vec![];
        while let Some(chunk) = field.next().await {
            chunks.push(chunk?);
        }
        let loaded = image::load_from_memory(&chunks.concat()).unwrap();
        let resized = loaded.resize(100, 100, image::imageops::FilterType::Lanczos3);
        let mut buf = Cursor::new(vec![]);
        resized
            .write_to(&mut buf, image::ImageFormat::Jpeg)
            .unwrap();

        return Ok(HttpResponse::build(StatusCode::OK).body(buf.into_inner()));
    }

    panic!();
}

async fn index(req: HttpRequest) -> Result<HttpResponse, actix_web::Error> {
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(
            r#"
            <html>
                <body>
                    <form method="post" action="/upload" enctype="multipart/form-data">
                        <input type="file" id="files" name="files" multiple>
                        <input type="submit">
                    </form>
                </body>
            </html>
            "#,
        ))
}

async fn main_async() -> Result<(), Error> {
    use actix_web::{App, HttpServer};

    HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .service(actix_web::web::resource("/upload").route(actix_web::web::post().to(upload)))
            .service(actix_web::web::resource("/").route(actix_web::web::get().to(index)))
    })
    .bind(("0.0.0.0", 8080))
    .map_err(Error::Bind)?
    .workers(1)
    .run()
    .await
    .map_err(Error::Actix)
}

fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    actix_web::rt::System::new().block_on(main_async())
}

#[derive(Debug)]
enum Error {
    Actix(std::io::Error),
    Bind(std::io::Error),
}

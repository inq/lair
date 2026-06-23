use std::cell::Cell;

use http::StatusCode;
use serde::Deserialize;

#[derive(Deserialize)]
struct Article {
    title: String,
    created_at: String,
    paragraphs: Vec<String>,
}

impl Article {
    fn all() -> Vec<Article> {
        serde_yaml::from_str(include_str!("../../resources/articles.yaml")).unwrap()
    }
}

struct WaveRng(u32);

impl WaveRng {
    fn next(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }

    fn range(&mut self, min: f64, max: f64) -> f64 {
        let t = (self.next() % 10000) as f64 / 10000.0;
        min + t * (max - min)
    }
}

struct Wave;

impl Wave {
    fn svg() -> String {
        use std::fmt::Write;

        let mut rng = WaveRng(0xDEAD_BEEF);
        let mut out = String::with_capacity(8192);

        for i in 0..10u32 {
            let y = 15.0 + 1.0 * i as f64;
            let base = if i % 2 == 0 { "fff" } else { "dde" };
            let alpha = char::from_digit(i + 1, 16).unwrap();
            let fill = format!("#{}{}", base, alpha);
            let sw = rng.range(0.12, 0.37);

            let mut amps = Vec::new();
            let mut period = 0.0_f64;
            while period < 120.0 {
                let a = rng.range(6.0, 15.0);
                amps.push(a);
                period += a * 2.0;
            }
            if amps.len() % 2 != 0 {
                let a = rng.range(6.0, 15.0);
                amps.push(a);
                period += a * 2.0;
            }

            let mut beziers = String::new();
            for (j, &a) in amps.iter().enumerate() {
                let cy = if j % 2 == 0 { -a } else { a };
                let _ = write!(beziers, " q{:.2} {:.2},{:.2} 0", a, cy, a * 2.0);
            }

            let vw = period * 2.0;
            let _ = writeln!(
                out,
                "<svg class=\"wave wave-{}\" style=\"--wv:{:.2}\" xmlns=\"http://www.w3.org/2000/svg\" \
                 viewBox=\"0 0 {:.2} 50\" preserveAspectRatio=\"none\">\
                 <path fill=\"{}\" d=\"M0,{:.2}{beziers}{beziers} L{:.2},{:.2}V50H0Z\"/>\
                 <path fill=\"none\" stroke=\"#333{}\" stroke-width=\"{:.2}\" d=\"M0,{:.2}{beziers}{beziers}\"/>\
                 </svg>",
                i,
                vw,
                vw,
                fill,
                y,
                vw,
                y,
                alpha,
                sw,
                y,
                beziers = beziers,
            );
        }

        out
    }
}

struct Index;

impl Index {
    fn bytes() -> &'static [u8] {
        let css = include_str!("../../resources/style.css");
        let svg_defs = include_str!("../../resources/icons.svg");
        let wave_svg = Wave::svg();
        let articles = Article::all();
        let body = tent::load_html_body!("resources/index.tent");
        let mut resp = sark::http::Response::ok();
        resp.set_body(body.finish());
        let bytes = resp.into_body_bytes();
        Box::leak(bytes.to_vec().into_boxed_slice())
    }
}

pub(super) struct State {
    index: &'static [u8],
    fjalla_one: &'static [u8],
    nanum: &'static [u8],
    counter: Cell<u64>,
}

impl State {
    pub(super) fn new() -> Self {
        Self {
            index: Index::bytes(),
            fjalla_one: include_bytes!("../../assets/fjalla-one.woff2"),
            nanum: include_bytes!("../../assets/nanum.woff2"),
            counter: Cell::new(0),
        }
    }
}

#[sark_gen::response(raw)]
#[header("content-type", "text/html")]
pub(super) struct IndexResponse {
    status: StatusCode,
    body: &'static [u8],
}

#[sark_gen::response(raw)]
#[header("content-type", "text/plain")]
pub(super) struct CountResponse {
    status: StatusCode,
    body: o3::buffer::Owned,
}

#[sark_gen::response(raw)]
#[header("content-type", "font/woff2")]
pub(super) struct AssetResponse {
    status: StatusCode,
    body: &'static [u8],
}

#[sark_gen::request]
pub(super) struct IndexRequest {}

#[sark_gen::request]
pub(super) struct CountRequest {}

#[sark_gen::request]
pub(super) struct AssetRequest {}

#[sark_gen::handler]
#[response_body(Static)]
#[static_response]
pub(super) fn index(_req: IndexRequest, state: &State) -> IndexResponse {
    IndexResponse {
        status: StatusCode::OK,
        body: state.index,
    }
}

#[sark_gen::handler]
pub(super) fn count(_req: CountRequest, state: &State) -> CountResponse {
    let n = state.counter.get().saturating_add(1);
    state.counter.set(n);
    let body = format!("Hello from Lair! Visit count: {}", n);
    CountResponse {
        status: StatusCode::OK,
        body: o3::buffer::Owned::from(body.as_bytes()),
    }
}

#[sark_gen::handler]
#[response_body(Static)]
#[static_response]
pub(super) fn asset_fjalla_one(_req: AssetRequest, state: &State) -> AssetResponse {
    AssetResponse {
        status: StatusCode::OK,
        body: state.fjalla_one,
    }
}

#[sark_gen::handler]
#[response_body(Static)]
#[static_response]
pub(super) fn asset_nanum(_req: AssetRequest, state: &State) -> AssetResponse {
    AssetResponse {
        status: StatusCode::OK,
        body: state.nanum,
    }
}

sark_gen::define_route! {
    pub(super) Lair: State => {
        GET "/" => index,
        GET "/count" => count,
        scope "/assets" => [
            GET "/fjalla-one.woff2" => asset_fjalla_one,
            GET "/nanum.woff2" => asset_nanum,
        ],
    }
}

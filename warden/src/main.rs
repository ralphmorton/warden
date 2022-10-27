use clap::Parser;
use http::header;
use http::{Request, Response};
use http::response;
use hyper::{Body, Client, Error, Server};
use hyper_tls::HttpsConnector;
use std::convert::From;
use std::iter::once;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use tower::make::Shared;
use tower::ServiceBuilder;
use tower_http::add_extension::AddExtensionLayer;
use tower_http::cors::CorsLayer;
use tower_http::sensitive_headers::SetSensitiveRequestHeadersLayer;
use tower_http::trace::TraceLayer;
use warden::agent;
use warden::agent::Ruleset;
use warden::args::Args;

#[tokio::main]
pub async fn main() {
    let args = Args::parse();

    let ruleset = agent::start(&args);

    let tracing_filter = format!("{},hyper=error,mio=error", args.log_level);
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_filter)
        .init();

    let service = ServiceBuilder::new()
        .layer(SetSensitiveRequestHeadersLayer::new(once(
            header::AUTHORIZATION,
        )))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .layer(AddExtensionLayer::new(ruleset))
        .service_fn(handler);

    let addr = SocketAddr::from((
        IpAddr::from_str(args.addr.as_str()).expect("Valid IP address specified"),
        args.port,
    ));

    let server = Server::bind(&addr).serve(Shared::new(service));

    if let Err(err) = server.await {
        eprintln!("server error: {}", err);
    }
}

async fn handler(req: Request<Body>) -> Result<Response<Body>, Error> {
    let ruleset = req
        .extensions()
        .get::<Ruleset>()
        .expect("ruleset available")
        .clone();

    match ruleset.iter().find(|r| r.applies(&req)) {
        None => Ok(err_404()),
        Some(rule) => {
            let https = HttpsConnector::new();
            let client = Client::builder().build::<_, hyper::Body>(https);

            match rule.transform_req(req) {
                None => Ok(err_404()),
                Some(r) => client.request(r).await,
            }
        },
    }
}

fn err_404() -> Response<Body> {
    response::Builder::new()
        .status(404)
        .body(Body::from("Not found"))
        .expect("can construct 404 response")
}

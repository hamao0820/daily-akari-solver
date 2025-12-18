use http::StatusCode;
use tracing_subscriber::{
    fmt::{format::Pretty, time::UtcTime},
    prelude::*,
};
use tracing_web::{performance_layer, MakeConsoleWriter};
use worker::{event, Context, Cors, Env, Method, Request, Response, Router};

#[event(start)]
fn start() {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        // 日本時間に設定
        .with_timer(UtcTime::rfc_3339())
        .with_writer(MakeConsoleWriter);
    let perf_layer = performance_layer().with_details_from_fields(Pretty::default());
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(perf_layer)
        .init();
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> worker::Result<Response> {
    console_error_panic_hook::set_once();

    let cors = Cors::default()
        // .with_origins([
        //     "http://localhost:8080",
        //     "https://library.pwll.dev",
        //     "https://library.kentakom1213.workers.dev",
        // ])
        .with_methods([Method::Get, Method::Post, Method::Options])
        .with_allowed_headers(["Content-Type", "Authorization"]);

    let resp = Router::new()
        .get("/health", |_, _| Response::ok("Daily Akari Solver!"))
        .post_async("/", |_req, _ctx| async move {
            Response::error("Bad Request", StatusCode::BAD_REQUEST.as_u16())
        })
        .run(req, env)
        .await?
        .with_cors(&cors)?;

    Ok(resp)
}

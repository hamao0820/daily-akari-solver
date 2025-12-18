mod io;

use akari::{solver, Solver};
use http::StatusCode;
use io::{SolveRequest, SolveResponse};
use tracing_subscriber::{
    fmt::{format::Pretty, time::UtcTime},
    prelude::*,
};
use tracing_web::{performance_layer, MakeConsoleWriter};
use worker::{event, Context, Cors, Env, Method, Request, Response, Router};

/// Try to solve the puzzle in the request with the CFS solver.
fn solve_request_with_cfs(req: &SolveRequest) -> (SolveResponse, StatusCode) {
    let field = match req.to_field() {
        Ok(field) => field,
        Err(msg) => return (SolveResponse::failed(req, msg), StatusCode::BAD_REQUEST),
    };

    let solver = solver::Fast::new();
    match solver.solve(&field) {
        Some(solution) => {
            let response_body = SolveResponse::solved(solution.akari_indices());
            (response_body, StatusCode::OK)
        }
        None => {
            let response_body = SolveResponse::failed(req, "No solution found");
            (response_body, StatusCode::OK)
        }
    }
}

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
        .post_async("/", |mut req: Request, _ctx| async move {
            let payload: SolveRequest = match req.json().await {
                Ok(body) => body,
                Err(err) => {
                    return Response::error(
                        format!("invalid JSON payload: {err}"),
                        StatusCode::BAD_REQUEST.as_u16(),
                    )
                }
            };

            let (response_body, status) = solve_request_with_cfs(&payload);

            let mut res = Response::from_json(&response_body)?;
            res = res.with_status(status.as_u16());
            Ok(res)
        })
        .run(req, env)
        .await?
        .with_cors(&cors)?;

    Ok(resp)
}

use std::{env, net::SocketAddr};

use axum::{
    http::{Method, Uri},
    middleware,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use ctx::Ctx;
use model::ModelController;
use serde_json::json;
use tower_cookies::CookieManagerLayer;
use tower_http::services::ServeDir;
use uuid::Uuid;
use web::routes_login;

mod ctx;
mod error;
mod log;
mod model;
mod web;

use crate::log::log_request;

pub use self::error::{AuthError, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let port = env::var("PORT")
        .ok()
        .and_then(|port| port.parse::<u16>().ok())
        .unwrap_or(3000);

    let mc = ModelController::new().await?;

    let routes_api = web::routes_tickets::routes(mc.clone())
        .route_layer(middleware::from_fn(web::mw_auth::mw_require_auth));

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .merge(routes_login::routes())
        .nest("/api", routes_api)
        .layer(middleware::map_response(main_response_mapper))
        .layer(middleware::from_fn_with_state(
            mc.clone(),
            web::mw_auth::mw_ctx_resolver,
        ))
        .layer(CookieManagerLayer::new())
        .fallback_service(fallback_routes());

    let listener = tokio::net::TcpListener::bind(&SocketAddr::from(([0, 0, 0, 0], port)))
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn main_response_mapper(
    ctx: Option<Ctx>,
    uri: Uri,
    req_method: Method,
    res: Response,
) -> Response {
    let uuid = Uuid::new_v4();

    let service_error = res.extensions().get::<AuthError>();
    let client_status_error = service_error.map(|se| se.client_status_and_error());

    let error_response = client_status_error
        .as_ref()
        .map(|(status_code, client_error)| {
            let client_error_body = json!({
                "error": {
                    "type": client_error.as_ref(),
                    "req_uuid": uuid.to_string(),
                }
            });

            println!("    ->> client_error_body: {client_error_body}");

            (*status_code, Json(client_error_body)).into_response()
        });

    let client_error = client_status_error.unzip().1;
    let _ = log_request(uuid, req_method, uri, ctx, service_error, client_error).await;

    println!();

    error_response.unwrap_or(res)
}

fn fallback_routes() -> Router {
    Router::new().nest_service("/", ServeDir::new("./"))
}

use axum::{routing::post, Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use tower_cookies::{Cookie, Cookies};

use crate::{web::AUTH_TOKEN, AuthError, Result};

#[derive(Debug, Deserialize)]
struct LoginPayload {
    name: String,
    pwd: String,
}

pub fn routes() -> Router {
    Router::new().route("/auth", post(api_login))
}

async fn api_login(cookies: Cookies, payload: Json<LoginPayload>) -> Result<Json<Value>> {
    println!("->> {:<12} - api login", "Handler");

    if payload.name != "pedro" || payload.pwd != "asd" {
        return Err(AuthError::LoginFail);
    }

    cookies.add(Cookie::new(AUTH_TOKEN, "test"));

    let body = Json(json!({
        "result": {
            "succes" : true
        }
    }));

    Ok(body)
}

use axum::extract::Path;
use axum::http::{Method, Uri};
use axum::response::Response;
use axum::routing::{get, get_service};
use axum::{
    extract::Query,
    response::{Html, IntoResponse},
    Router,
};
use axum::{middleware, Json};

use ctx::Ctx;
use serde::Deserialize;
use serde_json::json;
use tower_cookies::CookieManagerLayer;
use tower_http::services::ServeDir;
use uuid::Uuid;

use std::net::SocketAddr;

pub use self::error::{Error, Result};
use crate::db::{create_pg_pool, run_migrations};
use crate::log::log_request;
use crate::model::ModelController;

mod ctx;
mod db;
mod error;
mod log;
mod model;
mod web;

#[derive(Debug, Deserialize)]
struct HelloParams {
    name: Option<String>,
}

// Routes
fn hello_routes() -> Router {
    Router::new()
        .route("/hello", get(hello_handler))
        .route("/hello2/:name", get(hello2_handler))
}

fn static_routes() -> Router {
    Router::new().nest_service("/", get_service(ServeDir::new("./")))
}

// Layers
async fn main_response_mapper(
    ctx: Option<Ctx>,
    uri: Uri,
    req_method: Method,
    res: Response,
) -> Response {
    println!("->> {:<12} - main_response_mapper", "RES_MAPPER");
    let uuid = Uuid::new_v4();

    // -- Get the eventual response error.
    let service_error = res.extensions().get::<Error>();
    let client_status_error = service_error.map(|se| se.client_status_and_error());

    // -- If client error, build the new reponse.
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

            // Build the new response from the client_error_body
            (*status_code, Json(client_error_body)).into_response()
        });

    // Build and log the server log line.
    let client_error = client_status_error.unzip().1;
    log_request(uuid, req_method, uri, ctx, service_error, client_error).await;

    println!();
    error_response.unwrap_or(res)
}
async fn hello_handler(Query(params): Query<HelloParams>) -> impl IntoResponse {
    println!("->> {:<12} - hello_handler - {params:?}", "HANDLER");

    let name = params.name.as_deref().unwrap_or("World");
    Html(format!("Hello <strong>{name}</strong>"))
}

async fn hello2_handler(Path(name): Path<String>) -> impl IntoResponse {
    println!("->> {:<12} - hello2_handler - {name:?}", "HANDLER");

    Html(format!("Hello2 <strong>{name}</strong>"))
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the database connection
    let pool = create_pg_pool().await?;

    // Run the migrations
    run_migrations(&pool).await?;

    // Initialize ModelController
    let mc = ModelController::new().await?;

    let routes_apis = web::routes_tickets::routes(mc.clone())
        .route_layer(middleware::from_fn(web::mw_auth::mw_require_auth));

    let routes_all = Router::new()
        .merge(hello_routes())
        .merge(web::routes_login::routes())
        .nest("/api", routes_apis)
        .layer(middleware::map_response(main_response_mapper))
        .layer(middleware::from_fn_with_state(
            mc.clone(),
            web::mw_auth::mw_ctx_resolver,
        ))
        .layer(CookieManagerLayer::new())
        .fallback_service(static_routes());

    // Start Server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Listening on {addr} \n");
    axum::Server::bind(&addr)
        .serve(routes_all.into_make_service())
        .await
        .unwrap();

    Ok(())
}

use askama::Template;
use axum::body::Body;
use axum::body::Bytes;
use axum::extract::connect_info::Connected;
use axum::extract::{ConnectInfo, Extension};
use axum::http::Request;
use axum::middleware::{self, Next};
use axum::response::Html;
use axum::response::Response;
use axum::{routing::get, Router};
use http_body::Body as HttpBody;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
//mod api;
mod db;
use db::Db;
use std::error::Error as StdError;

#[derive(askama::Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub title: String,
    pub desc: String,
    pub add_info: String,
}

async fn index(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Extension(db): Extension<Arc<Db>>,
) -> Result<Html<String>, axum::http::StatusCode> {
    // Extract IP address from SocketAddr
    let ip_address = addr.ip().to_string();
    // Update or insert the IP address in the database
    if let Err(e) = db.update_or_insert_ip(&ip_address).await {
        eprintln!("Error updating database: {}", e);
        return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    }

    let template = IndexTemplate {
        title: "Hola👋, soy varo".to_string(),
        desc: "Web hecha con Rust y Htmx".to_string(),
        add_info:
            "Web provisional hecha con Rust usando plantillas de askama y htmx. En construcción 🚧 "
                .to_string(),
    };
    template
        .render()
        .map(Html)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)
}

async fn track_metrics(req: Request<Body>, next: Next) -> Response {
    let start = Instant::now();
    let uri = req.uri().to_string();
    let method = req.method().to_string();

    let response = next.run(req).await;

    let latency = start.elapsed().as_millis();
    let status = response.status().as_u16().to_string();

    println!(
        "method='{}' uri='{}' status='{}' latency='{}ms'",
        method, uri, status, latency
    );

    response
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Inicialización del tracing (como antes)
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or("debug,hyper=off".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Inicialización de la base de datos (como antes)
    let db = Arc::new(db::Db::new().await?);

    // Construye la aplicación
    let app = Router::new()
        .route("/", get(index))
        .layer(Extension(db.clone()))
        .nest_service("/assets", tower_http::services::ServeDir::new("assets"))
        .nest_service(
            "/favicon.ico",
            tower_http::services::ServeFile::new("assets/favicon.ico"),
        )
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(middleware::from_fn(track_metrics));

    // Ejecuta la aplicación
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
    Ok(())
}

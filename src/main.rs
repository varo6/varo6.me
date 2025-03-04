use askama::Template;
use axum::body::Body;
use axum::extract::{ConnectInfo, Extension};
use axum::http::{HeaderMap, Request};
use axum::middleware::{self, Next};
use axum::response::Html;
use axum::response::Response;
use axum::{routing::get, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
mod api; 
mod db;
use db::Db;

#[derive(askama::Template)]
#[template(path = "base.html")]
pub struct IndexTemplate {
    pub title: String,
    pub base_div: String,
    pub sum: i64,
}

async fn page_content(
    path: axum::extract::Path<String>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Extension(db): Extension<Arc<Db>>,
) -> Result<Html<String>, axum::http::StatusCode> {
    // Extract forwarded IP like in your current implementation
    let ip_address = headers
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
        .unwrap_or_else(|| addr.ip().to_string());

    if let Err(e) = db.update_or_insert_ip(&ip_address).await {
        eprintln!("Error updating database: {}", e);
        return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    }

    let nconns = match db.get_nconns().await {
        Ok(count) => count,
        Err(e) => {
            eprintln!("Error getting connection count: {}", e);
            return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Determine which content to load based on path
    let base_div = match path.as_str() {
        "" | "about" => {
            let about = api::AboutTemplate {};
            about.render().unwrap_or_default()
        },
        "projects" => {
            let projects = api::ProjectsTemplate {};
            projects.render().unwrap_or_default()
        },
        "workflow" => {
            let workflow = api::WorkflowTemplate {};
            workflow.render().unwrap_or_default()
        },
        _ => return Err(axum::http::StatusCode::NOT_FOUND),
    };

    let template = IndexTemplate {
        title: "varo6.me".to_string(),
        base_div,
        sum: nconns,
    };
    
    template
        .render()
        .map(Html)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)
}

async fn index(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Extension(db): Extension<Arc<Db>>,
) -> Result<Html<String>, axum::http::StatusCode> {
    // Use the about template as default for the root route
    let about = api::AboutTemplate {};
    let base_div = about.render().unwrap_or_default();
    
    // Extract forwarded IP like before
    let ip_address = headers
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
        .unwrap_or_else(|| addr.ip().to_string());

    if let Err(e) = db.update_or_insert_ip(&ip_address).await {
        eprintln!("Error updating database: {}", e);
        return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    }

    let nconns = match db.get_nconns().await {
        Ok(count) => count,
        Err(e) => {
            eprintln!("Error getting connection count: {}", e);
            return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let template = IndexTemplate {
        title: "varo6.me".to_string(),
        base_div,
        sum: nconns,
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
   
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or("debug,hyper=off".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

   
    let db = Arc::new(db::Db::new().await?);

   
    let app = Router::new()
        .route("/", get(index))
        .route("/{path}", get(page_content))
        // Routes desde api.rs
        .merge(api::routes())  
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
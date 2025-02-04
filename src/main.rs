use askama::Template;
use axum::response::Html;
use axum::{routing::get, Router};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

//mod api;
//mod db;

#[derive(askama::Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub text: String,
}

async fn index() -> Result<Html<String>, axum::http::StatusCode> {
    let template = IndexTemplate {
        text: "Hello, world!".to_string(),
    };
    template
        .render()
        .map(Html)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)
}

#[tokio::main]
async fn main() {
    // Start registry -> https://docs.rs/tracing-subscriber/latest/tracing_subscriber/registry/struct.Registry.html
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or("debug,hyper=off".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // use db module from db.rs
    // let pool = db::connect_to_database().await;

    // migrate db
    // sqlx::migrate!().run(&pool).await.unwrap();

    // build our application:
    // https://docs.rs/axum/latest/axum/struct.Router.html#method.nest for static serve
    //
    let app = Router::new()
        .route("/", get(index))
        .nest_service("/assets", tower_http::services::ServeDir::new("assets"))
        .nest_service(
            "/favicon.ico",
            tower_http::services::ServeFile::new("assets/favicon.ico"),
        )
        //  .nest("/api", api::api_routes())
        .layer(tower_http::trace::TraceLayer::new_for_http());
    // .with_state(pool);

    // run it
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

use askama::Template;
use axum::response::Html;
use axum::{routing::get, Router};

#[derive(Template)]
#[template(path = "about.html")]
pub struct AboutTemplate {}

pub async fn about() -> Result<Html<String>, axum::http::StatusCode> {
    let template = AboutTemplate {};
    template
        .render()
        .map(Html)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)
}

#[derive(Template)]
#[template(path = "projects.html")]
pub struct ProjectsTemplate {}

pub async fn projects() -> Result<Html<String>, axum::http::StatusCode> {
    let template = ProjectsTemplate {};
    template
        .render()
        .map(Html)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)
}

#[derive(Template)]
#[template(path = "workflow.html")]
pub struct WorkflowTemplate {}

pub async fn workflow() -> Result<Html<String>, axum::http::StatusCode> {
    let template = WorkflowTemplate {};
    template
        .render()
        .map(Html)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)
}

// Función para configurar las rutas API
pub fn routes() -> Router {
    Router::new()
        .route("/api/projects", get(projects))
        .route("/api/about", get(about))
        .route("/api/workflow", get(workflow))
}
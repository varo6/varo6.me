use askama::Template;
use axum::response::Html;
use axum::{routing::get, Router};

pub async fn about() -> Html<&'static str> {
    Html("<h1> About </h1>
        <p>Aquí se irán subiendo los proyectos, de momento está en construcción. 🚧</p>")
}



// Versión con template (comentada)
// pub async fn hello_htmx_template() -> Result<Html<String>, axum::http::StatusCode> {
//     let template = HelloFragment {};
//     template
//         .render()
//         .map(Html)
//         .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)
// }

pub async fn projects() -> Html<&'static str> {
    Html("<h1> Proyectos </h1>
        <p>Aquí se irán subiendo los proyectos, de momento está en construcción. 🚧</p>")
}

// Función para configurar las rutas API
pub fn routes() -> Router {
    Router::new()
        .route("/api/projects", get(projects))
        .route("/api/about", get(about))
}
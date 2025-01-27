
use axum::{
    response::Html,
    routing::get,
    Router,
};
use tokio::fs;

#[tokio::main]
async fn main() {
    // Cargar el contenido del archivo index.html
    let html_content = fs::read_to_string("index.html").await.unwrap();

    // Construir la aplicación con una ruta que devuelve el contenido HTML
    let app = Router::new().route("/", get(|| async { Html(html_content) }));

    // Ejecutar la aplicación en el puerto 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

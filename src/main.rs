use std::{fs, io, str::from_utf8};

use axum::{
  response::Html,
  routing::{get, get_service},
  Json, Router,
};
use http::Method;
use local_ip_address::local_ip;
use rust_embed::RustEmbed;
use serde_json::{json, Value};
use tower_http::{
  cors::{Any, CorsLayer},
  services::ServeDir,
};

#[derive(RustEmbed)]
#[folder = "src/resources"]
#[prefix = "resources/"]
struct Asset;

fn get_list() -> Json<Value> {
  let entries = fs::read_dir("./")
    .unwrap()
    .map(|res| res.map(|e| e.path()))
    .collect::<Result<Vec<_>, io::Error>>()
    .unwrap();
  json!({
  "code": 200,
  "success": true,
  "payload": {
      "list": entries,
  }})
  .into()
}

fn index_html() -> Html<String> {
  let my_local_ip = local_ip().unwrap();
  let html = Asset::get("resources/index.html").unwrap();
  let processed_html = from_utf8(html.data.as_ref())
    .unwrap()
    .replace("localhost", my_local_ip.to_string().as_str());
  Html(processed_html)
}

fn routes_static() -> Router {
  Router::new().nest_service("/", get_service(ServeDir::new("./")))
}

fn routes() -> Router {
  Router::new()
    .route("/", get(index_html()))
    .route("/list", get(get_list()))
    .fallback_service(routes_static())
}

#[tokio::main]
async fn main() {
  let cors = CorsLayer::new()
    // allow `GET` and `POST` when accessing the resource
    .allow_methods([Method::GET, Method::POST])
    // allow requests from any origin
    .allow_origin(Any);

  let my_local_ip = local_ip().unwrap();
  let routes = Router::new().merge(routes().layer(cors));
  let listener = tokio::net::TcpListener::bind("0.0.0.0:1998").await.unwrap();
  println!("Server running at http://localhost:1998");
  println!("Server running at http://{}:1998", my_local_ip);
  axum::serve(listener, routes).await.unwrap();
}

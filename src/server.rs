use std::{collections::HashMap, fs, io, str::from_utf8};

use axum::{
  extract::Query,
  response::Html,
  routing::{get, get_service},
  Json, Router,
};
use http::Method;
use local_ip_address::local_ip;
use rust_embed::RustEmbed;
use serde::Serialize;
use serde_json::{json, Value};
use tower_http::{
  cors::{Any, CorsLayer},
  services::ServeDir,
};

#[derive(RustEmbed)]
#[folder = "src/assets"]
#[prefix = "assets/"]
struct Asset;

#[derive(Debug, Serialize)]
struct Metadata {
  is_dir: bool,
  is_file: bool,
  filename: String,
}

async fn get_list(Query(params): Query<HashMap<String, String>>) -> Json<Value> {
  let path = params.get("path").unwrap();
  match fs::read_dir(path) {
    Ok(read_dir) => {
      let mut data: Vec<Metadata> = vec![];
      for entry in read_dir {
        let dir_entry = entry.unwrap();
        let is_dir = dir_entry.metadata().unwrap().is_dir();
        let is_file = dir_entry.metadata().unwrap().is_file();
        let filename = if is_file {
          dir_entry.file_name().to_str().unwrap().to_string()
        } else {
          dir_entry.path().to_str().unwrap().to_string()
        };
        println!("{}", filename);
        data.push(Metadata {
          is_dir,
          is_file,
          filename,
        });
      }
      json!({
      "code": 200,
      "success": true,
      "payload": {
          "list": data,
      }})
      .into()
    }
    Err(error) => json!({
    "code": 404,
    "success": false,
    "payload": {
        "msg": error.to_string(),
    }})
    .into(),
  }
}

async fn index_html() -> Html<String> {
  let my_local_ip = local_ip().unwrap();
  let html = Asset::get("assets/index.html").unwrap();
  let processed_html = from_utf8(html.data.as_ref())
    .unwrap()
    .replace("localhost", my_local_ip.to_string().as_str());
  Html(processed_html.to_string())
}

async fn get_vue() -> String {
  let vue = Asset::get("assets/vue.min.js").unwrap();
  let processed_vue = from_utf8(vue.data.as_ref()).unwrap();
  processed_vue.to_string()
}

fn routes_static() -> Router {
  Router::new().nest_service("/", get_service(ServeDir::new("./")))
}

fn routes() -> Router {
  Router::new()
    .route("/", get(index_html))
    .route("/vue.min.js", get(get_vue))
    .route("/list", get(get_list))
    .fallback_service(routes_static())
}

pub async fn start() {
  const PORT: u16 = 1998;

  let cors = CorsLayer::new()
    // allow `GET` and `POST` when accessing the resource
    .allow_methods([Method::GET, Method::POST])
    // allow requests from any origin
    .allow_origin(Any);

  let my_local_ip = local_ip().unwrap();
  let routes = Router::new().merge(routes().layer(cors));
  match tokio::net::TcpListener::bind(("0.0.0.0", PORT)).await {
    Ok(listener) => {
      println!("Server running at http://127.0.0.1:{}", PORT);
      println!("Server running at http://{}:{}", my_local_ip, PORT);
      axum::serve(listener, routes).await.unwrap();
    }
    Err(error) => match error.kind() {
      io::ErrorKind::AddrInUse => println!("端口已被占用"),
      _ => panic!("其他错误"),
    },
  }
}

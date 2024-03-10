use server::start;

mod server;

#[tokio::main]
async fn main() {
  start().await;
}

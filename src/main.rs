use crate::app::App;

mod app;
pub mod auth;
pub mod error;
pub mod models;
pub mod repository;
mod routes;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    App::start().await
}

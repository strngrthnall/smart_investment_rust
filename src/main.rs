#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    smart_investment_rust::app::App::start().await
}

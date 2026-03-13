use dotenv::dotenv;
use loco_rs::cli;
use migration::Migrator;
use the_handler::app::App;

#[tokio::main]
async fn main() -> loco_rs::Result<()> {
    dotenv().ok();
    cli::main::<App, Migrator>().await
}

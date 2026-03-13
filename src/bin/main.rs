use loco_rs::cli;
use migration::Migrator;
use the_handler::app::App;

#[tokio::main]
async fn main() -> loco_rs::Result<()> {
    cli::main::<App, Migrator>().await
}

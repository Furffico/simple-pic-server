use anyhow::Result;

mod getconfig;
mod server;

#[tokio::main]
async fn main() -> Result<()> {
    let args = getconfig::Args::get()?;
    println!("{:#?}", args);

    server::runserver(&args).await
}

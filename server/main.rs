mod finfo;
mod server;
mod static_var;
mod template;
use anyhow::Result;
use server::run_server;

#[tokio::main]
pub async fn main() -> Result<()> {
    run_server().await
}

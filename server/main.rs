mod finfo;
mod server;
mod static_const;
mod template;
use anyhow::Result;
use server::run_server;
use std::env::set_current_dir;
use static_const::CONFIG;

#[tokio::main]
pub async fn main() -> Result<()> {
    // change current directory
    set_current_dir(CONFIG.get_string("basepath").unwrap_or("./".to_string()))
        .expect("failed switching current working directory");
    run_server().await
}

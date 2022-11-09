use anyhow::{anyhow, Result};
use clap::{Parser, ValueHint};
use std::net::SocketAddr;
use std::path::Path;

/// A Rust implimentation of dd@home client
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)] // Read from `Cargo.toml`
pub struct Args {
    /// the base directory
    #[arg(default_value = ".", value_hint(ValueHint::DirPath))]
    pub basedir: String,

    /// the socket address to bind to
    #[arg(
        short,
        long,
        default_value = "127.0.0.1:8400",
        value_hint(ValueHint::Hostname)
    )]
    pub listen: SocketAddr,
}

impl Args {
    pub fn get() -> Result<Args> {
        let args = Args::parse();
        match args.validate() {
            Ok(_) => Ok(args),
            Err(e) => Err(e),
        }
    }

    pub fn validate(&self) -> Result<()> {
        // check basedir
        let basedir = Path::new(&self.basedir);
        if !basedir.is_dir() || !basedir.exists() {
            return Err(anyhow!("Not a valid directory: {}", &self.basedir));
        }
        Ok(())
    }
}

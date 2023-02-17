use std::error::Error;

use crate::oracles::Oracles;

mod config;
mod mempool;
mod oracles;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let oracles = Oracles::new().find_all().await?;
    println!("{oracles:?}");
    Ok(())
}

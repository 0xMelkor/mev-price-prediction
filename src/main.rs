use std::{error::Error};

use ethers::types::U256;
use mempool::subscribe_mempool_events;
use tokio::sync::mpsc;

use crate::oracles::Oracles;

mod config;
mod mempool;
mod oracles;

/// _________                         __         .__       _____  _______________   ____
/// \_   ___ \_______ ___.__. _______/  |______  |  |     /     \ \_   _____/\   \ /   /
/// /    \  \/\_  __ <   |  |/  ___/\   __\__  \ |  |    /  \ /  \ |    __)_  \   Y   / 
/// \     \____|  | \/\___  |\___ \  |  |  / __ \|  |__ /    Y    \|        \  \     /  
///  \______  /|__|   / ____/____  > |__| (____  /____/ \____|__  /_______  /   \___/   
///         \/        \/         \/            \/               \/        \/            
/// 
/// WARN: This uses `eth_newPendingTransactionFilter` under the hood. 
/// Which is generally not exposed by RPC service providers (e.g. Infura).
/// Come on bro! It's time to spawn your own node!
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Let's retrieve the list of ChainLink oracles from AAVE V2
    let oracles = Oracles::new().find_all().await?;

    let (tx, mut rx) = mpsc::channel::<U256>(1);

    // For the sake of illustration we track price changes 
    // for just one oracle
    let oracle = oracles.get(0).unwrap();
    subscribe_mempool_events(tx,  oracle.address);
    println!("Monitoring {} price updates in the mempool", oracle.asset);

    while let Some(new_price) = rx.recv().await {
        println!("{} price is about to change! {new_price:?}", oracle.asset)

}
    Ok(())
}

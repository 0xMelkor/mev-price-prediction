use std::error::Error;

use ethers::types::{Transaction, U256};
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

#[derive(Debug)]
pub struct PricePrediction {
    pub new_price: U256,
    /// The transaction directed to an oracle
    /// that will update the asset's price
    pub transaction: Transaction,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Let's retrieve the list of ChainLink oracles from AAVE V2
    let oracles = Oracles::new().find_all().await?;
    println!("{oracles:?}");

    let (tx, mut rx) = mpsc::channel::<PricePrediction>(1);

    // For the sake of illustration we track price changes
    // for only one oracle
    let oracle = oracles.get(0).unwrap();
    subscribe_mempool_events(tx, oracle.address);
    println!("Monitoring {} price updates in the mempool", oracle.asset);

    while let Some(opportunity) = rx.recv().await {
        println!(
            "The price of {} is going to be {} ETH",
            oracle.asset, opportunity.new_price
        );

        // Knowing txn poses your bot in a privileged position.
        // You know the price is going to change in a bunch of seconds.
        // Use this future knowledge to make your bot more competitive.
        let _txn = opportunity.transaction;
    }
    Ok(())
}

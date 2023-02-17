use std::error::Error;

use ethers::{
    prelude::MiddlewareBuilder,
    providers::{Http, Middleware, Provider},
    signers::LocalWallet,
    types::{Address, Eip1559TransactionRequest, Transaction, U256, U64},
};
use mempool::subscribe_mempool_events;
use tokio::sync::mpsc;

use crate::{config::Config, oracles::Oracles};

mod config;
mod mempool;
mod oracles;

#[derive(Debug)]
pub struct BackrunOpportunity {
    pub new_price: U256,
    pub transaction: Transaction,
}

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
    println!("{oracles:?}");

    let (tx, mut rx) = mpsc::channel::<BackrunOpportunity>(1);

    // For the sake of illustration we track price changes
    // for just one oracle
    let oracle = oracles.get(0).unwrap();
    subscribe_mempool_events(tx, oracle.address);
    println!("Monitoring {} price updates in the mempool", oracle.asset);

    while let Some(opportunity) = rx.recv().await {
        println!(
            "The price of {} is going to be {} ETH",
            oracle.asset, opportunity.new_price
        );

        let txn = opportunity.transaction;
        match txn.transaction_type {
            // EIP-1559 (0x02)
            Some(x) if x == U64::from(2) => {
                backrun(txn).await;
            }
            _ => (),
        }
    }
    Ok(())
}

/// Backruns the EIP1559 transaction of the oracle.
/// NOTE: This is to get the rationale, so what's provided here is just a fake transaction.
async fn backrun(txn: Transaction) {
    let config = Config::new();
    let http_url = &config.http_url;

    let key = "YOUR-SIGNING-KEY-HERE";
    let signer = key.parse::<LocalWallet>().unwrap();

    let provider = Provider::<Http>::try_from(http_url)
        .unwrap()
        .with_signer(signer);

    // Adjust the priority to fall exactly behind txn
    let max_fee = txn.max_fee_per_gas.unwrap() - U256::one();
    let tip = txn.max_priority_fee_per_gas.unwrap() - U256::one();

    let back_txn = Eip1559TransactionRequest::new()
        .from(Address::default())
        .to(Address::default())
        .value(U256::zero())
        .max_fee_per_gas(max_fee)
        .max_priority_fee_per_gas(tip);

    provider.send_transaction(back_txn, None).await.unwrap();
}

use ethers::{
    core::abi::AbiDecode,
    prelude::abigen,
    providers::{Middleware, Provider, StreamExt, Ws},
    types::{Address, Bytes, Transaction, U256},
};
use tokio::sync::mpsc::Sender;

use crate::{config::Config, PricePrediction};

abigen!(
    OffchainAggregator,
    r#"[
        function transmit(bytes calldata report, bytes32[] calldata rs, bytes32[] calldata ss, bytes32 rawVs) external
    ]"#
);

/// Intercepts pending transactions in the mempool that are submitted by the ChainLink network
/// to update an asset's price. These transactions are sent to an `OffChainAggregator` smart contract.
///
/// * `tx` is a feedback channel used to notifify the caller about pending transactions.
/// * `oracle` is the actual address of the `OffChainAggregator`, that we use as a filter.
///
/// NOTE: That `oracle` is relative to a single ERC20 token.
pub fn subscribe_mempool_events(tx: Sender<PricePrediction>, oracle: Address) {
    tokio::spawn(async move {
        let config = Config::new();
        let ws_url = config.ws_url;

        loop {
            let provider = Provider::<Ws>::connect(&ws_url)
                .await
                .expect("Unable to connect web socket");

            let stream = provider
                .watch_pending_transactions()
                .await
                .expect("Unable to connect txn stream");

            let mut tx_stream = stream.transactions_unordered(usize::MAX);

            // Listen to pending transactions
            while let Some(txn) = tx_stream.next().await {
                match txn {
                    Err(e) => {
                        let err = format!("{e:?}");
                        if err.contains("Disconnect") {
                            println!("Reconnecting websocket");
                            break;
                        }
                    }
                    Ok(txn) => {
                        // Ensure transaction is "to" our price oracle
                        if !is_to_oracle(&txn, oracle) {
                            continue;
                        }

                        // Ensure transaction is not yet mined
                        if !is_pending(&txn, &provider).await {
                            continue;
                        }

                        // Try to decode the transaction and extract the new price
                        if let Ok(decoded) = TransmitCall::decode(txn.input.clone()) {
                            let report: Bytes = decoded.report;
                            let mut observations: Vec<U256> = vec![];
                            const WORD_SIZE: usize = 32;
                            for word in report.to_vec().chunks(WORD_SIZE).into_iter().skip(4) {
                                observations.push(U256::from(word));
                            }

                            // Chainlink network provides a list of sorted price observations in the txn.
                            // The smart contract takes the median of all values. The median represents the
                            // new price that will be written on-chain
                            // ==========================================================================
                            // Same logic as OffchainAggregator.sol:L614
                            // https://github.com/smartcontractkit/libocr/blob/master/contract/OffchainAggregator.sol
                            if let Some(median) = observations.get(observations.len() / 2) {
                                tx.send(PricePrediction {
                                    new_price: *median,
                                    transaction: txn,
                                })
                                .await
                                .unwrap();
                            }
                        }
                    }
                }
            }
        }
    });
}

fn is_to_oracle(txn: &Transaction, oracle: Address) -> bool {
    match txn.to {
        None => false,
        Some(to) => to == oracle,
    }
}

async fn is_pending(txn: &Transaction, provider: &Provider<Ws>) -> bool {
    match provider.get_transaction_receipt(txn.hash).await {
        Ok(Some(r)) => {
            println!("Found transaction receipt {:?}, skipping...", r);
            false
        }
        Err(e) => {
            println!("Error during retrieval of txn receipt {e:?}");
            false
        }
        Ok(None) => true,
    }
}

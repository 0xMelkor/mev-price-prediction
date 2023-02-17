use std::{error::Error, sync::Arc};

use ethers::{
    prelude::abigen,
    providers::{Http, Provider},
    types::Address,
};

use crate::config::Config;

abigen!(AggregatorInterface, "./abi/AggregatorInterface.json",);
abigen!(AaveOracle, "./abi/AaveOracle.json",);
abigen!(
    AaveProtocolDataProvider,
    "./abi/AaveProtocolDataProvider.json",
);

#[derive(Debug)]
pub struct Oracle {
    pub asset: String,
    pub address: Address,
}

pub struct Oracles {
    config: Config,
    provider: Arc<Provider<Http>>,
}

impl Oracles {
    pub fn new() -> Self {
        let config = Config::new();
        let provider = Provider::<Http>::try_from(&config.http_url).expect("Invalid config");
        let provider = Arc::new(provider);
        Self { config, provider }
    }

    /// Returns a list of ChainLink oracles for all reserves on AAVE V2.
    ///
    /// Some reserves might not have a ChainLink oracle associated.
    /// This is the case of WETH, that is used as the base currency on AAVE V2.
    /// This means that the value of all other reserves is expressed in WETH.
    /// In other words there's no reason to track the value of WETH, since will always be 1 WETH.
    pub async fn find_all(&self) -> Result<Vec<Oracle>, Box<dyn Error>> {
        let mut oracles = vec![];
        let token_list: Vec<TokenData> = self.reserve_list().await?;
        for token in token_list.into_iter() {
            match self.chainlink_aggregator(token.token_address).await {
                Ok(address) => oracles.push(Oracle {
                    asset: token.symbol,
                    address,
                }),
                Err(e) => println!("WARN: Cannot retrieve oracle for {}: {e:?}", token.symbol),
            }
        }
        Ok(oracles)
    }

    /// Returns the full list of reserves on AAVE
    async fn reserve_list(&self) -> Result<Vec<TokenData>, Box<dyn Error>> {
        let client = Arc::clone(&self.provider);
        let address: Address = self.config.aave_protocol_data_provider;
        let contract = AaveProtocolDataProvider::new(address, client);
        match contract.get_all_reserves_tokens().call().await {
            Ok(tokens) => Ok(tokens),
            Err(e) => Err(format!("Unable to retrieve token list {e:?}"))?,
        }
    }

    /// Each reserve on AAVE is associated to a ChainLink aggregator proxy.
    /// Returns the actual aggregator behind the proxy.
    async fn chainlink_aggregator(&self, reserve: Address) -> Result<Address, Box<dyn Error>> {
        let client = Arc::clone(&self.provider);
        let proxy: Address = self.chainlink_proxy(reserve).await?;
        let aggregator_proxy = AggregatorInterface::new(proxy, client);
        match aggregator_proxy.aggregator().call().await {
            Ok(aggregator) => Ok(aggregator),
            Err(e) => Err(format!("Unable to retrieve ChainLink aggregator {e:?}"))?,
        }
    }

    async fn chainlink_proxy(&self, reserve: Address) -> Result<Address, Box<dyn Error>> {
        let client = Arc::clone(&self.provider);
        let address: Address = self.config.aave_oracle;
        let contract = AaveOracle::new(address, client);
        match contract.get_source_of_asset(reserve).call().await {
            Ok(oracle) => Ok(oracle),
            Err(e) => Err(format!(
                "Unable to retrieve price oracle proxy for reserve {reserve:?}: {e:?}"
            ))?,
        }
    }
}

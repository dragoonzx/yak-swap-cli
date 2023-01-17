use std::{borrow::BorrowMut, sync::Arc};

use crate::{
    abis::{FormattedOfferWithGas, YakAdapter, YakRouter},
    network::Network,
    token::Token,
};
use adapters::Adapter;
use ethers::{
    abi::Address,
    contract::Contract,
    prelude::abigen,
    providers::{Http, Provider},
    types::{H160, U256},
};
use futures::future;
use serde::Deserialize;
use tokio::task::JoinError;

pub mod adapters;

pub struct Query {}

pub enum ExternalQuoteError {
    NetworkNotSupported,
    ReqwestError(reqwest::Error),
}

impl From<reqwest::Error> for ExternalQuoteError {
    fn from(err: reqwest::Error) -> ExternalQuoteError {
        ExternalQuoteError::ReqwestError(err)
    }
}

impl Query {
    #[tokio::main]
    pub async fn get_adapters() -> Vec<Adapter> {
        let current_network = Network::get_current_network();
        let provider = Arc::new(
            Provider::<Http>::try_from(current_network.rpc_url)
                .expect("could not instantiate HTTP Provider"),
        );

        // @todo memo get_adapters to refresh only on yak_router_address change
        // we need it to match address <> name for query
        if let Some(yak_router_address) = current_network.yak_router {
            let router_contract = Arc::new(YakRouter::new(
                yak_router_address.parse::<H160>().unwrap(),
                provider.clone(),
            ));
            let count: &Result<U256, _> = &router_contract.adapters_count().call().await;

            match count {
                Ok(count) => {
                    // create parallel requests for .. in count to get Vec of Adapters
                    let mut tasks = vec![];

                    for i in 0..count.as_u32() {
                        let provider_cloned = provider.clone();
                        let router_contract = router_contract.clone();

                        let task = tokio::spawn(async move {
                            let adapter_address: Option<H160> =
                                router_contract.adapters(U256::from(i)).call().await.ok();

                            if let Some(adapter_address) = adapter_address {
                                let adapter_contract =
                                    YakAdapter::new(adapter_address.to_owned(), provider_cloned);
                                let adapter_name = adapter_contract.name().call().await.unwrap();

                                Adapter {
                                    address: adapter_address.to_owned(),
                                    name: adapter_name,
                                }
                            } else {
                                panic!("Error while getting adapter address");
                            }
                        });

                        tasks.push(task);
                    }

                    let completed_tasks = future::join_all(tasks)
                        .await
                        .into_iter()
                        // @todo may panic so consider to refactor this
                        .map(|adapter| adapter.unwrap())
                        .collect();

                    let adapters = completed_tasks;

                    return adapters;
                }
                Err(_err) => {
                    panic!("Error when call router.adaptersCount");
                }
            }
        } else {
            panic!("No Yak Router address");
        }
    }

    #[tokio::main]
    pub async fn query_adapter(
        adapter: H160,
        amount: U256,
        token_in: H160,
        token_out: H160,
    ) -> Result<U256, ethers::contract::ContractError<ethers::providers::Provider<Http>>> {
        let current_network = Network::get_current_network();
        let provider = Arc::new(
            Provider::<Http>::try_from(current_network.rpc_url)
                .expect("could not instantiate HTTP Provider"),
        );

        let mut token_in = token_in;

        if Token::is_native(token_in) {
            token_in = Token::get_native_wrapped(current_network.chain_id);
        }

        let mut token_out = token_out;

        if Token::is_native(token_out) {
            token_out = Token::get_native_wrapped(current_network.chain_id);
        }

        let adapter_contract = YakAdapter::new(adapter.to_owned(), provider);
        let query: Result<
            U256,
            ethers::contract::ContractError<ethers::providers::Provider<Http>>,
        > = adapter_contract
            .query(amount, token_in, token_out)
            .call()
            .await;

        query
    }

    #[tokio::main]
    pub async fn find_best_path_with_gas(
        amount: U256,
        token_in: H160,
        token_out: H160,
        max_steps: i32,
    ) -> Result<
        FormattedOfferWithGas,
        ethers::contract::ContractError<ethers::providers::Provider<Http>>,
    > {
        let gas_price = 225;

        let current_network = Network::get_current_network();
        let provider = Arc::new(
            Provider::<Http>::try_from(current_network.rpc_url)
                .expect("could not instantiate HTTP Provider"),
        );

        let mut token_in = token_in;

        if Token::is_native(token_in) {
            token_in = Token::get_native_wrapped(current_network.chain_id);
        }

        let mut token_out = token_out;

        if Token::is_native(token_out) {
            token_out = Token::get_native_wrapped(current_network.chain_id);
        }

        if let Some(yak_router_address) = current_network.yak_router {
            let router_contract = Arc::new(YakRouter::new(
                yak_router_address.parse::<H160>().unwrap(),
                provider.clone(),
            ));

            let best_path = router_contract
                .find_best_path_with_gas(
                    amount,
                    token_in,
                    token_out,
                    U256::from(max_steps),
                    U256::from(gas_price),
                )
                .call()
                .await;

            best_path
        } else {
            panic!("No Yak Router address");
        }
    }

    // {
    //     "fromToken": {
    //       "symbol": "YAK",
    //       "name": "Yak Token",
    //       "decimals": 18,
    //       "address": "0x59414b3089ce2af0010e7523dea7e2b35d776ec7",
    //       "logoURI": "https://tokens.1inch.io/0x59414b3089ce2af0010e7523dea7e2b35d776ec7.png",
    //       "eip2612": true,
    //       "tags": [
    //         "tokens"
    //       ]
    //     },
    //     "toToken": {
    //       "symbol": "WAVAX",
    //       "name": "Wrapped AVAX",
    //       "decimals": 18,
    //       "address": "0xb31f66aa3c1e785363f0875a1b74e27b85fd66c7",
    //       "logoURI": "https://tokens.1inch.io/0xb31f66aa3c1e785363f0875a1b74e27b85fd66c7.png",
    //       "wrappedNative": "true",
    //       "tags": [
    //         "tokens",
    //         "PEG:AVAX"
    //       ]
    //     },
    //     "toTokenAmount": "156365459656653720063",
    //     "fromTokenAmount": "10000000000000000000",
    //     "protocols": [
    //       [
    //         [
    //           {
    //             "name": "PANGOLIN",
    //             "part": 100,
    //             "fromTokenAddress": "0x59414b3089ce2af0010e7523dea7e2b35d776ec7",
    //             "toTokenAddress": "0xb97ef9ef8734c71904d8002f8b6bc66dd9c48a6e"
    //           }
    //         ],
    //         [
    //           {
    //             "name": "AVALANCHE_KYBERSWAP_ELASTIC",
    //             "part": 100,
    //             "fromTokenAddress": "0xb97ef9ef8734c71904d8002f8b6bc66dd9c48a6e",
    //             "toTokenAddress": "0xb31f66aa3c1e785363f0875a1b74e27b85fd66c7"
    //           }
    //         ]
    //       ],
    //       [
    //         [
    //           {
    //             "name": "LYDIA",
    //             "part": 2,
    //             "fromTokenAddress": "0x59414b3089ce2af0010e7523dea7e2b35d776ec7",
    //             "toTokenAddress": "0xb31f66aa3c1e785363f0875a1b74e27b85fd66c7"
    //           },
    //           {
    //             "name": "TRADERJOE",
    //             "part": 8,
    //             "fromTokenAddress": "0x59414b3089ce2af0010e7523dea7e2b35d776ec7",
    //             "toTokenAddress": "0xb31f66aa3c1e785363f0875a1b74e27b85fd66c7"
    //           },
    //           {
    //             "name": "PANGOLIN",
    //             "part": 90,
    //             "fromTokenAddress": "0x59414b3089ce2af0010e7523dea7e2b35d776ec7",
    //             "toTokenAddress": "0xb31f66aa3c1e785363f0875a1b74e27b85fd66c7"
    //           }
    //         ]
    //       ]
    //     ],
    //     "estimatedGas": 1057918
    //   }

    #[tokio::main]
    pub async fn get_1inch_price(
        amount: U256,
        token_in: H160,
        token_out: H160,
    ) -> Result<ExternalQuote, ExternalQuoteError> {
        let current_network = Network::get_current_network();
        let supported_networks = vec![
            1,          // Ethereum
            56,         // BSC
            137,        // Polygon
            10,         // Optimism
            42161,      // Arbitrum
            100,        // Gnosis Chain
            250,        // Fantom
            8217,       // Klaytn
            1313161554, // Aurora
            43114,      // Avalanche
        ];

        if !supported_networks.contains(&current_network.chain_id) {
            return Err(ExternalQuoteError::NetworkNotSupported);
        }

        let mut token_in = token_in;

        if Token::is_native(token_in) {
            token_in = Token::get_native_wrapped(current_network.chain_id);
        }

        let mut token_out = token_out;

        if Token::is_native(token_out) {
            token_out = Token::get_native_wrapped(current_network.chain_id);
        }

        // check that network is supported (Avalanche, Optimism, Arbitrum)
        let request_url = format!(
            "https://api.1inch.io/v5.0/{chain_id}/quote?fromTokenAddress={:?}&toTokenAddress={:?}&amount={}",
            token_in,
            token_out,
            amount,
            chain_id = current_network.chain_id
        );

        let response = reqwest::get(&request_url).await?;

        let external_quote: ExternalQuote = response.json().await?;

        Ok(external_quote)
    }
}

#[derive(Debug, Deserialize)]
pub struct ExternalQuote {
    pub toTokenAmount: String,
    pub estimatedGas: u32,
}

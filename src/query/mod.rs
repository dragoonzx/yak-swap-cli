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
    prelude::{abigen, k256::sha2::digest::block_buffer::Error},
    providers::{Http, Provider},
    types::{H160, U256},
};
use futures::future;
use tokio::task::JoinError;

mod adapters;

pub struct Query {}

impl Query {
    #[tokio::main]
    pub async fn get_adapters() -> Vec<Adapter> {
        // get provider from network rpc
        // init yak_router contract
        // yak_router.adaptersCount() -> get adapter address from yak + get adapter name from adapter
        let current_network = Network::get_current_network();
        let provider = Arc::new(
            Provider::<Http>::try_from(current_network.rpc_url)
                .expect("could not instantiate HTTP Provider"),
        );

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
    ) -> Result<
        FormattedOfferWithGas,
        ethers::contract::ContractError<ethers::providers::Provider<Http>>,
    > {
        let max_steps = 3;
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
}

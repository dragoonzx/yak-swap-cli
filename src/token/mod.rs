use core::fmt;
use std::{collections::HashMap, sync::Arc};

use console::style;
use ethers::{
    abi::{AbiEncode, Address},
    prelude::{k256::ecdsa::SigningKey, SignerMiddleware},
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer, Wallet},
    types::{BlockId, TransactionReceipt, H160, U256},
};
use serde::{Deserialize, Serialize};

use crate::{abis::ERC20, network::Network};

#[path = "../token/storage.rs"]
pub mod token_storage;

#[derive(Deserialize)]
struct CoingeckoVersion {
    major: u16,
    minor: u16,
    patch: u16,
}

#[derive(Deserialize)]
struct CoingeckoResponse {
    keywords: Vec<String>,
    logoURI: String,
    name: String,
    timestamp: String,
    tokens: Vec<Token>,
    version: CoingeckoVersion,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Token {
    pub address: String,
    pub chainId: Option<u32>,
    pub decimals: u32,
    pub logoURI: String,
    pub name: String,
    pub symbol: String,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} ({})",
            self.name,
            self.symbol,
            self.address
                .parse::<H160>()
                .unwrap_or(H160::zero())
                .to_string()
        )
    }
}

enum ExternalTokenError {
    NetworkNotSupported,
    ReqwestError(reqwest::Error),
}

impl From<reqwest::Error> for ExternalTokenError {
    fn from(err: reqwest::Error) -> ExternalTokenError {
        ExternalTokenError::ReqwestError(err)
    }
}

impl Token {
    pub fn unknown() -> Self {
        Self {
            address: H160::zero().to_string(),
            chainId: Some(0),
            decimals: 0,
            logoURI: String::default(),
            name: "Unknown".to_owned(),
            symbol: String::default(),
        }
    }

    fn supported_networks_ids() -> HashMap<&'static str, &'static str> {
        HashMap::from([
            ("Avalanche", "avalanche"),
            ("Dogechain", "dogechain"),
            ("Optimism", "optimistic-ethereum"),
            ("Arbitrum", "arbitrum-one"),
            ("Aurora", "aurora"),
        ])
    }

    pub fn get_native_wrapped(chain_id: u32) -> H160 {
        let wrapped_by_chain: HashMap<u32, String> = HashMap::from([
            // @dev WAVAX
            (
                43114,
                "0xb31f66aa3c1e785363f0875a1b74e27b85fd66c7".to_owned(),
            ),
            // @dev WDOGE
            (
                2000,
                "0xb7ddc6414bf4f5515b52d8bdd69973ae205ff101".to_owned(),
            ),
            // @dev Arbitrum WETH (need test)
            (
                42161,
                "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1".to_owned(),
            ),
            // @dev Optimism WETH (need test)
            (10, "0x4200000000000000000000000000000000000006".to_owned()),
        ]);

        wrapped_by_chain
            .get(&chain_id)
            .expect("No wrapped in chain")
            .parse::<H160>()
            .unwrap()
    }

    // @todo memoize
    #[tokio::main]
    pub async fn get_tokens() -> Vec<Token> {
        let cur_network = Network::get_current_network();

        let coingecko_response = Self::get_external_tokens(&cur_network).await;
        let mut coingecko_tokens: Vec<Token> = vec![];

        match coingecko_response {
            Ok(tokens) => {
                coingecko_tokens = tokens;
            }
            Err(err) => match err {
                ExternalTokenError::NetworkNotSupported => {
                    println!(
                        "{}",
                        style("Network not supported to get coingecko tokens, only locally added tokens shown").red()
                    );
                }
                ExternalTokenError::ReqwestError(error) => {
                    println!(
                        "{} {}",
                        style("Error while getting tokens from coingecko, only locally added tokens shown").red(),
                        error
                    );
                }
            },
        }

        // @dev join tokens from external source with local tokens
        let local_tokens = token_storage::TokenStorage::get_local_tokens();

        let mut tokens_current_chain: Vec<Token> = local_tokens
            .into_iter()
            .filter(|token| token.chainId == Some(cur_network.chain_id))
            .collect();

        tokens_current_chain.append(&mut coingecko_tokens);

        let native_token = Token {
            address: "0x0000000000000000000000000000000000000000".to_owned(),
            chainId: Some(cur_network.chain_id),
            decimals: 18,
            logoURI: "".to_owned(),
            name: cur_network.currency_name,
            symbol: cur_network.currency_symbol,
        };

        tokens_current_chain.push(native_token);

        tokens_current_chain
    }

    async fn get_external_tokens(cur_network: &Network) -> Result<Vec<Token>, ExternalTokenError> {
        let supported_networks_ids = Self::supported_networks_ids();

        let cur_network_id = supported_networks_ids.get(&*cur_network.short_name);

        if cur_network_id.is_none() {
            return Err(ExternalTokenError::NetworkNotSupported);
        }

        let external_url = format!(
            "https://tokens.coingecko.com/{}/all.json",
            cur_network_id.unwrap()
        )
        .to_owned();

        let response = reqwest::get(&external_url).await?;
        let mut coingecko = response.json::<CoingeckoResponse>().await?;

        Ok(coingecko.tokens)
    }

    pub fn is_native(address: H160) -> bool {
        address.is_zero()
    }

    #[tokio::main]
    pub async fn get_token_balance(
        owner: H160,
        token_address: H160,
        current_network: Arc<Network>,
    ) -> U256 {
        let provider = Arc::new(
            Provider::<Http>::try_from(current_network.rpc_url.to_owned())
                .expect("could not instantiate HTTP Provider"),
        );

        let token_contract = ERC20::new(token_address, provider);

        let token_balance = token_contract.balance_of(owner).call().await.unwrap();

        token_balance
    }

    #[tokio::main]
    pub async fn get_allowance(
        owner: H160,
        spender: H160,
        token_address: H160,
        current_network: Arc<Network>,
    ) -> U256 {
        let provider = Arc::new(
            Provider::<Http>::try_from(current_network.rpc_url.to_owned())
                .expect("could not instantiate HTTP Provider"),
        );

        let token_contract = ERC20::new(token_address, provider);

        let allowance = token_contract
            .allowance(owner, spender)
            .call()
            .await
            .unwrap();

        allowance
    }

    #[tokio::main]
    pub async fn approve(
        spender: H160,
        value: U256,
        token_address: H160,
        signer: &Wallet<SigningKey>,
        current_network: Arc<Network>,
    ) -> Option<TransactionReceipt> {
        let provider = Arc::new(
            Provider::<Http>::try_from(current_network.rpc_url.to_owned())
                .expect("could not instantiate HTTP Provider"),
        );

        let provider = Arc::new(
            SignerMiddleware::new_with_provider_chain(provider, signer.to_owned())
                .await
                .unwrap(),
        );

        let token_contract = ERC20::new(token_address, provider);

        let call = token_contract.approve(spender, value);
        let pending_tx = call.send().await.expect("Error when approve call");

        let receipt = pending_tx
            .await
            .expect("Error while getting confirmations on approve");

        receipt
    }

    #[tokio::main]
    pub async fn get_native_balance(current_address: H160, current_network: Arc<Network>) -> U256 {
        let provider = Arc::new(
            Provider::<Http>::try_from(current_network.rpc_url.to_owned())
                .expect("could not instantiate HTTP Provider"),
        );

        let blk = Some(BlockId::from(provider.get_block_number().await.unwrap()));

        let balance = provider.get_balance(current_address, blk).await.unwrap();

        balance
    }
}

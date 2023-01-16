use std::{
    ops::{Mul, Sub},
    sync::Arc,
};

use ethers::{
    prelude::{k256::ecdsa::SigningKey, SignerMiddleware},
    providers::{Http, Provider},
    signers::Wallet,
    types::{TransactionReceipt, H160, U256},
};

use crate::{
    abis::{Trade, YakRouter, IWETH},
    network::Network,
    settings::Settings,
    token::Token,
};

#[derive(Clone, Copy)]
pub enum FromToNative {
    FromNative,
    ToNative,
}

pub struct Swap {}

impl Swap {
    #[tokio::main]
    pub async fn swap_no_split(
        mut trade: Trade,
        to: H160,
        from_to_native: Option<FromToNative>,
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

        let yak_router_contract = YakRouter::new(
            current_network
                .yak_router
                .as_ref()
                .unwrap()
                .parse::<H160>()
                .unwrap(),
            provider,
        );

        trade.handle_slippage_setting();

        // if trade path starts from avax swap_no_split_from_avax
        // else if trade path to avax swap_no_split_to_avax
        if let Some(from_to_native) = from_to_native {
            match from_to_native {
                FromToNative::FromNative => {
                    let value_amount = trade.amount_in;

                    let call = yak_router_contract
                        .swap_no_split_from_avax(trade, to, U256::from(0))
                        .value(value_amount);
                    let pending_tx = call
                        .send()
                        .await
                        .expect("Error when swap no split from avax call");

                    let receipt = pending_tx
                        .await
                        .expect("Error while getting confirmations on swap no split from avax");

                    receipt
                }
                FromToNative::ToNative => {
                    let call = yak_router_contract.swap_no_split_to_avax(trade, to, U256::from(0));
                    let pending_tx = call
                        .send()
                        .await
                        .expect("Error when swap no split to avax call");

                    let receipt = pending_tx
                        .await
                        .expect("Error while getting confirmations on swap no split to avax");

                    receipt
                }
            }
        } else {
            let call = yak_router_contract.swap_no_split(trade, to, U256::from(0));
            let pending_tx = call.send().await.expect("Error when swap no split call");

            let receipt = pending_tx
                .await
                .expect("Error while getting confirmations on swap no split");

            receipt
        }
    }

    #[tokio::main]
    pub async fn wrap_native(
        amount_in: U256,
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

        let native_address = Token::get_native_wrapped(current_network.chain_id);

        let wrap_contract = IWETH::new(native_address, provider);

        let call = wrap_contract.deposit().value(amount_in);
        let pending_tx = call.send().await.expect("Error when wrap deposit call");

        let receipt = pending_tx
            .await
            .expect("Error while getting confirmations on wrap deposit");

        receipt
    }

    #[tokio::main]
    pub async fn unwrap_native(
        amount_in: U256,
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

        let native_address = Token::get_native_wrapped(current_network.chain_id);

        let wrap_contract = IWETH::new(native_address, provider);

        let call = wrap_contract.withdraw(amount_in);
        let pending_tx = call.send().await.expect("Error when wrap withdraw call");

        let receipt = pending_tx
            .await
            .expect("Error while getting confirmations on wrap withdraw");

        receipt
    }

    pub fn decide_from_to_native(address_from: H160, address_to: H160) -> Option<FromToNative> {
        if Token::is_native(address_from) {
            return Some(FromToNative::FromNative);
        }

        if Token::is_native(address_to) {
            return Some(FromToNative::ToNative);
        }

        None
    }
}

impl Trade {
    pub fn handle_slippage_setting(&mut self) {
        // @dev e.g. 5 = 0.5%
        let slippage = Settings::get_slippage();

        let amount_out_with_slippage = U256::from(self.amount_out).sub(
            self.amount_out
                .checked_div(U256::from(1000))
                .unwrap()
                .mul(U256::from(slippage)),
        );
        self.amount_out = amount_out_with_slippage;
    }
}

use std::{
    ops::{Mul, Sub},
    sync::Arc,
};

use ethers::{
    abi::{self, Address},
    prelude::{k256::ecdsa::SigningKey, SignerMiddleware},
    providers::{Http, Provider},
    signers::{Signer, Wallet},
    types::{TransactionReceipt, H160, H256, U256},
    utils::keccak256,
};

use crate::{
    abis::{Trade, YakRouter, ERC20, IWETH},
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

                    pending_tx
                        .await
                        .expect("Error while getting confirmations on swap no split from avax")
                }
                FromToNative::ToNative => {
                    let call = yak_router_contract.swap_no_split_to_avax(trade, to, U256::from(0));
                    let pending_tx = call
                        .send()
                        .await
                        .expect("Error when swap no split to avax call");

                    pending_tx
                        .await
                        .expect("Error while getting confirmations on swap no split to avax")
                }
            }
        } else {
            let call = yak_router_contract.swap_no_split(trade, to, U256::from(0));
            let pending_tx = call.send().await.expect("Error when swap no split call");

            pending_tx
                .await
                .expect("Error while getting confirmations on swap no split")
        }
    }

    #[tokio::main]
    pub async fn swap_no_split_with_permit(
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
            provider.clone(),
        );

        let token_in_contract = ERC20::new(trade.path[0], provider.clone());

        trade.handle_slippage_setting();

        let default_deadline = U256::MAX;

        let nonce_count = token_in_contract
            .nonces(signer.address())
            .call()
            .await
            .expect("Expect to get nonce");

        let owner: Address = signer.address();
        let spender: Address = current_network
            .yak_router
            .as_ref()
            .unwrap()
            .parse::<H160>()
            .unwrap();
        let value = trade.amount_in;
        let nonce = nonce_count;
        let deadline = default_deadline;
        let verifying_contract: Address = trade.path[0];
        let name = "Yak Token";
        let version = "1";
        let chainid = current_network.chain_id;

        // Typehash for the permit() function
        let permit_typehash = keccak256(
            "Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)",
        );
        // Typehash for the struct used to generate the domain separator
        let domain_typehash = keccak256(
            "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)",
        );

        // Corresponds to solidity's abi.encode()
        let domain_separator_input = abi::encode(&[
            ethers::abi::Token::Uint(U256::from(domain_typehash)),
            ethers::abi::Token::Uint(U256::from(keccak256(name))),
            ethers::abi::Token::Uint(U256::from(keccak256(version))),
            ethers::abi::Token::Uint(U256::from(chainid)),
            ethers::abi::Token::Address(verifying_contract),
        ]);

        let domain_separator = keccak256(&domain_separator_input);

        let struct_input = abi::encode(&vec![
            ethers::abi::Token::Uint(U256::from(permit_typehash)),
            ethers::abi::Token::Address(owner),
            ethers::abi::Token::Address(spender),
            ethers::abi::Token::Uint(value),
            ethers::abi::Token::Uint(nonce),
            ethers::abi::Token::Uint(deadline),
        ]);
        let struct_hash = keccak256(&struct_input);

        let digest_input = [
            &[0x19, 0x01],
            domain_separator.as_ref(),
            struct_hash.as_ref(),
        ]
        .concat();

        let permit_hash = <H256>::from(keccak256(&digest_input));

        let signature = signer.sign_hash(permit_hash);

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

                    pending_tx
                        .await
                        .expect("Error while getting confirmations on swap no split from avax")
                }
                FromToNative::ToNative => {
                    let call = yak_router_contract.swap_no_split_to_avax_with_permit(
                        trade,
                        to,
                        U256::from(0),
                        default_deadline,
                        signature.v as u8,
                        <[u8; 32]>::from(signature.r),
                        <[u8; 32]>::from(signature.s),
                    );
                    let pending_tx = call
                        .send()
                        .await
                        .expect("Error when swap no split to avax call");

                    pending_tx
                        .await
                        .expect("Error while getting confirmations on swap no split to avax")
                }
            }
        } else {
            let call = yak_router_contract.swap_no_split_with_permit(
                trade,
                to,
                U256::from(0),
                default_deadline,
                signature.v as u8,
                <[u8; 32]>::from(signature.r),
                <[u8; 32]>::from(signature.s),
            );
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

        pending_tx
            .await
            .expect("Error while getting confirmations on wrap deposit")
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

        pending_tx
            .await
            .expect("Error while getting confirmations on wrap withdraw")
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

        let amount_out_with_slippage = self.amount_out.sub(
            self.amount_out
                .checked_div(U256::from(1000))
                .unwrap()
                .mul(U256::from(slippage)),
        );
        self.amount_out = amount_out_with_slippage;
    }
}

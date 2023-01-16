use std::sync::Arc;

use crate::abis::Trade;
use crate::db::DB;
use crate::network::Network;
use crate::swap::{FromToNative, Swap};
use crate::terminal::storage::WalletStorage;
use crate::Terminal;
use crate::{token::Token, wallet::AccountWallet};
use console::{style, Term};
use dialoguer::{theme::ColorfulTheme, FuzzySelect, Input, Select};
use dialoguer::{Confirm, Password};
use ethers::{
    prelude::k256::elliptic_curve::Error,
    types::{H160, U256},
    utils::{format_units, parse_units},
};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use spinners::{Spinner, Spinners};

#[path = "./query.rs"]
mod query;

#[path = "../wallet/storage.rs"]
mod wallet_storage;

pub struct SwapScreen {}

pub struct SwapPrompt {
    amount_in: U256,
    token_in: Token,
    token_out: Token,
}

#[derive(FromPrimitive)]
enum SwapTopics {
    Swap,
    WrapNative,
    UnwrapNative,
    Back,
}

impl SwapScreen {
    pub fn render() -> std::io::Result<()> {
        let topics = [
            "1. Swap tokens",
            "2. Wrap native token",
            "3. Unwrap native token",
            "<- Go back",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .items(&topics)
            .default(0)
            .interact_on_opt(&Term::stderr())?;

        match selection {
            Some(index) => match FromPrimitive::from_usize(index) {
                Some(SwapTopics::Swap) => {
                    Self::swap();
                    Terminal::render();
                }
                Some(SwapTopics::WrapNative) => {
                    Self::wrap_native();
                    Terminal::render();
                }
                Some(SwapTopics::UnwrapNative) => {
                    Self::unwrap_native();
                    Terminal::render();
                }
                Some(SwapTopics::Back) => {
                    Terminal::render();
                }
                None => panic!("Error while selecting swap screen topic"),
            },
            None => println!("You did not select anything"),
        }

        Ok(())
    }

    fn swap() {
        // @todo check if token contract has permit, than no need in approve
        // @todo suggest to use permit2, onboard user somehow
        let prompt_query = query::QueryScreen::prompt_query();

        let mut sp = Spinner::new(Spinners::Aesthetic, "Getting best offer...".into());

        let find_path_result = crate::query::Query::find_best_path_with_gas(
            prompt_query.amount_in,
            prompt_query.token_in.address.parse::<H160>().unwrap(),
            prompt_query.token_out.address.parse::<H160>().unwrap(),
            prompt_query.max_steps,
        );

        sp.stop_with_newline();

        let formatted_offer = find_path_result.ok().expect("Error when getting best path");

        if formatted_offer.adapters.is_empty() {
            println!("Path not found 😔");
            return;
        }

        query::QueryScreen::format_offer_result(
            formatted_offer.to_owned(),
            prompt_query.token_out.to_owned(),
        );

        let confirm = Confirm::new()
            .with_prompt("Do you want to continue?")
            .default(true)
            .interact()
            .unwrap();

        if !confirm {
            println!("Ok, next time");
            return;
        }

        let db_instance = DB.lock().unwrap();

        let current_wallet = db_instance.get::<WalletStorage>(WalletStorage::DB_CURRENT_WALLET);

        drop(db_instance);

        if let Some(current_wallet) = current_wallet {
            let password: String = Password::new()
                .with_prompt("Current Wallet password")
                .interact()
                .unwrap();

            let wallet =
                crate::wallet::AccountWallet::decrypt_wallet(current_wallet.name, password);

            if !wallet.is_ok() {
                return;
            }

            let wallet = wallet.expect("Something wrong with wallet");

            let current_network = Arc::new(Network::get_current_network());

            let signing_wallet = wallet.wallet();

            let from_to_native = Swap::decide_from_to_native(
                prompt_query.token_in.address.parse::<H160>().unwrap(),
                prompt_query.token_out.address.parse::<H160>().unwrap(),
            );

            let mut sp = Spinner::new(Spinners::Aesthetic, "Getting current balance...".into());

            let mut token_in_balance = U256::zero();
            let mut is_from_native = false;

            if let Some(from_to_native) = from_to_native {
                match from_to_native {
                    FromToNative::FromNative => {
                        is_from_native = true;
                        token_in_balance = Token::get_native_balance(
                            current_wallet.address,
                            current_network.clone(),
                        );
                    }
                    _ => {
                        token_in_balance = crate::token::Token::get_token_balance(
                            current_wallet.address,
                            prompt_query.token_in.address.parse::<H160>().unwrap(),
                            current_network.clone(),
                        );
                    }
                }
            } else {
                token_in_balance = crate::token::Token::get_token_balance(
                    current_wallet.address,
                    prompt_query.token_in.address.parse::<H160>().unwrap(),
                    current_network.clone(),
                );
            }

            sp.stop_with_newline();

            if token_in_balance < prompt_query.amount_in {
                println!(
                    "Balance of {} less than amount you want to swap",
                    prompt_query.token_in.symbol
                );
                // @todo try other amount or token
                return;
            }

            let yak_router_address = current_network
                .yak_router
                .as_ref()
                .unwrap()
                .parse::<H160>()
                .unwrap();

            let mut allowance = U256::zero();
            if !is_from_native {
                allowance = crate::token::Token::get_allowance(
                    current_wallet.address,
                    yak_router_address,
                    prompt_query.token_in.address.parse::<H160>().unwrap(),
                    current_network.clone(),
                );
            }

            if !is_from_native && allowance < prompt_query.amount_in {
                println!(
                    "Allowance of {} less than amount you want to swap",
                    prompt_query.token_in.symbol
                );

                let confirm = Confirm::new()
                    .with_prompt("Do you want to approve tokens to spend?")
                    .default(true)
                    .interact()
                    .unwrap();

                if !confirm {
                    println!("Ok, next time");
                    return;
                }

                let mut sp = Spinner::new(Spinners::Aesthetic, "Approving...".into());

                let approve_receipt = crate::token::Token::approve(
                    yak_router_address,
                    U256::MAX,
                    prompt_query.token_in.address.parse::<H160>().unwrap(),
                    signing_wallet,
                    current_network.clone(),
                );

                sp.stop_with_newline();

                if let Some(approve_receipt) = approve_receipt {
                    println!("TX Hash: {}", approve_receipt.transaction_hash);
                } else {
                    println!("Error when getting tx hash on approve");
                }
            }

            // spinner & swap
            let mut sp = Spinner::new(Spinners::Aesthetic, "Swapping...".into());

            let trade = Trade {
                amount_in: *formatted_offer.amounts.first().unwrap(),
                amount_out: *formatted_offer.amounts.last().unwrap(),
                path: formatted_offer.path,
                adapters: formatted_offer.adapters,
            };

            let swap_receipt = Swap::swap_no_split(
                trade,
                current_wallet.address,
                from_to_native,
                signing_wallet,
                current_network.clone(),
            );

            sp.stop_with_newline();

            if let Some(swap_receipt) = swap_receipt {
                println!("{}", style("Hooray, successful swap!").green());
                let tx_url = format!(
                    "{explorer}/tx/{:?}",
                    swap_receipt.transaction_hash,
                    explorer = current_network.explorer_url
                );
                println!("tx url: {}", tx_url);
            } else {
                println!("Error when getting tx hash on swap");
            }
        } else {
            println!("No wallet set");
        }
    }

    fn wrap_native() {
        let current_wallet = wallet_storage::WalletStorage::get_current_wallet();

        if let Some(current_wallet) = current_wallet {
            let current_network = Arc::new(Network::get_current_network());

            let amount_input = Input::<String>::new()
                .with_prompt("Amount to Wrap")
                .interact_text()
                .unwrap();

            let amount_in = parse_units(amount_input, 18).unwrap();

            let mut sp = Spinner::new(Spinners::Aesthetic, "Getting current balance...".into());

            let native_balance =
                Token::get_native_balance(current_wallet.address, current_network.clone());

            sp.stop_with_newline();

            if amount_in > native_balance {
                println!("Not enough balance");
                return;
            }

            let password: String = Password::new()
                .with_prompt("Current Wallet password")
                .interact()
                .unwrap();

            let wallet =
                crate::wallet::AccountWallet::decrypt_wallet(current_wallet.name, password)
                    .expect("Wrong password or account not set");

            let signing_wallet = wallet.wallet();

            let mut sp = Spinner::new(Spinners::Aesthetic, "Wrapping tokens...".into());

            let receipt = Swap::wrap_native(amount_in, signing_wallet, current_network.clone());

            sp.stop_with_newline();

            if let Some(receipt) = receipt {
                println!("{}", style("Hooray, successful wrap!").green());
                let tx_url = format!(
                    "{explorer}/tx/{:?}",
                    receipt.transaction_hash,
                    explorer = current_network.explorer_url
                );
                println!("tx url: {}", tx_url);
            } else {
                println!("Error when getting tx hash on wrap");
            }
        } else {
            println!("No current wallet set");
        }
    }

    fn unwrap_native() {
        let current_wallet = wallet_storage::WalletStorage::get_current_wallet();

        if let Some(current_wallet) = current_wallet {
            let current_network = Arc::new(Network::get_current_network());

            let amount_input = Input::<String>::new()
                .with_prompt("Amount to Unwrap")
                .interact_text()
                .unwrap();

            let amount_in = parse_units(amount_input, 18).unwrap();

            let mut sp = Spinner::new(Spinners::Aesthetic, "Getting current balance...".into());

            let wrapped_token = Token::get_native_wrapped(current_network.chain_id);

            let token_balance = Token::get_token_balance(
                current_wallet.address,
                wrapped_token,
                current_network.clone(),
            );

            sp.stop_with_newline();

            if amount_in > token_balance {
                println!("Not enough balance");
                return;
            }

            let password: String = Password::new()
                .with_prompt("Current Wallet password")
                .interact()
                .unwrap();

            let wallet =
                crate::wallet::AccountWallet::decrypt_wallet(current_wallet.name, password)
                    .expect("Wrong password or account not set");

            let signing_wallet = wallet.wallet();

            let mut sp = Spinner::new(Spinners::Aesthetic, "Unwrapping tokens...".into());

            let receipt = Swap::unwrap_native(amount_in, signing_wallet, current_network.clone());

            sp.stop_with_newline();

            if let Some(receipt) = receipt {
                println!("{}", style("Hooray, successful unwrap!").green());
                let tx_url = format!(
                    "{explorer}/tx/{:?}",
                    receipt.transaction_hash,
                    explorer = current_network.explorer_url
                );
                println!("tx url: {}", tx_url);
            } else {
                println!("Error when getting tx hash on unwrap");
            }
        } else {
            println!("No current wallet set");
        }
    }
}

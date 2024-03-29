use std::sync::Arc;

use crate::abis::Trade;
use crate::db::DB;
use crate::network::Network;
use crate::query::{ExternalQuote, ExternalQuoteError, Query};
use crate::settings::Settings;
use crate::swap::{FromToNative, Swap};
use crate::terminal::storage::WalletStorage;
use crate::token::Token;
use crate::Terminal;
use console::{style, Term};
use dialoguer::{theme::ColorfulTheme, Input, Select};
use dialoguer::{Confirm, Password};
use ethers::{
    types::{H160, U256},
    utils::parse_units,
};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use spinners::{Spinner, Spinners};

use super::query::QueryScreen;

use crate::wallet::storage;

pub struct SwapScreen {}

#[derive(FromPrimitive)]
enum SwapTopics {
    Swap,
    WrapNative,
    UnwrapNative,
    Back,
}

impl SwapScreen {
    pub fn render() {
        let topics = [
            "1. Swap tokens",
            "2. Wrap native token",
            "3. Unwrap native token",
            "<- Go back",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .items(&topics)
            .default(0)
            .interact_on_opt(&Term::stderr())
            .unwrap();

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
    }

    fn swap() {
        let prompt_query = QueryScreen::prompt_query();

        let mut sp = Spinner::new(Spinners::Aesthetic, "Getting best offer...".into());

        let find_path_result = crate::query::Query::find_best_path_with_gas(
            prompt_query.amount_in,
            prompt_query.token_in.address.parse::<H160>().unwrap(),
            prompt_query.token_out.address.parse::<H160>().unwrap(),
            prompt_query.max_steps,
        );

        sp.stop_with_newline();

        let formatted_offer = find_path_result.ok().expect("Error when getting best path");

        // @dev start external price fetching
        let is_external_allowed = Settings::is_external_allowed();

        let mut external_quote_result = ExternalQuote {
            to_token_amount: String::default(),
            estimated_gas: 0,
        };

        if is_external_allowed {
            let mut sp = Spinner::new(Spinners::Aesthetic, "Getting quote from 1inch...".into());

            // get best path from 1inch
            let external_quote = Query::get_1inch_price(
                prompt_query.amount_in,
                prompt_query.token_in.address.parse::<H160>().unwrap(),
                prompt_query.token_out.address.parse::<H160>().unwrap(),
            );

            match external_quote {
                Ok(quote) => {
                    external_quote_result = quote;
                    sp.stop_with_message("Finished getting quote from 1inch ✅".to_owned());
                }
                Err(err) => match err {
                    ExternalQuoteError::NetworkNotSupported => {
                        println!("Network not supported to get 1inch price");
                        sp.stop_with_message("Error getting quote from 1inch ⛔️".to_owned());
                    }
                    ExternalQuoteError::ReqwestError(err) => {
                        println!("Error while requesting 1inch price {}", err);
                        sp.stop_with_message("Error getting quote from 1inch ⛔️".to_owned());
                    }
                },
            }
        }
        // @dev end external price fetching

        if formatted_offer.adapters.is_empty() {
            println!("Path not found 😔");

            if is_external_allowed && !external_quote_result.to_token_amount.is_empty() {
                let gas_price = Query::get_gas_price();

                println!("But 1inch found offer");
                QueryScreen::format_external_offer(
                    external_quote_result,
                    None,
                    prompt_query.token_out,
                    &gas_price,
                );
            }

            return;
        }

        let gas_price = Query::get_gas_price();

        QueryScreen::format_offer_result(
            formatted_offer.to_owned(),
            prompt_query.token_out.to_owned(),
            &gas_price,
        );

        if is_external_allowed {
            QueryScreen::format_external_offer(
                external_quote_result,
                Some(formatted_offer.to_owned()),
                prompt_query.token_out.to_owned(),
                &gas_price,
            );
        }

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

            if wallet.is_err() {
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

            let token_in_balance: U256;
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

            let has_permit =
                Query::has_permit(prompt_query.token_in.address.parse::<H160>().unwrap());

            // @dev check allowance
            let mut allowance = U256::zero();
            if !is_from_native {
                allowance = crate::token::Token::get_allowance(
                    current_wallet.address,
                    yak_router_address,
                    prompt_query.token_in.address.parse::<H160>().unwrap(),
                    current_network.clone(),
                );
            }

            let need_permit = has_permit && allowance < prompt_query.amount_in;

            // @dev approve tokens
            if !is_from_native && !has_permit && allowance < prompt_query.amount_in {
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

            let swap_receipt = if !need_permit {
                Swap::swap_no_split(
                    trade,
                    current_wallet.address,
                    from_to_native,
                    signing_wallet,
                    current_network.clone(),
                )
            } else {
                Swap::swap_no_split_with_permit(
                    trade,
                    current_wallet.address,
                    from_to_native,
                    signing_wallet,
                    current_network.clone(),
                )
            };

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
        let current_wallet = storage::WalletStorage::get_current_wallet();

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
        let current_wallet = storage::WalletStorage::get_current_wallet();

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

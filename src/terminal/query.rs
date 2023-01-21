use crate::abis::FormattedOfferWithGas;
use crate::db::DB;
use crate::network::Network;
use crate::query::adapters::Adapter;
use crate::query::{ExternalQuote, ExternalQuoteError, Query};
use crate::settings::Settings;
use crate::swap::{FromToNative, Swap};
use crate::token::Token;
use crate::Terminal;
use console::style;
use console::Term;
use dialoguer::{theme::ColorfulTheme, FuzzySelect, Input, Select};
use ethers::providers::ProviderError;
use ethers::{
    types::{H160, U256},
    utils::{format_units, parse_units},
};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use spinners::{Spinner, Spinners};
use std::ops::Mul;
use std::sync::Arc;

use crate::wallet::storage::WalletStorage;

pub struct QueryScreen {}

pub struct QueryPrompt {
    pub amount_in: U256,
    pub token_in: Token,
    pub token_out: Token,
    pub max_steps: i32,
}

#[derive(FromPrimitive)]
enum QueryTopics {
    BestPath,
    SingleAdapter,
    List,
    Back,
}

impl QueryScreen {
    pub fn render() {
        let topics = [
            "1. Query best path",
            "2. Query single adapter",
            "3. List adapters",
            "<- Go back",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .items(&topics)
            .default(0)
            .interact_on_opt(&Term::stderr())
            .unwrap();

        match selection {
            Some(index) => match FromPrimitive::from_usize(index) {
                Some(QueryTopics::BestPath) => {
                    let prompt_query = Self::prompt_query();

                    let mut sp = Spinner::new(Spinners::Aesthetic, "Getting best path...".into());

                    let find_path_result = Query::find_best_path_with_gas(
                        prompt_query.amount_in,
                        prompt_query.token_in.address.parse::<H160>().unwrap(),
                        prompt_query.token_out.address.parse::<H160>().unwrap(),
                        prompt_query.max_steps,
                    );

                    sp.stop_with_message("Finished getting best path ✅".to_owned());

                    // @dev start external price fetching
                    let is_external_allowed = Settings::is_external_allowed();

                    let mut external_quote_result = ExternalQuote {
                        to_token_amount: String::default(),
                        estimated_gas: 0,
                    };

                    if is_external_allowed {
                        let mut sp =
                            Spinner::new(Spinners::Aesthetic, "Getting quote from 1inch...".into());

                        // get best path from 1inch
                        let external_quote = Query::get_1inch_price(
                            prompt_query.amount_in,
                            prompt_query.token_in.address.parse::<H160>().unwrap(),
                            prompt_query.token_out.address.parse::<H160>().unwrap(),
                        );

                        match external_quote {
                            Ok(quote) => {
                                external_quote_result = quote;
                                sp.stop_with_message(
                                    "Finished getting quote from 1inch ✅".to_owned(),
                                );
                            }
                            Err(err) => match err {
                                ExternalQuoteError::NetworkNotSupported => {
                                    println!("Network not supported to get 1inch price");
                                    sp.stop_with_message(
                                        "Error getting quote from 1inch ⛔️".to_owned(),
                                    );
                                }
                                ExternalQuoteError::ReqwestError(err) => {
                                    println!("Error while requesting 1inch price {}", err);
                                    sp.stop_with_message(
                                        "Error getting quote from 1inch ⛔️".to_owned(),
                                    );
                                }
                            },
                        }
                    }
                    // @dev end external price fetching

                    match find_path_result {
                        Ok(formatted_offer) => {
                            if formatted_offer.adapters.is_empty() {
                                println!("Yak path not found 😔");

                                if is_external_allowed
                                    && !external_quote_result.to_token_amount.is_empty()
                                {
                                    let gas_price = Query::get_gas_price();

                                    println!("But 1inch found offer");
                                    Self::format_external_offer(
                                        external_quote_result,
                                        None,
                                        prompt_query.token_out.to_owned(),
                                        &gas_price,
                                    );
                                }

                                ()
                            } else {
                                let gas_price = Query::get_gas_price();

                                Self::format_offer_result(
                                    formatted_offer.to_owned(),
                                    prompt_query.token_out.to_owned(),
                                    &gas_price,
                                );

                                if is_external_allowed {
                                    Self::format_external_offer(
                                        external_quote_result,
                                        Some(formatted_offer),
                                        prompt_query.token_out,
                                        &gas_price,
                                    );
                                }
                            }
                        }
                        Err(err) => {
                            println!("{}", err);
                        }
                    }

                    Terminal::render();
                }
                Some(QueryTopics::SingleAdapter) => {
                    let adapters = Query::get_adapters();

                    let adapter_selection = Select::with_theme(&ColorfulTheme::default())
                        .items(&adapters)
                        .default(0)
                        .interact_on_opt(&Term::stderr())
                        .unwrap();

                    match adapter_selection {
                        Some(index) => {
                            let prompt_query = Self::prompt_query();

                            let mut sp =
                                Spinner::new(Spinners::Aesthetic, "Querying adapter...".into());

                            let amount_out = Query::query_adapter(
                                adapters[index].address,
                                prompt_query.amount_in,
                                prompt_query.token_in.address.parse::<H160>().unwrap(),
                                prompt_query.token_out.address.parse::<H160>().unwrap(),
                            );

                            sp.stop_with_message("Finished ✅".to_owned());

                            println!();

                            match amount_out {
                                Ok(amount_out) => {
                                    println!(
                                        "You receive: {} {}",
                                        format_units(amount_out, prompt_query.token_out.decimals)
                                            .unwrap(),
                                        prompt_query.token_out.symbol
                                    );
                                }
                                Err(err) => {
                                    println!("{}", err);
                                }
                            }
                        }
                        None => println!("User did not select anything"),
                    }

                    Self::render();
                }
                Some(QueryTopics::List) => {
                    let mut sp = Spinner::new(Spinners::Aesthetic, "Getting adapters...".into());
                    let adapters = Query::get_adapters();
                    sp.stop_with_message("Finished loading adapters ✅".to_owned());

                    println!();
                    for adapter in adapters {
                        println!("{}", adapter);
                    }
                    println!();

                    Self::render();
                }
                Some(QueryTopics::Back) => {
                    Terminal::render();
                }
                None => panic!("Error while selecting account screen topic"),
            },
            None => println!("You did not select anything"),
        }
    }

    pub fn prompt_query() -> QueryPrompt {
        let db_instance = DB.lock().unwrap();

        let current_wallet = db_instance.get::<WalletStorage>(WalletStorage::DB_CURRENT_WALLET);

        drop(db_instance);

        let tokens = crate::token::Token::get_tokens();

        let token_in_selection = FuzzySelect::with_theme(&ColorfulTheme::default())
            .items(&tokens)
            .with_prompt("Token in")
            .default(0)
            .interact_on_opt(&Term::stderr())
            .unwrap();

        let token_in = &tokens[token_in_selection.unwrap()];

        // @dev getting balance here is optional, so we dont want to panic if no account set or call failed
        if let Some(current_wallet) = current_wallet {
            let token_in_balance = Self::get_token_in_balance(token_in.to_owned(), current_wallet);

            if let Ok(balance) = token_in_balance {
                println!(
                    "You have {} {}",
                    format_units(balance, token_in.decimals).unwrap(),
                    token_in.symbol
                );
            }
        }

        let amount_input = Input::<String>::new()
            .with_prompt("Amount In")
            .interact_text()
            .unwrap();

        let token_out_selection = FuzzySelect::with_theme(&ColorfulTheme::default())
            .items(&tokens)
            .with_prompt("Token out")
            .default(0)
            .interact_on_opt(&Term::stderr())
            .unwrap();

        // @dev configure how many steps you want to search the path with
        let max_steps = Settings::get_max_steps();

        let token_out = &tokens[token_out_selection.unwrap()];

        let amount_in = parse_units(amount_input, token_in.decimals).unwrap();

        QueryPrompt {
            amount_in,
            token_in: token_in.to_owned(),
            token_out: token_out.to_owned(),
            max_steps,
        }
    }

    fn get_token_in_balance(token_in: Token, current_wallet: WalletStorage) -> Result<U256, ()> {
        let current_network = Arc::new(Network::get_current_network());

        let from_to_native =
            Swap::decide_from_to_native(token_in.address.parse::<H160>().unwrap(), H160::zero());

        let mut sp = Spinner::new(Spinners::Aesthetic, "Getting current balance...".into());

        let token_in_balance: U256;

        if let Some(from_to_native) = from_to_native {
            match from_to_native {
                FromToNative::FromNative => {
                    token_in_balance =
                        Token::get_native_balance(current_wallet.address, current_network);
                }
                _ => {
                    token_in_balance = crate::token::Token::get_token_balance(
                        current_wallet.address,
                        token_in.address.parse::<H160>().unwrap(),
                        current_network,
                    );
                }
            }
        } else {
            token_in_balance = crate::token::Token::get_token_balance(
                current_wallet.address,
                token_in.address.parse::<H160>().unwrap(),
                current_network,
            );
        }

        sp.stop_with_message("Finished getting balance ✅".to_owned());

        Ok(token_in_balance)
    }

    pub fn format_offer_result(
        formatted_offer: FormattedOfferWithGas,
        token_out: Token,
        gas_price: &Result<U256, ProviderError>,
    ) {
        let current_network = Network::get_current_network();
        let tokens = crate::token::Token::get_tokens();

        let path = formatted_offer
            .path
            .into_iter()
            .map(|addr| {
                let token = tokens
                    .iter()
                    .find(|token| token.address.parse::<H160>().unwrap() == addr);
                token.unwrap_or(&Token::unknown()).to_string()
            })
            .collect::<Vec<String>>();

        let network_adapters = Query::get_adapters();

        let adapters = formatted_offer
            .adapters
            .into_iter()
            .map(|addr| {
                let adapter = network_adapters
                    .iter()
                    .find(|adapter| adapter.address == addr);
                adapter
                    .unwrap_or(&Adapter {
                        name: "Unknown".to_owned(),
                        address: H160::zero(),
                    })
                    .to_string()
            })
            .collect::<Vec<String>>();

        let estimated_gas = match gas_price {
            Ok(gas_price) => {
                format!(
                    "{} {}",
                    style(
                        format_units(gas_price.mul(formatted_offer.gas_estimate), "ether")
                            .unwrap()
                            .parse::<f64>()
                            .unwrap()
                    )
                    .green(),
                    current_network.currency_symbol
                )
            }
            Err(err) => {
                format!("error getting gas price {}", err)
            }
        };

        println!();
        println!("Yak Offer Path:");
        println!("Adapters: {}", adapters.join(" => "));
        println!("Tokens: {}", path.join(" => "));
        println!(
            "You will get: {:.4} {}",
            style(
                format_units(formatted_offer.amounts.last().unwrap(), token_out.decimals)
                    .unwrap()
                    .parse::<f64>()
                    .unwrap()
            )
            .green(),
            token_out.symbol
        );
        println!("Estimated gas price: {}", estimated_gas);
        println!();
    }

    pub fn format_external_offer(
        offer: ExternalQuote,
        yak_offer: Option<FormattedOfferWithGas>,
        token_out: Token,
        gas_price: &Result<U256, ProviderError>,
    ) {
        let current_network = Network::get_current_network();

        let estimated_gas = match gas_price {
            Ok(gas_price) => {
                format!(
                    "{} {}",
                    style(
                        format_units(gas_price.mul(offer.estimated_gas), "ether")
                            .unwrap()
                            .parse::<f64>()
                            .unwrap()
                    )
                    .green(),
                    current_network.currency_symbol
                )
            }
            Err(err) => {
                format!("error getting gas price {}", err)
            }
        };

        if let Some(yak_offer) = yak_offer {
            let yak_offer_amount =
                format_units(yak_offer.amounts.last().unwrap(), token_out.decimals)
                    .unwrap()
                    .parse::<f64>()
                    .unwrap();
            let offer_amount = format_units(
                U256::from_dec_str(&offer.to_token_amount).unwrap(),
                token_out.decimals,
            )
            .unwrap()
            .parse::<f64>()
            .unwrap();

            let format_string = if yak_offer_amount < offer_amount {
                format!("1inch offer price: {:.2}", style(offer_amount).green())
            } else {
                format!("1inch offer price: {:.2}", style(offer_amount).red())
            };

            println!("{} {}", format_string, token_out.symbol);
            println!("1inch estimated gas: {}", estimated_gas);

            return;
        }

        println!();
        println!(
            "1inch offer price: {} {}",
            format_units(
                U256::from_dec_str(&offer.to_token_amount).unwrap(),
                token_out.decimals
            )
            .unwrap(),
            token_out.symbol
        );
        println!("1inch estimated gas: {}", estimated_gas);
        println!();
    }
}

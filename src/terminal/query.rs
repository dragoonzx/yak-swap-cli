use crate::network::Network;
use crate::Terminal;
use crate::{token::Token, wallet::AccountWallet};
use console::Term;
use dialoguer::{theme::ColorfulTheme, FuzzySelect, Input, Select};
use ethers::{
    prelude::k256::elliptic_curve::Error,
    types::{H160, U256},
    utils::{format_units, parse_units},
};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use spinners::{Spinner, Spinners};

pub struct QueryScreen {}

pub struct QueryPrompt {
    pub amount_in: U256,
    pub token_in: Token,
    pub token_out: Token,
}

#[derive(FromPrimitive)]
enum QueryTopics {
    BestPath,
    SingleAdapter,
    List,
    Back,
}

impl QueryScreen {
    pub fn render() -> std::io::Result<()> {
        let topics = [
            "1. Query best path",
            "2. Query single adapter",
            "3. List adapters",
            "<- Go back",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .items(&topics)
            .default(0)
            .interact_on_opt(&Term::stderr())?;

        match selection {
            Some(index) => match FromPrimitive::from_usize(index) {
                Some(QueryTopics::BestPath) => {
                    let prompt_query = Self::prompt_query();

                    let mut sp = Spinner::new(Spinners::Aesthetic, "Getting best path...".into());

                    let find_path_result = crate::query::Query::find_best_path_with_gas(
                        prompt_query.amount_in,
                        prompt_query.token_in.address.parse::<H160>().unwrap(),
                        prompt_query.token_out.address.parse::<H160>().unwrap(),
                    );

                    sp.stop_with_message("Finished getting best path ✅".to_owned());

                    match find_path_result {
                        Ok(formatted_offer) => {
                            let path = formatted_offer
                                .path
                                .into_iter()
                                .map(|addr| addr.to_string())
                                .collect::<Vec<String>>();

                            println!();
                            println!("Offer Path:");
                            println!("{}", path.join(" => "));
                            println!(
                                "You will get: {} {}",
                                format_units(
                                    formatted_offer.amounts.last().unwrap(),
                                    prompt_query.token_out.decimals
                                )
                                .unwrap(),
                                prompt_query.token_out.symbol
                            );
                            println!();
                        }
                        Err(err) => {
                            println!("{}", err);
                        }
                    }

                    Terminal::render();
                }
                Some(QueryTopics::SingleAdapter) => {
                    let adapters = crate::query::Query::get_adapters();

                    let adapter_selection = Select::with_theme(&ColorfulTheme::default())
                        .items(&adapters)
                        .default(0)
                        .interact_on_opt(&Term::stderr())?;

                    match adapter_selection {
                        Some(index) => {
                            let prompt_query = Self::prompt_query();

                            let mut sp =
                                Spinner::new(Spinners::Aesthetic, "Querying adapter...".into());

                            let amount_out = crate::query::Query::query_adapter(
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
                    let adapters = crate::query::Query::get_adapters();
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

        Ok(())
    }

    pub fn prompt_query() -> QueryPrompt {
        let current_network = Network::get_current_network();
        let tokens = crate::token::Token::get_tokens(current_network).unwrap();

        let token_in_selection = FuzzySelect::with_theme(&ColorfulTheme::default())
            .items(&tokens)
            .with_prompt("Token in")
            .default(0)
            .interact_on_opt(&Term::stderr())
            .unwrap();

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

        let token_in = &tokens[token_in_selection.unwrap()];
        let token_out = &tokens[token_out_selection.unwrap()];

        let amount_in = parse_units(amount_input, token_in.decimals).unwrap();

        QueryPrompt {
            amount_in,
            token_in: token_in.to_owned(),
            token_out: token_out.to_owned(),
        }
    }
}

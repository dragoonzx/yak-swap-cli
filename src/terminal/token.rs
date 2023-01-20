use crate::{token::token_storage::TokenStorage, token::Token, Terminal};
use console::{style, Term};
use dialoguer::{theme::ColorfulTheme, Input, Password, Select};
use ethers::prelude::k256::elliptic_curve::Error;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

pub struct TokenScreen {}

#[derive(FromPrimitive)]
enum TokenTopics {
    Add,
    Remove,
    Back,
}

impl TokenScreen {
    pub fn render() -> std::io::Result<()> {
        let topics = ["1. Add token", "2. Remove token", "<- Go back"];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .items(&topics)
            .default(0)
            .interact_on_opt(&Term::stderr())?;

        match selection {
            Some(index) => match FromPrimitive::from_usize(index) {
                Some(TokenTopics::Add) => {
                    Self::add_token();

                    println!("{}", style("Token successfully added!").green());

                    Terminal::render();
                }
                Some(TokenTopics::Remove) => {
                    Self::remove_token();

                    Terminal::render();
                }
                Some(TokenTopics::Back) => {
                    Terminal::render();
                }
                None => panic!("Error while selecting token screen topic"),
            },
            None => println!("You did not select anything"),
        }

        Ok(())
    }

    fn add_token() -> Result<Token, Error> {
        let name: String = Input::new()
            .with_prompt("Token Name")
            .interact_text()
            .unwrap();
        let symbol: String = Input::new()
            .with_prompt("Token Symbol")
            .interact_text()
            .unwrap();

        let chain_id: u32 = Input::new()
            .with_prompt("Token Chain Id")
            .interact_text()
            .unwrap();

        let address: String = Input::new()
            .with_prompt("Token Address")
            .interact_text()
            .unwrap();

        let decimals: u32 = Input::new()
            .with_prompt("Token Decimals")
            .interact_text()
            .unwrap();

        let token = TokenStorage::save_token(address, chain_id, decimals, name, symbol);

        Ok(token)
    }

    fn remove_token() {
        let local_tokens = TokenStorage::get_local_tokens();

        if local_tokens.is_empty() {
            println!("Empty list of tokens");
            return;
        }

        // select local token
        let selection = Select::with_theme(&ColorfulTheme::default())
            .items(&local_tokens)
            .default(0)
            .interact_on_opt(&Term::stderr())
            .unwrap();

        let selected_token = &local_tokens[selection.unwrap()];

        TokenStorage::remove_token(selected_token.to_owned());

        println!("{}", style("Token successfully removed!").green());
    }
}

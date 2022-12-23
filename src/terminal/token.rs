use crate::{token::Token, Terminal};
use console::Term;
use dialoguer::{theme::ColorfulTheme, Input, Password, Select};
use ethers::prelude::k256::elliptic_curve::Error;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

pub struct TokenScreen {}

#[path = "../token/storage.rs"]
mod token_storage;

#[derive(FromPrimitive)]
enum TokenTopics {
    Add,
    Back,
}

impl TokenScreen {
    pub fn render() -> std::io::Result<()> {
        let topics = ["1. Add token", "<- Go back"];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .items(&topics)
            .default(0)
            .interact_on_opt(&Term::stderr())?;

        match selection {
            Some(index) => match FromPrimitive::from_usize(index) {
                Some(TokenTopics::Add) => {
                    Self::add_token();

                    println!("Token successfully added!");

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

        let token =
            token_storage::TokenStorage::save_token(address, chain_id, decimals, name, symbol);

        Ok(token)
    }
}

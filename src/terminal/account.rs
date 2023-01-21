use crate::wallet::AccountWallet;
use crate::Terminal;
use console::Term;
use dialoguer::{theme::ColorfulTheme, Input, Password, Select};
use ethers::prelude::k256::elliptic_curve::Error;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

pub struct AccountScreen {}

#[derive(FromPrimitive)]
enum AccountTopics {
    Add,
    Remove,
    Set,
    Back,
}

impl AccountScreen {
    pub fn render() {
        let topics = [
            "1. Add account",
            "2. Remove account",
            "3. Set current account",
            "<- Go back",
        ];

        // print current accounts in storage to inform
        // also we need option to get back in main screen
        // + after we finish add, rm or set go to main screen
        AccountWallet::print_wallets();

        let selection = Select::with_theme(&ColorfulTheme::default())
            .items(&topics)
            .default(0)
            .interact_on_opt(&Term::stderr())
            .unwrap();

        match selection {
            Some(index) => match FromPrimitive::from_usize(index) {
                Some(AccountTopics::Add) => {
                    Self::add_account().unwrap();
                    Terminal::render();
                }
                Some(AccountTopics::Remove) => {
                    Self::remove_account();
                    Terminal::render();
                }
                Some(AccountTopics::Set) => {
                    Self::set_account();
                    Terminal::render();
                }
                Some(AccountTopics::Back) => {
                    Terminal::render();
                }
                None => panic!("Error while selecting account screen topic"),
            },
            None => println!("You did not select anything"),
        }
    }

    fn add_account() -> Result<AccountWallet, Error> {
        let name: String = Input::new()
            .with_prompt("Account Name")
            .interact_text()
            .unwrap();
        let private_key: String = Input::new()
            .with_prompt("Private Key")
            .interact_text()
            .unwrap();

        let password: String = Password::new()
            .with_prompt("Password")
            .with_confirmation("Confirm password", "Passwords mismatching")
            .interact()
            .unwrap();

        let account = AccountWallet::new(name, private_key, password);

        Ok(account)
    }

    fn set_account() {
        AccountWallet::set_wallet();
    }

    fn remove_account() {
        AccountWallet::remove_wallet();
    }
}

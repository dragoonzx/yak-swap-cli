use std::{
    io,
    io::{prelude::*, stdout},
};

use console::Term;
use crossterm::execute;
use dialoguer::{theme::ColorfulTheme, Input, Password, Select};
use ethers::{
    prelude::k256::{elliptic_curve::ScalarCore, Secp256k1, SecretKey},
    signers::{LocalWallet, Signer},
    types::H160,
};
use futures::executor::block_on;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use settimeout::set_timeout;
use spinners::{Spinner, Spinners};

use crate::db::DB;
use crate::network::Network;
use wallet::AccountWallet;

use account::AccountScreen;
use network::NetworkScreen;
use query::QueryScreen;
use storage::WalletStorage;
use swap::SwapScreen;
use token::TokenScreen;

mod account;
mod network;
mod query;
#[path = "../wallet/storage.rs"]
mod storage;
mod swap;
mod token;
#[path = "../wallet/mod.rs"]
mod wallet;

pub struct Terminal {}

#[derive(FromPrimitive)]
enum StartScreens {
    Query,
    Swap,
    Account,
    Network,
    Token,
}

impl Terminal {
    pub fn greet() {
        let welcome_message = "Yak Swap CLI v0.1 (beta)";
        println!("{}", welcome_message);
    }

    pub fn settings_bar() {
        let db_instance = DB.lock().unwrap();

        Terminal::clear_terminal();

        Terminal::greet();

        let current_wallet = db_instance.get::<WalletStorage>(WalletStorage::DB_CURRENT_WALLET);

        let mut address = "None".to_owned();

        if let Some(wallet) = current_wallet {
            address = wallet.address.to_string();
        }

        std::mem::drop(db_instance);

        let current_network = Network::get_current_network();

        println!();
        println!(
            "{}",
            format!(
                "Account: {} \t Network: {} (chain id: {}) \t RPC URL: {}",
                address, current_network.name, current_network.chain_id, current_network.rpc_url
            )
        );
        println!();
        println!(
            "Yak Router Contract: {}",
            current_network.yak_router.unwrap_or("None".to_owned())
        );
        println!();
    }

    pub fn render() -> std::io::Result<()> {
        Terminal::pause();

        Terminal::settings_bar();

        let start_screen_topics = [
            "1. Query",
            "2. Swap",
            "3. Account",
            "4. Network",
            "5. Tokens",
        ];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .items(&start_screen_topics)
            .default(0)
            .interact_on_opt(&Term::stderr())?;

        match selection {
            Some(index) => match FromPrimitive::from_usize(index) {
                Some(StartScreens::Query) => {
                    QueryScreen::render();
                }
                Some(StartScreens::Swap) => {
                    SwapScreen::render();
                }
                Some(StartScreens::Account) => {
                    AccountScreen::render();
                }
                Some(StartScreens::Network) => {
                    NetworkScreen::render();
                }
                Some(StartScreens::Token) => {
                    TokenScreen::render();
                }
                None => panic!("Error while selecting main screen topic"),
            },
            None => println!("You did not select anything"),
        }

        Ok(())
    }

    pub fn clear_terminal() {
        execute!(
            stdout(),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
        )
        .unwrap();
        execute!(stdout(), crossterm::cursor::MoveTo(0, 0)).unwrap();
    }

    fn pause() {
        let mut stdin = io::stdin();
        let mut stdout = io::stdout();

        // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
        write!(stdout, "Press any key to continue...").unwrap();
        stdout.flush().unwrap();

        // Read a single byte and discard
        let _ = stdin.read(&mut [0u8]).unwrap();
    }
}

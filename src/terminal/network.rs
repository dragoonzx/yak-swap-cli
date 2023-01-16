use crate::wallet::AccountWallet;
use crate::Terminal;
use console::Term;
use dialoguer::{theme::ColorfulTheme, Input, Password, Select};
use ethers::prelude::k256::elliptic_curve::Error;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use crate::network::Network;

pub struct NetworkScreen {}

#[derive(FromPrimitive)]
enum NetworkTopics {
    // Add,
    // Remove,
    Set,
    UpdateRPC,
    UpdateRouter,
    Back,
}

impl NetworkScreen {
    pub fn render() -> std::io::Result<()> {
        let topics = [
            // "1. Add network",
            // "2. Remove network",
            "1. Set current network",
            "2. Update network RPC URL",
            "3. Update network YAK Router Address",
            "<- Go back",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .items(&topics)
            .default(0)
            .interact_on_opt(&Term::stderr())?;

        match selection {
            Some(index) => match FromPrimitive::from_usize(index) {
                // Some(NetworkTopics::Add) => {
                //     Self::add_network();
                //     Terminal::render();
                // }
                // Some(NetworkTopics::Remove) => {
                //     Self::remove_network();
                //     Terminal::render();
                // }
                Some(NetworkTopics::Set) => {
                    Self::set_network();
                    Terminal::render();
                }
                Some(NetworkTopics::UpdateRPC) => {
                    Self::update_rpc();
                    Terminal::render();
                }
                Some(NetworkTopics::UpdateRouter) => {
                    Self::update_router();
                    Terminal::render();
                }
                Some(NetworkTopics::Back) => {
                    Terminal::render();
                }
                None => panic!("Error while selecting account screen topic"),
            },
            None => println!("You did not select anything"),
        }

        Ok(())
    }

    fn add_network() {
        unimplemented!();
    }

    fn remove_network() {
        unimplemented!();
    }

    fn set_network() {
        let items = Network::get_supported_networks();

        if items.is_empty() {
            println!("Empty list of supported networks");
            return;
        }

        let selection = Select::with_theme(&ColorfulTheme::default())
            .items(&items)
            .default(0)
            .interact_on_opt(&Term::stderr())
            .unwrap();

        if let Some(selected) = selection {
            let selected_network = items[selected].to_owned();
            Network::set_current_network(selected_network);
        } else {
            panic!("Network not selected");
        }
    }

    fn update_rpc() {
        let items = Network::get_supported_networks();

        if items.is_empty() {
            println!("Empty list of supported networks");
            return;
        }

        let selection = Select::with_theme(&ColorfulTheme::default())
            .items(&items)
            .with_prompt("Select chain to update RPC URL for")
            .default(0)
            .interact_on_opt(&Term::stderr())
            .unwrap();

        if let Some(selected) = selection {
            let selected_network = items[selected].to_owned();

            let rpc_url = Input::<String>::new()
                .with_prompt("New RPC URL")
                .interact_text()
                .unwrap();

            Network::update_rpc(selected_network.chain_id, rpc_url);
        } else {
            panic!("Network not selected");
        }
    }

    fn update_router() {
        let items = Network::get_supported_networks();

        if items.is_empty() {
            println!("Empty list of supported networks");
            return;
        }

        let selection = Select::with_theme(&ColorfulTheme::default())
            .items(&items)
            .with_prompt("Select chain to update YAK ROUTER for")
            .default(0)
            .interact_on_opt(&Term::stderr())
            .unwrap();

        if let Some(selected) = selection {
            let selected_network = items[selected].to_owned();

            let yak_router = Input::<String>::new()
                .with_prompt("New Yak Router address")
                .interact_text()
                .unwrap();

            Network::update_yak(selected_network.chain_id, yak_router);
        } else {
            panic!("Network not selected");
        }
    }
}

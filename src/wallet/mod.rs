use std::fs;
use std::path::Path;
use std::str;
use std::sync::Mutex;

use console::Term;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Select};
use dialoguer::{Input, Password};
use eth_keystore::{decrypt_key, encrypt_key};
use ethers::prelude::k256::ecdsa::SigningKey;
use ethers::prelude::k256::elliptic_curve::Error;
use ethers::{
    prelude::{
        k256::SecretKey,
        rand::{self, RngCore},
    },
    signers::{LocalWallet, Signer, Wallet},
    types::H160,
};
use lazy_static::lazy_static;

use storage::WalletStorage;

use crate::terminal::Terminal;

pub mod storage;

pub struct AccountWallet {
    wallet: Wallet<SigningKey>,
}

lazy_static! {
    pub static ref CURRENT_WALLET: Mutex<AccountWallet> =
        Mutex::new(AccountWallet::current_wallet_init());
}

trait Constants {
    const PATH_KEYS: &'static str;
}

impl Constants for AccountWallet {
    const PATH_KEYS: &'static str = "./keys";
}

impl AccountWallet {
    pub fn new(name: String, pk: String, password: String) -> Self {
        fs::create_dir_all(AccountWallet::PATH_KEYS).unwrap();

        let dir = Path::new("./keys");
        let mut rng = rand::thread_rng();

        let wallet = pk.parse::<LocalWallet>().unwrap();

        let address = wallet.address();
        println!("Name is: {}", name);
        println!("Address is: {}", address);

        let mut pk_bytes = pk.to_owned();

        unsafe {
            rng.fill_bytes(pk_bytes.as_bytes_mut());
        }

        encrypt_key(&dir, &mut rng, &pk, password, Some(&name)).unwrap();

        WalletStorage::save_wallet(&name, address);

        Self { wallet }
    }

    pub fn decrypt_wallet(name: String, password: String) -> Result<Self, ()> {
        let pk_path = format!("{}/{}", AccountWallet::PATH_KEYS, name);
        let pk_decrypted = decrypt_key(pk_path, password);

        match pk_decrypted {
            Ok(res) => {
                let pk = str::from_utf8(&res).unwrap();

                let wallet = pk.parse::<LocalWallet>().unwrap();

                Ok(Self { wallet })
            }
            Err(err) => {
                println!("Error, wrong password or account does not exist");

                if Confirm::new()
                    .with_prompt("Do you want to re-enter your password?")
                    .default(true)
                    .interact()
                    .unwrap()
                {
                    let password: String = Password::new()
                        .with_prompt("Current Wallet password")
                        .interact()
                        .unwrap();

                    Self::decrypt_wallet(name, password)
                } else {
                    Err(())
                }
            }
        }
    }

    pub fn current_wallet_init() -> Self {
        let current_wallet = WalletStorage::get_current_wallet();

        if current_wallet.is_none() {
            panic!("No wallet set");
        }

        let password: String = Password::new().with_prompt("Password").interact().unwrap();

        let account = AccountWallet::decrypt_wallet(current_wallet.unwrap().name, password);

        match account {
            Ok(acc) => acc,
            Err(err) => {
                panic!("{:?}", err);
            }
        }
    }

    pub fn wallet(&self) -> &LocalWallet {
        &self.wallet
    }

    pub fn print_wallets() {
        let wallets = WalletStorage::get_wallets();

        if wallets.is_empty() {
            return;
        }

        println!("Wallets:");

        for wallet in wallets {
            println!("Name: {} \t Address: {}", wallet.name, wallet.address);
        }
        println!();
    }

    pub fn set_wallet() {
        let items = WalletStorage::get_wallets();

        if items.is_empty() {
            println!("Empty list of accounts");
            return;
        }

        let selection = Select::with_theme(&ColorfulTheme::default())
            .items(&items)
            .default(0)
            .interact_on_opt(&Term::stderr())
            .unwrap();

        if let Some(selected) = selection {
            let selected_wallet = items[selected].to_owned();
            WalletStorage::set_current_wallet(&selected_wallet.name, selected_wallet.address);
        } else {
            panic!("Wallet not selected");
        }
    }

    pub fn remove_wallet() {
        let items = WalletStorage::get_wallets();

        if items.is_empty() {
            println!("Empty list of accounts");
            return;
        }

        let selection = Select::with_theme(&ColorfulTheme::default())
            .items(&items)
            .default(0)
            .interact_on_opt(&Term::stderr())
            .unwrap();

        if let Some(selected) = selection {
            let selected_wallet = items[selected].to_owned();
            WalletStorage::remove_wallet(selected_wallet);
        } else {
            panic!("Wallet not selected");
        }
    }
}

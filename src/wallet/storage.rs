use std::fmt;

use crate::db::DB;
use ethers::types::H160;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct WalletStorage {
    pub name: String,
    pub address: H160,
}

impl fmt::Display for WalletStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Name: {} \t Address: {}", self.name, self.address)
    }
}

impl WalletStorage {
    pub const DB_WALLETS_LIST: &'static str = "wallets";
    pub const DB_CURRENT_WALLET: &'static str = "current";

    // name: String, address: H160
    pub fn get_wallets() -> Vec<Self> {
        let db_instance = DB.lock().unwrap();

        let mut wallets: Vec<WalletStorage> = Vec::new();

        if !db_instance.lexists(WalletStorage::DB_WALLETS_LIST) {
            return wallets;
        }

        for wallet_item in db_instance.liter(WalletStorage::DB_WALLETS_LIST) {
            wallets.push(wallet_item.get_item::<WalletStorage>().unwrap());
        }

        wallets
    }

    pub fn get_current_wallet() -> Option<Self> {
        let db_instance = DB.lock().unwrap();

        let current_wallet = db_instance.get::<WalletStorage>(WalletStorage::DB_CURRENT_WALLET);

        current_wallet
    }

    pub fn set_current_wallet(name: &str, address: H160) {
        let mut db_instance = DB.lock().unwrap();

        db_instance
            .set(
                WalletStorage::DB_CURRENT_WALLET,
                &WalletStorage {
                    name: name.to_owned(),
                    address,
                },
            )
            .unwrap();
    }

    pub fn save_wallet(name: &str, address: H160) {
        let mut db_instance = DB.lock().unwrap();
        if !db_instance.lexists(WalletStorage::DB_WALLETS_LIST) {
            db_instance.lcreate(WalletStorage::DB_WALLETS_LIST).unwrap();
            db_instance
                .set(
                    WalletStorage::DB_CURRENT_WALLET,
                    &WalletStorage {
                        name: name.to_owned(),
                        address,
                    },
                )
                .unwrap();
        }

        db_instance.ladd(
            WalletStorage::DB_WALLETS_LIST,
            &WalletStorage {
                name: name.to_owned(),
                address,
            },
        );
    }

    pub fn remove_wallet(wallet: WalletStorage) {
        let mut db_instance = DB.lock().unwrap();

        let wallets_len = db_instance.llen(WalletStorage::DB_WALLETS_LIST);

        if wallets_len <= 1 {
            db_instance.lrem_list(WalletStorage::DB_WALLETS_LIST);
            db_instance.rem(WalletStorage::DB_CURRENT_WALLET);
            return;
        }

        // check if wallet is current wallet => if yes delete current wallet key
        let current_wallet = db_instance.get::<WalletStorage>(WalletStorage::DB_CURRENT_WALLET);

        if let Some(current_wallet) = current_wallet {
            db_instance.rem(WalletStorage::DB_CURRENT_WALLET);
        }

        // remove wallet from list
        db_instance.lrem_value(WalletStorage::DB_WALLETS_LIST, &wallet);
    }
}

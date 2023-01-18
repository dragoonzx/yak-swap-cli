use std::{collections::HashMap, fmt};

use crate::db::DB;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Network {
    pub chain_id: u32,
    pub name: String,
    pub short_name: String,
    pub explorer_url: String,
    pub rpc_url: String,
    pub currency_name: String,
    pub currency_symbol: String,
    pub currency_decimals: u8,
    pub is_testnet: bool,
    pub yak_router: Option<String>,
}

impl std::fmt::Display for Network {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (RPC URL: {}, Chain Id: {})",
            self.name, self.rpc_url, self.chain_id
        )
    }
}

impl Network {
    pub const DB_CURRENT_NETWORK: &'static str = "current-network";
    pub const DB_CUSTOM_RPC: &'static str = "current-rpc";
    pub const DB_CUSTOM_YAK_ROUTER: &'static str = "current-yak";

    pub fn get_current_network() -> Self {
        let mut db_instance = DB.try_lock().unwrap();

        let current_network = db_instance.get::<Self>(Self::DB_CURRENT_NETWORK);

        if let Some(mut current_network) = current_network {
            let custom_rpcs = db_instance.get::<HashMap<u32, String>>(Self::DB_CUSTOM_RPC);
            let custom_yaks = db_instance.get::<HashMap<u32, String>>(Self::DB_CUSTOM_YAK_ROUTER);

            if let Some(custom_rpcs) = custom_rpcs {
                let custom_rpc = custom_rpcs.get(&current_network.chain_id);

                if let Some(custom_rpc) = custom_rpc {
                    current_network.set_custom_rpc(custom_rpc.to_owned());
                }
            }

            if let Some(custom_yaks) = custom_yaks {
                let custom_yak_router = custom_yaks.get(&current_network.chain_id);

                if let Some(custom_yak_router) = custom_yak_router {
                    current_network.set_custom_router(custom_yak_router.to_owned())
                }
            }

            current_network
        } else {
            let default_network = Self::get_supported_networks()[0].to_owned();

            db_instance
                .set(Self::DB_CURRENT_NETWORK, &default_network)
                .unwrap();

            default_network
        }
    }

    pub fn set_current_network(network: Network) {
        let mut db_instance = DB.try_lock().unwrap();
        db_instance.set(Self::DB_CURRENT_NETWORK, &network).unwrap();
    }

    fn set_custom_rpc(&mut self, rpc: String) {
        self.rpc_url = rpc;
    }

    fn set_custom_router(&mut self, router: String) {
        self.yak_router = Some(router);
    }

    pub fn update_rpc(chain_id: u32, new_rpc: String) {
        let mut db_instance = DB.try_lock().unwrap();

        let custom_rpcs = db_instance.get::<HashMap<u32, String>>(Self::DB_CUSTOM_RPC);

        if let Some(mut custom_rpcs) = custom_rpcs {
            custom_rpcs.insert(chain_id, new_rpc);
            db_instance.set(Self::DB_CUSTOM_RPC, &custom_rpcs).unwrap();
        } else {
            let custom_rpcs = HashMap::from([(chain_id, new_rpc)]);
            db_instance.set(Self::DB_CUSTOM_RPC, &custom_rpcs).unwrap();
        }
    }

    pub fn update_yak(chain_id: u32, new_yak: String) {
        let mut db_instance = DB.try_lock().unwrap();

        let custom_yaks = db_instance.get::<HashMap<u32, String>>(Self::DB_CUSTOM_YAK_ROUTER);

        if let Some(mut custom_yaks) = custom_yaks {
            custom_yaks.insert(chain_id, new_yak);
            db_instance
                .set(Self::DB_CUSTOM_YAK_ROUTER, &custom_yaks)
                .unwrap();
        } else {
            let custom_yaks = HashMap::from([(chain_id, new_yak)]);
            db_instance
                .set(Self::DB_CUSTOM_YAK_ROUTER, &custom_yaks)
                .unwrap();
        }
    }

    pub fn get_supported_networks() -> [Self; 10] {
        [
            // Avalanche
            Network {
                chain_id: 43114,
                name: "Avalanche C-Chain".to_owned(),
                short_name: "Avalanche".to_owned(),
                explorer_url: "https://snowtrace.io".to_owned(),
                rpc_url: "https://api.avax.network/ext/bc/C/rpc".to_owned(),
                currency_name: "Avalanche".to_owned(),
                currency_symbol: "AVAX".to_owned(),
                currency_decimals: 18,
                is_testnet: false,
                yak_router: Some("0xC4729E56b831d74bBc18797e0e17A295fA77488c".to_owned()),
            },
            Network {
                chain_id: 43113,
                name: "Avalanche Fuji Testnet".to_owned(),
                short_name: "Avalanche Fuji".to_owned(),
                explorer_url: "https://testnet.snowtrace.io".to_owned(),
                rpc_url: "https://api.avax-test.network/ext/bc/C/rpc".to_owned(),
                currency_name: "Avalanche".to_owned(),
                currency_symbol: "AVAX".to_owned(),
                currency_decimals: 18,
                is_testnet: true,
                yak_router: None,
            },
            // Dogechain
            Network {
                chain_id: 2000,
                name: "Dogechain Mainnet".to_owned(),
                short_name: "Dogechain".to_owned(),
                explorer_url: "https://explorer.dogechain.dog".to_owned(),
                rpc_url: "https://rpc-sg.dogechain.dog".to_owned(),
                currency_name: "Dogecoin".to_owned(),
                currency_symbol: "DOGE".to_owned(),
                currency_decimals: 18,
                is_testnet: false,
                yak_router: None,
            },
            Network {
                chain_id: 568,
                name: "Dogechain Testnet".to_owned(),
                short_name: "Dogechain Testnet".to_owned(),
                explorer_url: "https://explorer-testnet.dogechain.dog".to_owned(),
                rpc_url: "https://rpc-testnet.dogechain.dog".to_owned(),
                currency_name: "Dogecoin".to_owned(),
                currency_symbol: "DOGE".to_owned(),
                currency_decimals: 18,
                is_testnet: true,
                yak_router: None,
            },
            // Optimism
            Network {
                chain_id: 10,
                name: "Optimism Mainnet".to_owned(),
                short_name: "Optimism".to_owned(),
                explorer_url: "https://optimistic.etherscan.io".to_owned(),
                rpc_url: "https://mainnet.optimism.io".to_owned(),
                currency_name: "OP Ethereum".to_owned(),
                currency_symbol: "opETH".to_owned(),
                currency_decimals: 18,
                is_testnet: false,
                yak_router: None,
            },
            Network {
                chain_id: 69,
                name: "Optimism Testnet".to_owned(),
                short_name: "Optimism Testnet".to_owned(),
                explorer_url: "https://kovan-optimistic.etherscan.io".to_owned(),
                rpc_url: "https://kovan.optimism.io".to_owned(),
                currency_name: "OP Ethereum".to_owned(),
                currency_symbol: "opETH".to_owned(),
                currency_decimals: 18,
                is_testnet: true,
                yak_router: None,
            },
            // Arbitrum
            Network {
                chain_id: 42161,
                name: "Arbitrum Mainnet".to_owned(),
                short_name: "Arbitrum".to_owned(),
                explorer_url: "https://arbiscan.io".to_owned(),
                rpc_url: "https://arb1.arbitrum.io/rpc".to_owned(),
                currency_name: "ETH".to_owned(),
                currency_symbol: "ETH".to_owned(),
                currency_decimals: 18,
                is_testnet: false,
                yak_router: None,
            },
            Network {
                chain_id: 421613,
                name: "Arbitrum Goerli Testnet".to_owned(),
                short_name: "Optimism Testnet".to_owned(),
                explorer_url: "https://goerli.arbiscan.io/".to_owned(),
                rpc_url: "https://goerli-rollup.arbitrum.io/rpc".to_owned(),
                currency_name: "ETH".to_owned(),
                currency_symbol: "ETH".to_owned(),
                currency_decimals: 18,
                is_testnet: true,
                yak_router: None,
            },
            // Aurora
            Network {
                chain_id: 1313161554,
                name: "Aurora Mainnet".to_owned(),
                short_name: "Aurora".to_owned(),
                explorer_url: "https://aurorascan.dev".to_owned(),
                rpc_url: "https://mainnet.aurora.dev".to_owned(),
                currency_name: "ETH".to_owned(),
                currency_symbol: "ETH".to_owned(),
                currency_decimals: 18,
                is_testnet: false,
                yak_router: None,
            },
            Network {
                chain_id: 1313161555,
                name: "Aurora Testnet".to_owned(),
                short_name: "Aurora Testnet".to_owned(),
                explorer_url: "https://testnet.aurorascan.dev".to_owned(),
                rpc_url: "https://testnet.aurora.dev".to_owned(),
                currency_name: "ETH".to_owned(),
                currency_symbol: "ETH".to_owned(),
                currency_decimals: 18,
                is_testnet: true,
                yak_router: None,
            },
        ]
    }
}

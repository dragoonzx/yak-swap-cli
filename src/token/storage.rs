use std::fmt;

use crate::db::DB;
use ethers::types::H160;

use crate::token::Token;

pub struct TokenStorage {}

impl TokenStorage {
    pub const DB_TOKENS_LIST: &'static str = "tokens";

    pub fn get_local_tokens() -> Vec<Token> {
        let db_instance = DB.lock().unwrap();

        let mut tokens: Vec<Token> = Vec::new();

        if !db_instance.lexists(TokenStorage::DB_TOKENS_LIST) {
            return tokens;
        }

        for token in db_instance.liter(TokenStorage::DB_TOKENS_LIST) {
            tokens.push(token.get_item::<Token>().unwrap());
        }

        tokens
    }

    pub fn save_token(
        address: String,
        chain_id: u32,
        decimals: u32,
        name: String,
        symbol: String,
    ) -> Token {
        let mut db_instance = DB.lock().unwrap();
        if !db_instance.lexists(TokenStorage::DB_TOKENS_LIST) {
            db_instance.lcreate(TokenStorage::DB_TOKENS_LIST).unwrap();
        }

        let token = Token {
            address: address.to_owned(),
            chainId: Some(chain_id),
            decimals,
            logoURI: "".to_owned(),
            name: name.to_owned(),
            symbol: symbol.to_owned(),
        };

        db_instance.ladd(TokenStorage::DB_TOKENS_LIST, &token);

        token
    }
}

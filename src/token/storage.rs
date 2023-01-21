use crate::db::DB;

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
            address,
            chain_id: Some(chain_id),
            decimals,
            name,
            symbol,
        };

        db_instance.ladd(TokenStorage::DB_TOKENS_LIST, &token);

        token
    }

    pub fn remove_token(token: Token) {
        let mut db_instance = DB.lock().unwrap();

        let tokens_len = db_instance.llen(Self::DB_TOKENS_LIST);

        if tokens_len <= 1 {
            db_instance.lrem_list(Self::DB_TOKENS_LIST).unwrap();
            return;
        }

        // remove wallet from list
        db_instance
            .lrem_value(Self::DB_TOKENS_LIST, &token)
            .unwrap();
    }
}

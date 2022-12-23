use std::sync::Mutex;

use lazy_static::lazy_static;
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};

const DB_PATH: &'static str = "./cli.db";

lazy_static! {
  pub static ref DB: Mutex<PickleDb> = Mutex::new(init_db(DB_PATH));
}

fn init_db(path: &str) -> PickleDb {
  let db = PickleDb::load(
    path,
    PickleDbDumpPolicy::AutoDump,
    SerializationMethod::Bin,
  );

  match db {
      Ok(db) => {
        db
      },
      Err(_) => {
        let new_db = PickleDb::new(
            path,
            PickleDbDumpPolicy::AutoDump,
            SerializationMethod::Json,
        );
        new_db
      }
  }
}
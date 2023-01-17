use crate::db::DB;

pub struct Settings {}

impl Settings {
    pub const DB_MAX_STEPS: &'static str = "max_steps";
    pub const DB_SLIPPAGE: &'static str = "slippage";
    pub const DB_COMPARE_WITH_EXTERNAL: &'static str = "is_external_allowed";

    pub fn set_max_steps(steps: i32) {
        let mut db_instance = DB.lock().unwrap();

        db_instance.set(Settings::DB_MAX_STEPS, &steps).unwrap();
    }

    pub fn get_max_steps() -> i32 {
        let db_instance = DB.lock().unwrap();

        let max_steps = db_instance.get::<i32>(Settings::DB_MAX_STEPS);

        max_steps.unwrap_or(3)
    }

    pub fn set_slippage(slippage: u32) {
        let mut db_instance = DB.lock().unwrap();

        db_instance.set(Settings::DB_SLIPPAGE, &slippage).unwrap();
    }

    // @dev slippage in u32 format, e.g.: 5 = 0.5%
    pub fn get_slippage() -> u32 {
        let db_instance = DB.lock().unwrap();

        let slippage = db_instance.get::<u32>(Settings::DB_SLIPPAGE);

        slippage.unwrap_or(5)
    }

    pub fn is_external_allowed() -> bool {
        let db_instance = DB.lock().unwrap();

        let is_external_allowed = db_instance.get::<bool>(Settings::DB_COMPARE_WITH_EXTERNAL);

        is_external_allowed.unwrap_or(false)
    }

    pub fn set_is_external_allowed(is_allowed: bool) {
        let mut db_instance = DB.lock().unwrap();

        db_instance
            .set(Settings::DB_COMPARE_WITH_EXTERNAL, &is_allowed)
            .unwrap();
    }
}

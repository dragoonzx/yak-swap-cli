use std::fmt;

use ethers::types::H160;

pub struct Adapter {
    pub address: H160,
    pub name: String,
}

impl fmt::Display for Adapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.address)
    }
}

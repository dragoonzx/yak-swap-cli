use ethers::prelude::abigen;

abigen!(YakRouter, "./abi/YakRouter.json");
abigen!(YakAdapter, "./abi/YakAdapter.json");
abigen!(ERC20, "./abi/ERC20ABI.json");
abigen!(IWETH, "./abi/IWETH.json");

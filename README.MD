Yak Swap CLI (Rust version)

Features:

- Account Management:

  - Store private key locally encrypted with Scrypt
  - Set default (current) account
  - Support multiple accounts

- Network Management:

  - Set current network for querying and swapping from supported list
  - Set custom RPC for network
  - Set custom Yak Router contract address

- Query:

  - List of adapters
  - Best path
  - Single adapter

- Swap:
  - Query & Swap between supported tokens
  - Wrap/Unwrap native token
  - Add custom tokens

Dev:
cargo run dev
cargo build --release

pickledb is used for local storage

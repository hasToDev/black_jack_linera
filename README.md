## Install Rust, Linera SDK, and Linera Service

The first step you have to do is install Rust. You can follow the steps to install Rust in the following article:

- [Install Rust - Rust Programming Language](https://www.rust-lang.org/tools/install)
- [Walkthrough: Installing Rust on Windows](https://www.alpharithms.com/installing-rust-on-windows-403718/)
- [How To Install Rust on Ubuntu 20.04](https://www.digitalocean.com/community/tutorials/install-rust-on-ubuntu-linux)

Next we install Linera SDK and Linera Service:

```
cargo install --locked linera-storage-service@0.13.1
cargo install --locked linera-sdk@0.13.1
cargo install --locked linera-service@0.13.1
```

Confirm that both Rust and Linera CLI are installed by running `rustc --version` and `linera --version`.

You should be aware that currently we can't install Linera Service on Windows.

## Clone, Build, and Deploy on Local Network

Clone the repository : `git clone https://github.com/hasToDev/black_jack_linera.git`

To build and deploy, run the following command in sequence, in a single terminal tab, without using a shell script:

```shell 
source /dev/stdin <<<"$(linera net helper 2>/dev/null)"

linera_spawn_and_read_wallet_variables linera net up --extra-wallets 1 --initial-amount 1000000000 --validators 2 --shards 2 --other-initial-chains 2

./auto_open_multi_chain.sh

linera -w1 service --port 8081
```

After deployment, you can access the Linera GraphQL service through the following URL: http://localhost:8081/

## Linera Version

This repository is tested with Linera Archimedes Testnet. 
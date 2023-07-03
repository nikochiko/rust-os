# Rust OS

My follow-along for the blog series [*Writing an OS in Rust*](https://os.phil-opp.com/).

Notes are currently somewhere else. I'll consolidate them here or on my blog later.

## Fresh setup instructions

**Prerequisites:** Install Rust, Cargo, and rustup. (Installation: [https://rustup.rs/](https://rustup.rs/)

1. Enable Rust nightly version in current directory (`rustup override set nightly`)
2. Install qemu (`sudo apt-get install qemu-system`)
3. Install bootimage crate (`cargo install bootimage`)

Running `cargo build` will do the rest.

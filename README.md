# ProppedBlockchain

This is the code base for simulating our ProppedBlockchain protocol on the Emulab testbed.

## HOW TO RUN

> `RUST_LOG=trace ./target/debug/propped_blockchain &> log.txt`

## INSTALLING ON EMULAB

For EACH emulab machine:

1. Log into it
2. Run `sudo su -` (you should now be at /root, also make sure you're in bash)
3. Install rust `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
4. Put the binaries into our current path `source /root/.cargo/env`
5. Move to area of file system maintained by disk snapshots `cd /local/`
5. Clone our github repo `git clone https://github.com/judgegrubb/ProppedBlockchain.git`
6. `cd ProppedBlockchain/`
7. `cargo build`
8. `RUST_LOG=trace ./target/debug/propped-blockchain &> log.txt &`
9. `tail -f log.txt`


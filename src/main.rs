use std::time::Instant;

use rust_blockchain::blockchain::BlockChain;

fn main() {
    println!("Hello, world!");

    let difficulty = 1;

    let mut blockchain = BlockChain::new(difficulty);
    let time = Instant::now();
    BlockChain::add_block(&mut blockchain);
    println!(
        "\t ⛏️⛏️⛏️    | Mined |   ⛏️⛏️⛏️\n\n\tElapsed: {:?}\n\n{:#?}",
        time.elapsed(),
        blockchain
    );
}

use std::time::Instant;

use rust_blockchain::blockchain::BlockChain;

fn main() {
    println!("Hello, world!");

    let difficulty = 1;

    let mut blockchain = BlockChain::new(difficulty);
    let instant = Instant::now();
    BlockChain::add_block(&mut blockchain, "T".to_string(), Instant::now());
    BlockChain::add_block(&mut blockchain, "a".to_string(), Instant::now());
    BlockChain::add_block(&mut blockchain, "h".to_string(), Instant::now());
    BlockChain::add_block(&mut blockchain, "i".to_string(), Instant::now());
    BlockChain::add_block(&mut blockchain, "n".to_string(), Instant::now());
    BlockChain::add_block(&mut blockchain, "l".to_string(), Instant::now());
    BlockChain::add_block(&mut blockchain, "i".to_string(), Instant::now());
    println!(
        "\t ⛏️⛏️⛏️    | Mined |   ⛏️⛏️⛏️\n\n\tElapsed: {:?}\n\n{:#?}",
        instant.elapsed(),
        blockchain
    );
}

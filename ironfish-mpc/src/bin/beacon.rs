extern crate pairing;
extern crate rand;
extern crate rand_chacha;

use blake2::{Blake2b512, Digest};
use std::convert::TryInto;
use std::fs::File;
use std::io::{BufReader, BufWriter};

fn decode_hex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

fn main() {
    let current_params = File::open("params").expect("couldn't open `./params`");
    let mut current_params = BufReader::with_capacity(1024 * 1024, current_params);

    let new_params = File::create("new_params").expect("couldn't create `./new_params`");
    let mut new_params = BufWriter::with_capacity(1024 * 1024, new_params);

    let mut sapling_spend = ironfish_phase2::MPCParameters::read(&mut current_params, false)
        .expect("couldn't deserialize Sapling Spend params");

    let mut sapling_output = ironfish_phase2::MPCParameters::read(&mut current_params, false)
        .expect("couldn't deserialize Sapling Output params");

    let mut sapling_mint = ironfish_phase2::MPCParameters::read(&mut current_params, false)
        .expect("couldn't deserialize Sapling Mint params");

    // Create an RNG based on the outcome of the random beacon
    let rng = &mut {
        use rand::SeedableRng;
        use rand_chacha::ChaChaRng;

        // Place beacon value here. The value will be the randomness generated by The League of Entropy's drand network
        // (network chain hash: 8990e7a9aaed2ffed73dbd7092123d6f289930540d7651336225dc172e51b2ce) in round #2759370.
        let beacon_value: [u8; 32] = decode_hex("").as_slice().try_into().unwrap();

        print!("Final result of beacon: ");
        for b in beacon_value.iter() {
            print!("{:02x}", b);
        }
        println!();

        ChaChaRng::from_seed(beacon_value)
    };

    let h1 = sapling_spend.contribute(rng);
    let h2 = sapling_output.contribute(rng);
    let h3 = sapling_mint.contribute(rng);

    sapling_spend
        .write(&mut new_params)
        .expect("couldn't write new Sapling Spend params");
    sapling_output
        .write(&mut new_params)
        .expect("couldn't write new Sapling Output params");
    sapling_mint
        .write(&mut new_params)
        .expect("couldn't write new Sapling Mint params");

    let mut h = Blake2b512::new();
    h.update(h1);
    h.update(h2);
    h.update(h3);
    let h = h.finalize();

    print!(
        "Done!\n\n\
              Your contribution has been written to `./new_params`\n\n\
              The contribution you made is bound to the following hash:\n"
    );

    for line in h.chunks(16) {
        print!("\t");
        for section in line.chunks(4) {
            for b in section {
                print!("{:02x}", b);
            }
            print!(" ");
        }
        println!();
    }

    println!("\n");
}

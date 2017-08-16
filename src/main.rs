#[macro_use]
extern crate serde;
extern crate serde_json;

use std::process::Command;

mod error;

const TESTNET: bool = true;

trait BitcoinCommand {
    const COMMAND: &'static str;
    type OutputFormat: for<'de> serde::Deserialize<'de>;
}

enum GetNewAddress {}

impl BitcoinCommand for GetNewAddress {
    const COMMAND: &'static str = "getnewaddress";
    type OutputFormat = String;
}

enum AddWitnessAddress {}

impl BitcoinCommand for AddWitnessAddress {
    const COMMAND: &'static str = "addwitnessaddress";
    type OutputFormat = String;
}

fn execute<X: BitcoinCommand>(testnet: bool, input: &[&str]) -> error::Result<X::OutputFormat> {
    let mut command = Command::new("bitcoin-cli");

    if testnet {
        command.arg("-testnet");
    }

    let raw_output = command.arg(X::COMMAND).args(input).output()?;

    let output: X::OutputFormat = serde_json::from_slice(&raw_output.stdout)?;

    Ok(output)
}

fn main() {
    let pay_to_public_key_hash_address = execute::<GetNewAddress>(TESTNET, &[]).unwrap();
    let segregated_witness_pay_to_script_hash_address =
        execute::<AddWitnessAddress>(TESTNET, &[&pay_to_public_key_hash_address]).unwrap();

    println!("{}", segregated_witness_pay_to_script_hash_address);
}

#[macro_use]
extern crate serde;
extern crate serde_json;

use std::process::Command;

mod error;

trait BitcoinCommand {
    const COMMAND: &'static str;
    type InputFormat;
    type OutputFormat: for<'de> serde::Deserialize<'de>;
}

#[derive(Debug, Copy, Clone)]
struct GetMemPoolInfoInput {}

#[derive(Debug, Copy, Clone, Deserialize)]
struct GetMemPoolInfoOutput {
    size: u32,
    bytes: u32,
    usage: u32,
    maxmempool: u32,
    mempoolminfee: u32,
}

enum GetMemPoolInfo {}

impl BitcoinCommand for GetMemPoolInfo {
    const COMMAND: &'static str = "getmempoolinfo";
    type InputFormat = GetMemPoolInfoInput;
    type OutputFormat = GetMemPoolInfoOutput;
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
    let y = execute::<GetMemPoolInfo>(true, &[]).unwrap();

    println!("{:?}", y);
}

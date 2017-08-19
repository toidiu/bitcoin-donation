// bitcoin-donation - Generate a Bitcoin address for donations.
// Copyright (C) 2017 Cooper Paul EdenDay
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

#![forbid(unsafe_code)]
#![deny(warnings)]

#[macro_use]
extern crate serde;
extern crate serde_json;

extern crate futures;
extern crate hyper;
extern crate tokio_core;

use std::env::args;
use std::io::{self, stderr, stdin, BufRead, Write};
use hyper::header::Basic;
use tokio_core::reactor::Core;
use hyper::Client;

mod rpc_run;

use rpc_run::execute;
use rpc_run::commands::*;

fn get_password() -> io::Result<String> {
    let stdin = stdin();
    let stderr = stderr();
    let mut stdin_lock = stdin.lock();
    let mut stderr_lock = stderr.lock();

    stderr_lock.write_all("Input RPC password: ".as_bytes())?;
    stderr_lock.flush()?;

    let mut password = String::new();
    stdin_lock.read_line(&mut password)?;

    password = password.trim().to_owned();

    Ok(password)
}

fn main() {
    let mut core = Core::new().expect("Could not initialize tokio");
    let client = Client::new(&core.handle());

    if let Some(uri_raw) = args().nth(1) {
        if let Ok(uri) = uri_raw.parse() {
            let credentials: Basic = Basic {
                username: String::new(),
                password: Some(
                    get_password().expect("Failed to read RPC password from STDIN"),
                ),
            };

            let pay_to_public_key_hash_address =
                execute::<GetNewAddress>(&mut core, &client, &uri, &credentials, &[]).unwrap();
            let segregated_witness_pay_to_script_hash_address = execute::<AddWitnessAddress>(
                &mut core,
                &client,
                &uri,
                &credentials,
                &[&pay_to_public_key_hash_address],
            ).unwrap();

            // Assert some things about the newly generated address.
            {
                let address_info = execute::<ValidateAddress>(
                    &mut core,
                    &client,
                    &uri,
                    &credentials,
                    &[&segregated_witness_pay_to_script_hash_address],
                ).unwrap();

                assert_eq!(address_info.isvalid, true);
                assert_eq!(address_info.ismine, Some(true));
                assert_eq!(address_info.iswatchonly, Some(false));
            }

            println!("{}", segregated_witness_pay_to_script_hash_address);
        } else {
            eprintln!("`bitcoind` RPC URL '{}' could not be parsed.", &uri_raw);
        }
    } else {
        eprintln!(
            "Command line argument '`bitcoind` RPC URL' required.\n\
             Example: `http://localhost:18332/` for testnet on localhost."
        );
    }
}

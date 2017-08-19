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
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

#![forbid(unsafe_code)]
#![deny(warnings)]

#[macro_use]
extern crate serde;
extern crate serde_json;

extern crate futures;
extern crate hyper;
extern crate tokio_core;

use std::process::exit;
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

enum Error {
    Cli,
    Uri(String),
    Rpc(rpc_run::Error),
}

impl From<rpc_run::Error> for Error {
    fn from(error: rpc_run::Error) -> Self {
        Error::Rpc(error)
    }
}

fn main() {
    if let Err(error) = real_main() {
        match error {
            Error::Cli => eprintln!(
                "Command line argument RPC URL required.\n\
                 Example (testnet on localhost): `bitcoin-donation http://localhost:18332/`."
            ),
            Error::Uri(error) => eprintln!("RPC URL '{}' could not be parsed.", error),
            Error::Rpc(rpc_run::Error::Http(error)) => eprintln!(
                "Fatal error: \
                 HTTP error: '{}'.",
                error
            ),
            Error::Rpc(rpc_run::Error::Auth) => eprintln!(
                "Fatal error: \
                 authentication failure."
            ),
            Error::Rpc(rpc_run::Error::Json(error)) => eprintln!(
                "Fatal error: \
                 json error: '{}'.",
                error
            ),
            Error::Rpc(rpc_run::Error::Rpc(error)) => eprintln!(
                "Fatal error: \
                 RPC error: '{}'.",
                error.message
            ),
        }
        exit(1);
    }
}

fn real_main() -> Result<(), Error> {
    let mut core = Core::new().expect("Could not initialize tokio core");
    let client = Client::new(&core.handle());

    let uri_raw = args().nth(1).ok_or(Error::Cli)?;
    let uri = uri_raw.parse().map_err(|_| Error::Uri(uri_raw))?;

    let credentials: Basic = Basic {
        username: String::new(),
        password: Some(
            get_password().expect("Failed to read RPC password from `stdin`"),
        ),
    };

    let pay_to_public_key_hash_address =
        execute::<GetNewAddress>(&mut core, &client, &uri, &credentials, &[])?;
    let segregated_witness_pay_to_script_hash_address = execute::<AddWitnessAddress>(
        &mut core,
        &client,
        &uri,
        &credentials,
        &[&pay_to_public_key_hash_address],
    )?;

    // Assert some things about the newly generated address.
    {
        let address_info = execute::<ValidateAddress>(
            &mut core,
            &client,
            &uri,
            &credentials,
            &[&segregated_witness_pay_to_script_hash_address],
        )?;

        assert_eq!(address_info.isvalid, true);
        assert_eq!(address_info.ismine, Some(true));
        assert_eq!(address_info.iswatchonly, Some(false));
    }

    println!("{}", segregated_witness_pay_to_script_hash_address);

    Ok(())
}

#[macro_use]
extern crate serde;
extern crate serde_json;

extern crate futures;
extern crate hyper;
extern crate tokio_core;

use std::env::args;
use std::io::{self, stderr, stdin, BufRead, Write};
use futures::{Future, Stream};
use hyper::{Body, Chunk, Client, Method, Request, StatusCode, Uri};
use hyper::header::{Authorization, Basic, ContentLength, ContentType};
use tokio_core::reactor::Core;
use hyper::client::HttpConnector;

mod error;

trait BitcoinCommand {
    const COMMAND: &'static str;
    type OutputFormat: for<'de> serde::Deserialize<'de>;
}

#[derive(Debug)]
enum GetNewAddress {}

impl BitcoinCommand for GetNewAddress {
    const COMMAND: &'static str = "getnewaddress";
    type OutputFormat = String;
}

#[derive(Debug)]
enum AddWitnessAddress {}

impl BitcoinCommand for AddWitnessAddress {
    const COMMAND: &'static str = "addwitnessaddress";
    type OutputFormat = String;
}

#[derive(Debug, Clone, Serialize)]
struct RpcInput<'a> {
    jsonrpc: f32,
    id: Option<&'a str>,
    method: &'a str,
    params: &'a [&'a str],
}

#[derive(Debug, Clone, Deserialize)]
pub struct RpcError {
    code: u32,
    message: String,
    data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct RpcOutput<T> {
    result: Option<T>,
    error: Option<RpcError>,
    id: Option<String>,
}

fn execute<X: BitcoinCommand>(
    core: &mut Core,
    client: &Client<HttpConnector, Body>,
    server: &Uri,
    credentials: &Basic,
    params: &[&str],
) -> error::Result<X::OutputFormat> {
    let mut request = Request::new(Method::Post, server.clone());
    request.headers_mut().set(ContentType::json());

    request
        .headers_mut()
        .set(Authorization(credentials.clone()));

    let input = RpcInput {
        jsonrpc: 2.0,
        id: None,
        method: X::COMMAND,
        params,
    };

    let encoded_input = serde_json::to_vec(&input)?;

    request
        .headers_mut()
        .set(ContentLength(encoded_input.len() as u64));
    request.set_body(encoded_input);

    let check_status = client.request(request).map(
        |response| if response.status() == StatusCode::Ok {
            Ok(response.body().concat2())
        } else {
            Err(error::Error::Http(hyper::Error::Status))
        },
    );

    let decode_body = core.run(check_status)??.map(|body: Chunk| {
        let x: RpcOutput<X::OutputFormat> = serde_json::from_slice(&body)?;

        if let Some(output) = x.result {
            Ok(output)
        } else {
            Err(error::Error::Rpc(
                x.error
                    .expect("`error` should be present if `result` is not"),
            ))
        }
    });

    let output: error::Result<X::OutputFormat> = core.run(decode_body)?;

    output
}

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

            println!("{}", segregated_witness_pay_to_script_hash_address);
        } else {
            eprintln!("`bitcoind` RPC URL '{}' could not be parsed.", &uri_raw);
        }
    } else {
        eprintln!("Command line argument '`bitcoind` RPC URL' required.");
    }
}

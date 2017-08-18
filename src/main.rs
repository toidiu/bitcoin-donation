#[macro_use]
extern crate serde;
extern crate serde_json;

extern crate futures;
extern crate hyper;
extern crate tokio_core;

use std::io::{self, Write};
use futures::{Future, Stream};
use hyper::{Body, Client, Method, Request, StatusCode, Uri};
use hyper::header::{Authorization, Basic, ContentLength, ContentType};
use tokio_core::reactor::Core;

use hyper::client::HttpConnector;

mod error;

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

#[derive(Debug, Copy, Clone, Serialize)]
struct RpcInput<'a> {
    jsonrpc: f32,
    id: Option<&'a str>,
    method: &'a str,
    params: &'a [&'a str],
}

fn execute<X: BitcoinCommand>(
    core: &mut Core,
    client: &Client<HttpConnector, Body>,
    server: Uri,
    credentials: Basic,
    params: &[&str],
) -> error::Result<X::OutputFormat> {
    let mut request = Request::new(Method::Post, server);
    request.headers_mut().set(ContentType::plaintext());

    request.headers_mut().set(Authorization(credentials));

    let input = RpcInput {
        jsonrpc: 2.0,
        id: None,
        method: X::COMMAND,
        params,
    };

    let encoded_input = serde_json::to_vec(&input).unwrap();

    request
        .headers_mut()
        .set(ContentLength(encoded_input.len() as u64));
    request.set_body(encoded_input);

    let work = client.request(request).map(|res| {
        println!("Response: {}", res.status());
    });

    core.run(work)?;

    Err(error::Error::Http(hyper::Error::Status))
}

fn main() {
    let mut core = Core::new().expect("Could not initialize tokio");
    let client = Client::new(&core.handle());

    let uri: Uri = "http://127.0.0.1:18332/".parse().unwrap();
    let credentials: Basic = Basic {
        username: String::new(),
        password: Some(String::from("ncMGIndJmnSo9YUd11iT")),
    };

    let pay_to_public_key_hash_address =
        execute::<GetNewAddress>(&mut core, &client, uri.clone(), credentials.clone(), &[])
            .unwrap();
    let segregated_witness_pay_to_script_hash_address = execute::<AddWitnessAddress>(
        &mut core,
        &client,
        uri,
        credentials,
        &[&pay_to_public_key_hash_address],
    ).unwrap();

    println!("{}", segregated_witness_pay_to_script_hash_address);
}

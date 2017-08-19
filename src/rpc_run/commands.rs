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

use super::BitcoinCommand;

pub enum GetNewAddress {}

impl BitcoinCommand for GetNewAddress {
    const COMMAND: &'static str = "getnewaddress";
    type OutputFormat = String;
}

pub enum AddWitnessAddress {}

impl BitcoinCommand for AddWitnessAddress {
    const COMMAND: &'static str = "addwitnessaddress";
    type OutputFormat = String;
}

// Docs borked, investigate.
#[allow(non_snake_case)]
#[derive(Debug, Clone, Deserialize)]
pub struct ValidateAddressOutput {
    pub isvalid: bool,
    pub address: Option<String>,
    pub scriptPubKey: Option<String>,
    pub ismine: Option<bool>,
    pub iswatchonly: Option<bool>,
    pub isscript: Option<bool>,
    pub pubkey: Option<String>,
    pub iscompressed: Option<bool>,
    pub account: Option<String>,
    pub timestamp: Option<i64>,
    pub hdkeypath: Option<String>,
    pub hdmasterkeyid: Option<String>,
}

pub enum ValidateAddress {}

impl BitcoinCommand for ValidateAddress {
    const COMMAND: &'static str = "validateaddress";
    type OutputFormat = ValidateAddressOutput;
}

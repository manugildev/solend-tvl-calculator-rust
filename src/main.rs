#![allow(dead_code)]
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;
use std::vec;

use lazy_static::lazy_static;

use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcProgramAccountsConfig;
use solana_client::rpc_config::RpcAccountInfoConfig;
use solana_client::rpc_filter::MemcmpEncoding;
use solana_client::rpc_filter::MemcmpEncodedBytes;
use solana_client::rpc_filter::RpcFilterType;
use solana_client::rpc_filter::Memcmp;
use solana_account_decoder::UiAccountEncoding;

use solana_sdk::native_token::lamports_to_sol;
use solana_sdk::account::Account;

use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;

use spl_token_lending::state::LendingMarket;
use spl_token_lending::state::Reserve;
use spl_token_lending::state::Obligation;

const RPC_URL : &str = "https://api.mainnet-beta.solana.com";

const PROGRAM_VERSION: u8 = 1;
const OBLIGATION_LEN: u64 = 1300;
const SOLEND_PROGRAM: &str = "So1endDq2YkqhipRh3WViPa8hdiSpxWy6z3Z6tMCpAo";
const LENDING_MARKET : &str = "4UpD2fh7xH3VP9QQaXtsS1YY3bxzWhtfpks7FatyKvdY";

lazy_static! {
    static ref RESERVES_TO_ASSET_MAP : HashMap<&'static str, &'static str> = [
        ("8PbodeaosQP19SjYFx855UMqWxH2HynZLdBXmsrbac36",  "SOL"),
        ("BgxfHJDzm44T7XG68MYKx7YisTjZu73tVovyZSjJMpmw", "USDC"),
        ("3PArRsZQ6SLkr1WERZWyC6AqsajtALMq4C66ZMYz4dKQ",  "ETH"),
        ("GYzjMCXTDue12eUGKKWAqtF5jcBYNmewr6Db6LaguEaX",  "BTC"),
        ("5suXmvdbKQ98VonxGCXqViuWRu8k4zgZRxndYKsH2fJg",  "SRM"),
        ("8K9WC8xoh2rtQNY7iEGXtPvfbDCi563SdWhCAhuMP2xE", "USDT"),
        ("2dC4V23zJxuv521iYQj8c471jrxYLNQFaGS6YPwtTHMd",  "FTT"),
        ("9n2exoMQwMTzfw6NFoFFujxYPndWVLtKREJePssrKb36",  "RAY"),
    ].iter().cloned().collect();
}

/// Accounts
/// SOLANA
// Reserve Account
const SOL_RE_ACC : &str = "8PbodeaosQP19SjYFx855UMqWxH2HynZLdBXmsrbac36";
// Program Account
const SOL_PT_ACC : &str = "8UviNr47S8eL6J3WfDxMRa3hvLta1VDJwNWqsDgtN3Cv";
// cToken Account
const SOL_CT_ACC: &str = "5h6ssFpeDeRbzsEHDbTQNH7nVGgsKrZydxdSTnLm6QdV";
/// USDC
const USDC_RE_ACC : &str = "BgxfHJDzm44T7XG68MYKx7YisTjZu73tVovyZSjJMpmw";

type Obligations = Vec<(Pubkey, Account, Obligation)>;

fn main() {
    let client = RpcClient::new_with_timeout(RPC_URL.to_string(), Duration::from_secs(120));
    let lending_market_pk= Pubkey::from_str(LENDING_MARKET).unwrap();
    let account = client.get_account(&lending_market_pk).unwrap();

    let lending_market: LendingMarket = LendingMarket::unpack_from_slice(account.data.as_slice()).unwrap();

    let reserve_pk = Pubkey::from_str(SOL_RE_ACC).unwrap();
    let account = client.get_account(&reserve_pk).unwrap();
    let reserve : Reserve = Reserve::unpack_from_slice(account.data.as_slice()).unwrap();

    // Obligations are derived from SOLEND PROGRAM ID
    let solend_program_pk = Pubkey::from_str(SOLEND_PROGRAM).unwrap();

    let program_accounts_config = RpcProgramAccountsConfig {
        filters: Some(vec![
            RpcFilterType::Memcmp(Memcmp {
                offset: 10,
                bytes: MemcmpEncodedBytes::Binary (LENDING_MARKET.to_string()),
                encoding: Some(MemcmpEncoding::Binary)
            }),
            RpcFilterType::DataSize(OBLIGATION_LEN),
        ]),
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            ..RpcAccountInfoConfig::default()
        },
        ..RpcProgramAccountsConfig::default()
        
    };
    
    // Get Obligations
    let program_accounts : Vec<(Pubkey, Account)> = client.get_program_accounts_with_config(&solend_program_pk, program_accounts_config).unwrap();

    let obligations : Obligations = program_accounts.iter().map(|(p,a)| parse_obligation(p, a)).collect();
    let mut total_deposits = HashMap::new(); 
    let mut total_borrows= HashMap::new(); 
    for (_p, _a, o) in obligations {
        for deposit in o.deposits {
            let reserve = &deposit.deposit_reserve;
            let current_deposit = deposit.deposited_amount;
            match RESERVES_TO_ASSET_MAP.get(&reserve.to_string()[..]) {
                Some(asset) => {
                    let total = total_deposits.entry(asset).or_insert(current_deposit);
                    *total += current_deposit;
                },
                None =>  { println!("Unrecognized asset {}", reserve.to_string()) }
            }
        }

        for borrow in o.borrows {
            let reserve = &borrow.borrow_reserve;
            // TODO: wards need to be converted
            let current_borrow = borrow.borrowed_amount_wads.try_ceil_u64().unwrap();
            match RESERVES_TO_ASSET_MAP.get(&reserve.to_string()[..]) {
                Some(asset) => {
                    let total = total_borrows.entry(asset).or_insert(current_borrow);
                    *total += current_borrow;
                },
                None =>  { println!("Unrecognized asset {}", reserve.to_string()) }
            }      
        }
    }

    println!("Number of users: {}", program_accounts.len());
    println!("{:#?}", total_deposits);
    println!("{:#?}", total_borrows);
}

fn parse_obligation(pubkey: &Pubkey, account: &Account) -> (Pubkey, Account, Obligation) {
    let obligation= Obligation::unpack_from_slice(&account.data).unwrap(); 
    return (*pubkey, account.clone(), obligation);
}
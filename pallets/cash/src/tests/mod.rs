#![allow(non_upper_case_globals)]

pub use codec::{Decode, Encode};
use ethereum_client::encode_block_hex;
pub use frame_support::{assert_err, assert_ok, dispatch::DispatchError};
pub use hex_literal::hex;
pub use our_std::convert::TryInto;
pub use our_std::{iter::FromIterator, str::FromStr};
use parking_lot::RwLock;
pub use sp_core::crypto::AccountId32;
use sp_core::offchain::testing;
use std::sync::Arc;

pub mod assets;
pub mod common;
pub mod mock;
pub mod protocol;
pub mod testdata;
pub mod worker;

pub use assets::*;
pub use mock::*;

pub use crate::{
    chains::*, core::*, factor::*, notices::*, params::*, rates::*, reason::*, symbol::*, types::*,
    *,
};

use test_env_log::test;

pub type SysEvent = frame_system::Event<Test>;

#[macro_export]
macro_rules! bal {
    ($string:expr, $units:expr) => {
        Balance::from_nominal($string, $units);
    };
}

#[macro_export]
macro_rules! qty {
    ($string:expr, $units:expr) => {
        Quantity::from_nominal($string, $units);
    };
}

pub fn val_a() -> ValidatorKeys {
    ValidatorKeys {
        substrate_id: AccountId32::from_str("5FfBQ3kwXrbdyoqLPvcXRp7ikWydXawpNs2Ceu3WwFdhZ8W4")
            .unwrap(),
        eth_address: <Ethereum as Chain>::str_to_address(
            "0x8ad1b2918c34ee5d3e881a57c68574ea9dbecb81",
        )
        .unwrap(), // pk: 6bc5ea78f041146e38233f5bc29c703c1cec8eaaa2214353ee8adf7fc598f23d
    }
}

pub fn val_b() -> ValidatorKeys {
    ValidatorKeys {
        substrate_id: AccountId32::from_str("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")
            .unwrap(),
        eth_address: <Ethereum as Chain>::str_to_address(
            "0x6a72a2f14577D9Cd0167801EFDd54a07B40d2b61",
        )
        .unwrap(), // pk: 50f05592dc31bfc65a77c4cc80f2764ba8f9a7cce29c94a51fe2d70cb5599374
    }
}

pub fn premined_block() -> ethereum_client::EthereumBlock {
    ethereum_client::EthereumBlock {
        hash: [
            97, 49, 76, 28, 104, 55, 225, 94, 96, 197, 182, 115, 47, 9, 33, 24, 221, 37, 227, 236,
            104, 31, 94, 8, 155, 58, 154, 210, 55, 78, 90, 138,
        ],
        parent_hash: [
            6, 46, 119, 220, 237, 67, 30, 182, 113, 165, 104, 57, 249, 109, 169, 18, 246, 141, 132,
            16, 36, 102, 87, 72, 211, 140, 211, 214, 121, 89, 97, 234,
        ],
        number: 1,
        events: vec![
            ethereum_client::EthereumEvent::Lock {
                asset: [
                    238, 238, 238, 238, 238, 238, 238, 238, 238, 238, 238, 238, 238, 238, 238, 238,
                    238, 238, 238, 238,
                ],
                sender: [
                    254, 177, 234, 39, 248, 136, 195, 132, 241, 176, 220, 20, 253, 107, 56, 125,
                    95, 244, 112, 49,
                ],
                chain: String::from("ETH"),
                recipient: [
                    81, 60, 31, 244, 53, 236, 206, 221, 15, 218, 94, 221, 42, 213, 229, 70, 31, 14,
                    135, 38, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                ],
                amount: 1000000000000000000,
            },
            ethereum_client::EthereumEvent::Lock {
                asset: [
                    216, 123, 167, 165, 11, 46, 126, 102, 15, 103, 138, 137, 94, 75, 114, 231, 203,
                    76, 205, 156,
                ],
                sender: [
                    254, 177, 234, 39, 248, 136, 195, 132, 241, 176, 220, 20, 253, 107, 56, 125,
                    95, 244, 112, 49,
                ],
                chain: String::from("ETH"),
                recipient: [
                    254, 177, 234, 39, 248, 136, 195, 132, 241, 176, 220, 20, 253, 107, 56, 125,
                    95, 244, 112, 49, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                ],
                amount: 1000000000000000000,
            },
            ethereum_client::EthereumEvent::Lock {
                asset: [
                    228, 232, 31, 166, 177, 99, 39, 212, 183, 140, 254, 184, 58, 173, 224, 75, 167,
                    7, 81, 101,
                ],
                sender: [
                    254, 177, 234, 39, 248, 136, 195, 132, 241, 176, 220, 20, 253, 107, 56, 125,
                    95, 244, 112, 49,
                ],
                chain: String::from("ETH"),
                recipient: [
                    254, 177, 234, 39, 248, 136, 195, 132, 241, 176, 220, 20, 253, 107, 56, 125,
                    95, 244, 112, 49, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                ],
                amount: 1000000000000000000,
            },
        ],
    }
}

pub fn initialize_storage() {
    pallet_oracle::Module::<Test>::initialize_reporters(
        vec![
            "0x85615b076615317c80f14cbad6501eec031cd51c",
            "0xfCEAdAFab14d46e20144F48824d0C09B1a03F2BC",
        ]
        .try_into()
        .unwrap(),
    );

    CashModule::initialize_assets(vec![
        AssetInfo {
            liquidity_factor: FromStr::from_str("7890").unwrap(),
            ..AssetInfo::minimal(
                FromStr::from_str("eth:0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE").unwrap(),
                FromStr::from_str("ETH/18").unwrap(),
            )
        },
        AssetInfo {
            ticker: FromStr::from_str("USD").unwrap(),
            liquidity_factor: FromStr::from_str("7890").unwrap(),
            ..AssetInfo::minimal(
                FromStr::from_str("eth:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48").unwrap(),
                FromStr::from_str("USDC/6").unwrap(),
            )
        },
    ]);

    CashModule::initialize_validators(vec![val_a(), val_b()]);

    Starports::insert(ChainId::Eth, ChainAccount::Eth([0x77; 20]));
    LastProcessedBlock::insert(ChainId::Eth, ChainBlock::Eth(premined_block()));
}

pub fn validator_a_sign(data: &[u8]) -> Result<ChainSignature, Reason> {
    std::env::set_var(
        "ETH_KEY",
        "6bc5ea78f041146e38233f5bc29c703c1cec8eaaa2214353ee8adf7fc598f23d",
    );
    validator_sign::<Test>(data)
}

pub fn a_receive_chain_blocks(blocks: &ChainBlocks) -> Result<(), DispatchError> {
    let signature = validator_a_sign(&blocks.encode())?;
    CashModule::receive_chain_blocks(Origin::none(), blocks.clone(), signature)
}

pub fn a_receive_chain_reorg(reorg: &ChainReorg) -> Result<(), DispatchError> {
    let signature = validator_a_sign(&reorg.encode())?;
    CashModule::receive_chain_reorg(Origin::none(), reorg.clone(), signature)
}

pub fn validator_b_sign(data: &[u8]) -> Result<ChainSignature, Reason> {
    std::env::set_var(
        "ETH_KEY",
        "50f05592dc31bfc65a77c4cc80f2764ba8f9a7cce29c94a51fe2d70cb5599374",
    );
    validator_sign::<Test>(data)
}

pub fn b_receive_chain_blocks(blocks: &ChainBlocks) -> Result<(), DispatchError> {
    let signature = validator_b_sign(&blocks.encode())?;
    CashModule::receive_chain_blocks(Origin::none(), blocks.clone(), signature)
}

pub fn b_receive_chain_reorg(reorg: &ChainReorg) -> Result<(), DispatchError> {
    let signature = validator_b_sign(&reorg.encode())?;
    CashModule::receive_chain_reorg(Origin::none(), reorg.clone(), signature)
}

pub fn all_receive_chain_blocks(blocks: &ChainBlocks) -> Result<(), DispatchError> {
    a_receive_chain_blocks(blocks)?;
    b_receive_chain_blocks(blocks)
}

pub fn gen_mock_responses(
    state: Arc<RwLock<testing::OffchainState>>,
    blocks: Vec<ethereum_client::EthereumBlock>,
    starport_address: <Ethereum as Chain>::Address,
) {
    let mut s = state.write();
    for block in blocks {
        let block_str = encode_block_hex(block.number);

        let get_block_params: Vec<serde_json::Value> = vec![block_str.clone().into(), true.into()];

        let get_block_data = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_getBlockByNumber",
            "params": get_block_params,
            "id":1
        });

        let get_block_result = serde_json::json!({
            "result": {
                "hash":format!("0x{}", hex::encode(&block.hash[..])),
                "number": block_str,
                "parentHash": format!("0x{}", hex::encode(&block.parent_hash[..])),
            }
        });

        let get_block = testing::PendingRequest {
            method: "POST".into(),
            uri: "https://ropsten-eth.compound.finance".into(),
            headers: vec![("Content-Type".to_owned(), "application/json".to_owned())],
            body: serde_json::to_vec(&get_block_data).unwrap(),
            response: Some(serde_json::to_vec(&get_block_result).unwrap()),
            sent: true,
            ..Default::default()
        };

        let get_logs_params = vec![serde_json::json!({
            "address": format!("0x{}", ::hex::encode(&starport_address[..])),
            "fromBlock": &block_str,
            "toBlock": &block_str,
        })];

        let get_logs_data = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_getLogs",
            "params": get_logs_params,
            "id":1
        });

        let get_logs = testing::PendingRequest {
            method: "POST".into(),
            uri: "https://ropsten-eth.compound.finance".into(),
            headers: vec![("Content-Type".to_owned(), "application/json".to_owned())],
            body: serde_json::to_vec(&get_logs_data).unwrap(),
            response: Some(br#"{"jsonrpc":"2.0","id":1,"result":[{"address":"0xd905abba1c5ea48c0598be9f3f8ae31290b58613","blockHash":"0xc94ceed3c8c68f09b1c7be28f594cc6fb01f9cdd7b68f3bf516cab9e89486fcf","blockNumber":"0x9928cb","data":"0x000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000006f05b59d3b2000000000000000000000000000000000000000000000000000000000000000000034554480000000000000000000000000000000000000000000000000000000000","logIndex":"0x58","removed":false,"topics":["0xc459acef3ffe957663bb49d644b20d0c790bcb41573893752a72ba6f023b9386","0x000000000000000000000000eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee","0x000000000000000000000000d3a38d4bd07b87e4516f30ee46cfe8ec4e8b73a4","0xd3a38d4bd07b87e4516f30ee46cfe8ec4e8b73a4000000000000000000000000"],"transactionHash":"0xbae1c242aea30e9ae20cb6c37e2f2d08982e31b42bf3d7dbde6466396abb360e","transactionIndex":"0x24"}]}"#.to_vec()),
            sent: true,
            ..Default::default()
        };

        s.expect_request(get_block);
        s.expect_request(get_logs);
    }
}

#[test]
fn test_it_fails_exec_trx_request_signed() {
    new_test_ext().execute_with(|| {
        // Dispatch a signed extrinsic.
        assert_err!(
            CashModule::exec_trx_request(
                Origin::signed(Default::default()),
                vec![],
                ChainAccountSignature::Eth([0; 20], [0; 65]),
                0
            ),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn test_set_interest_rate_model() {
    new_test_ext().execute_with(|| {
        initialize_storage();
        let expected_model = InterestRateModel::new_kink(100, 101, 5000, 202);
        CashModule::set_rate_model(Origin::root(), Eth, expected_model).unwrap();
        let asset_info = CashModule::asset(Eth).expect("no asset");
        assert_eq!(asset_info.rate_model, expected_model);
    });
}

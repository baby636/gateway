use crate::{
    chains::{ChainAccount, ChainBlock, ChainBlockNumber, ChainBlocks, ChainId},
    debug,
    reason::Reason,
};
use codec::{Decode, Encode};
use ethereum_client::{EthereumBlock, EthereumClientError};
use our_std::RuntimeDebug;
use types_derive::Types;

/// Type for errors coming from event ingression.
#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, Types)]
pub enum EventError {
    NoRpcUrl,
    NoStarportAddress,
    EthereumClientError(EthereumClientError),
    ErrorDecodingHex,
    PolygonClientError(EthereumClientError),
    ActionNotSupported,
}

/// Fetch a block from the underlying chain.
pub fn fetch_chain_block(
    chain_id: ChainId,
    number: ChainBlockNumber,
    starport: ChainAccount,
) -> Result<ChainBlock, Reason> {
    match (chain_id, starport) {
        (ChainId::Gate, _) => Err(Reason::Unreachable),
        (ChainId::Eth, ChainAccount::Eth(eth_starport_address)) => {
            Ok(fetch_eth_block(number, &eth_starport_address).map(ChainBlock::Eth)?)
        }
        (ChainId::Matic, ChainAccount::Matic(starport_address)) => {
            Ok(fetch_matic_block(number, &starport_address).map(ChainBlock::Matic)?)
        }
        (ChainId::Dot, _) => Err(Reason::Unreachable),
        _ => Err(Reason::Unreachable),
    }
}

/// Fetch more blocks from the underlying chain.
pub fn fetch_chain_blocks(
    chain_id: ChainId,
    from: ChainBlockNumber,
    to: ChainBlockNumber,
    starport: ChainAccount,
) -> Result<ChainBlocks, Reason> {
    match (chain_id, starport) {
        (ChainId::Gate, _) => Err(Reason::Unreachable),
        (ChainId::Eth, ChainAccount::Eth(eth_starport_address)) => {
            Ok(fetch_eth_blocks(from, to, &eth_starport_address)?)
        }
        (ChainId::Matic, ChainAccount::Matic(starport_address)) => {
            Ok(fetch_matic_blocks(from, to, &starport_address)?)
        }
        (ChainId::Dot, _) => Err(Reason::Unreachable),
        _ => Err(Reason::Unreachable),
    }
}

/// Fetch a single block from the Etherum Starport.
fn fetch_eth_block(
    number: ChainBlockNumber,
    eth_starport_address: &[u8; 20],
) -> Result<EthereumBlock, EventError> {
    debug!("Fetching Eth Block {}", number);
    let eth_rpc_url = runtime_interfaces::validator_config_interface::get_eth_rpc_url()
        .ok_or(EventError::NoRpcUrl)?;
    let eth_block = ethereum_client::get_block(&eth_rpc_url, eth_starport_address, number)
        .map_err(EventError::EthereumClientError)?;
    Ok(eth_block)
}

/// Fetch a single block from the Etherum Starport.
fn fetch_matic_block(
    number: ChainBlockNumber,
    starport_address: &[u8; 20],
) -> Result<EthereumBlock, EventError> {
    let rpc_url = runtime_interfaces::validator_config_interface::get_matic_rpc_url()
        .ok_or(EventError::NoRpcUrl)?;
    let block = ethereum_client::get_block(&rpc_url, starport_address, number)
        .map_err(EventError::PolygonClientError)?;
    Ok(block)
}

/// Fetch blocks from the Ethereum Starport, return up to `slack` blocks to add to the event queue.
fn fetch_eth_like_blocks<
    F: FnMut(ChainBlockNumber, &[u8; 20]) -> Result<EthereumBlock, EventError>,
    G: FnMut(Vec<EthereumBlock>) -> ChainBlocks,
>(
    chain_id: ChainId,
    from: ChainBlockNumber,
    to: ChainBlockNumber,
    starport_address: &[u8; 20],
    mut fetch_block_fn: F,
    no_result_error: EventError,
    mut chain_blocks_fn: G,
) -> Result<ChainBlocks, EventError> {
    debug!(
        "Fetching Blocks chain_id={:?}, from_block={}, to_block={}",
        chain_id, from, to
    );
    let mut acc: Vec<EthereumBlock> = vec![];
    for block_number in from..to {
        match fetch_block_fn(block_number, starport_address) {
            Ok(block) => {
                acc.push(block);
            }
            Err(err) => {
                if err == no_result_error {
                    break;
                }
                return Err(err);
            }
        }
    }
    Ok(chain_blocks_fn(acc))
}

/// Fetch blocks from the Ethereum Starport, return up to `slack` blocks to add to the event queue.
fn fetch_eth_blocks(
    from: ChainBlockNumber,
    to: ChainBlockNumber,
    eth_starport_address: &[u8; 20],
) -> Result<ChainBlocks, EventError> {
    fetch_eth_like_blocks(
        ChainId::Eth,
        from,
        to,
        eth_starport_address,
        fetch_eth_block,
        EventError::EthereumClientError(EthereumClientError::NoResult),
        ChainBlocks::Eth,
    )
}

/// Fetch blocks from the Polygon Starport, return up to `slack` blocks to add to the event queue.
fn fetch_matic_blocks(
    from: ChainBlockNumber,
    to: ChainBlockNumber,
    starport_address: &[u8; 20],
) -> Result<ChainBlocks, EventError> {
    fetch_eth_like_blocks(
        ChainId::Matic,
        from,
        to,
        starport_address,
        fetch_matic_block,
        EventError::PolygonClientError(EthereumClientError::NoResult),
        ChainBlocks::Matic,
    )
}

#[cfg(test)]
mod tests {
    use crate::events::*;
    use crate::tests::*;
    use sp_core::offchain::testing;
    use sp_core::offchain::{OffchainDbExt, OffchainWorkerExt};

    #[test]
    fn test_fetch_chain_blocks_eth_returns_proper_blocks() -> Result<(), Reason> {
        // XXX should this use new_test_ext_with_http_calls?
        let blocks_to_return = vec![
            ethereum_client::EthereumBlock {
                hash: [1u8; 32],
                parent_hash: [0u8; 32],
                number: 1,
                events: vec![],
            },
            ethereum_client::EthereumBlock {
                hash: [2u8; 32],
                parent_hash: [1u8; 32],
                number: 2,
                events: vec![],
            },
        ];

        let fetch_from = blocks_to_return[0].number;
        let fetch_to = blocks_to_return[blocks_to_return.len() - 1].number + 1;
        const STARPORT_ADDR: [u8; 20] = [1; 20];

        let (offchain, offchain_state) = testing::TestOffchainExt::new();
        let mut t = sp_io::TestExternalities::default();

        t.register_extension(OffchainDbExt::new(offchain.clone()));
        t.register_extension(OffchainWorkerExt::new(offchain));

        gen_mock_responses(offchain_state, blocks_to_return.clone(), STARPORT_ADDR);

        t.execute_with(|| {
            let fetched_blocks = fetch_eth_blocks(fetch_from, fetch_to, &STARPORT_ADDR).unwrap();

            match fetched_blocks {
                ChainBlocks::Eth(blocks) => {
                    for (expected, actual) in blocks_to_return.iter().zip(blocks.iter()) {
                        assert_eq!(actual.hash, expected.hash);
                        assert_eq!(actual.parent_hash, expected.parent_hash);
                        assert_eq!(actual.number, expected.number);
                    }
                }
                _ => panic!("unreachable"),
            }
        });

        Ok(())
    }
}

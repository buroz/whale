use std::error::Error;
use futures_util::StreamExt;
use alloy::{
    consensus::Transaction, 
    providers::{Provider, ProviderBuilder, WsConnect}, 
    rpc::types::eth::BlockId
};

const RPC_URL: &str = "wss://mainnet.gateway.tenderly.co";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let ws = WsConnect::new(RPC_URL); 

    let provider = ProviderBuilder::new().connect_ws(ws).await?;

    let sub = provider.subscribe_blocks().await?;
    let mut stream = sub.into_stream();

    while let Some(block_info) = stream.next().await {
        let provider_clone = provider.clone();

        tokio::spawn(async move {
            let block_id = BlockId::from(block_info.hash);
            match provider_clone.get_block(block_id).await {
                Ok(Some(block)) => {
                    let transactions = block.transactions.as_hashes().unwrap();
                    
                    for tx_hash in transactions {
                        let tx = provider_clone.get_transaction_by_hash(*tx_hash).await.unwrap().unwrap();
                        let value = tx.value();
                        if value > 0 {
                            let tx_request = tx.into_request();
                            let from = tx_request.from.unwrap();
                            let to = tx_request.to.unwrap().into_to().unwrap();
                            println!(
                                "{} | {}->{} : {:?} wei", 
                                tx_hash.to_string(), 
                                from.to_string().chars().take(5).collect::<String>(), 
                                to.to_string().chars().take(5).collect::<String>(), 
                                value
                            );
                        }
                    }
                }
                Ok(None) => {
                    eprintln!("Block not found: {:?}", block_info.hash);
                }
                Err(e) => {
                    eprintln!("Error fetching block: {:?}", e);
                }
            }
        });
    }

    Ok(())
}

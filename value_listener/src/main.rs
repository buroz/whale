use alloy::{ 
    consensus::Transaction, 
    providers::{Provider, 
    ProviderBuilder, WsConnect},
};
use futures_util::StreamExt;
use std::{error::Error};

const RPC_URL: &str = "wss://mainnet.gateway.tenderly.co";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let ws = WsConnect::new(RPC_URL); 

    let provider = ProviderBuilder::new().connect_ws(ws).await?;

    let sub = provider.subscribe_blocks().await?;
    let mut stream = sub.into_stream();
      
    while let Some(block_hash) = stream.next().await {
        let provider_clone = provider.clone();
        
        tokio::spawn(async move {
            let block = provider_clone.get_block_by_hash(block_hash.hash).await.unwrap();

            match block {
                Some(block) => {
                    if let Some(transactions) = block.transactions.as_transactions() {
                        for (_, tx) in transactions.iter().enumerate() {
                            let value = tx.value();
                            println!("{}", value);
                        }
                    }
                }
                None => {
                    println!("Block not found");
                }
            }
        });
    }

    Ok(())
}

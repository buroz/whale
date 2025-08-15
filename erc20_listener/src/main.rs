use alloy::{
    primitives::{address, Uint}, providers::{Provider, ProviderBuilder, WsConnect}, rpc::types::{BlockNumberOrTag, Filter}, sol, sol_types::SolEvent
};
use futures_util::StreamExt;
use std::{collections::HashMap, error::Error};

const RPC_URL: &str = "wss://mainnet.gateway.tenderly.co";

sol! {
    event Transfer(
        address indexed from,
        address indexed to,
        uint256 value
    );
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut tokens_map = HashMap::new();

    tokens_map.insert(address!("0xdAC17F958D2ee523a2206206994597C13D831ec7"), 6);  // USDT
    tokens_map.insert(address!("0xB8c77482e45F1F44dE1745F52C74426C631bDD52"), 18); // BNB
    tokens_map.insert(address!("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"), 6);  // USDC
    tokens_map.insert(address!("0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84"), 18); // stETH

    let ws = WsConnect::new(RPC_URL); 
    // let provider = Arc::new(ProviderBuilder::new().connect_ws(ws).await?); 
    let provider = ProviderBuilder::new().connect_ws(ws).await?;

    let filter = Filter::new()
        .event(Transfer::SIGNATURE)
        .from_block(BlockNumberOrTag::Latest);

    let sub = provider.subscribe_logs(&filter).await?;
    let mut stream = sub.into_stream();
  
    while let Some(log) = stream.next().await {
        let tokens_map_clone = tokens_map.clone();

        tokio::spawn(async move {
            if let Ok(transfer) = Transfer::decode_log_data(&log.data()) {
                let address = log.address();

                if let Some(decimals) = tokens_map_clone.get(&address) {
                    let divisor = Uint::from(10u64).pow(Uint::from(*decimals));
                    let value = transfer.value / divisor;
                    println!(
                        "{} | {} | {}->{}: {}", 
                        log.transaction_hash.unwrap().to_string(), 
                        address, 
                        transfer.from.to_string().chars().take(5).collect::<String>(), 
                        transfer.to.to_string().chars().take(5).collect::<String>(), 
                        value
                    );
                }
            }
        });
    }

    Ok(())
}

use alloy::{
    providers::{Provider, ProviderBuilder, WsConnect},
    rpc::types::{BlockNumberOrTag, Filter},
    sol,
    sol_types::SolEvent,
};
use anyhow::Result;
use futures_util::StreamExt;
use questdb::ingress::{Buffer, ProtocolVersion, Sender, TimestampMicros};
use std::sync::{Arc, Mutex};

const RPC_URL: &str = "wss://mainnet.gateway.tenderly.co";

sol! {
    event Transfer(
        address indexed from,
        address indexed to,
        uint256 value
    );
}

#[tokio::main]
async fn main() -> Result<()> {
    let sender = Sender::from_conf(
        "http::addr=questdb:9000;username=admin;password=quest;retry_timeout=20000;",
    )
    .map_err(|e| anyhow::anyhow!("QuestDB sender error: {}", e))?;

    let sender = Arc::new(Mutex::new(sender));

    let ws = WsConnect::new(RPC_URL);
    let provider = ProviderBuilder::new().connect_ws(ws).await?;

    let filter = Filter::new()
        .event(Transfer::SIGNATURE)
        .from_block(BlockNumberOrTag::Latest);

    let sub = provider.subscribe_logs(&filter).await?;
    let mut stream = sub.into_stream();

    while let Some(log) = stream.next().await {
        let sender_clone = sender.clone();

        tokio::spawn(async move {
            if let Ok(transfer) = Transfer::decode_log_data(&log.data()) {
                let address = log.address();

                let mut buffer = Buffer::new(ProtocolVersion::V2);

                let amount = transfer.value.to_string();

                if amount == "0" {
                    buffer.clear();
                    println!(
                        "Amount is 0 skipping: {:?}",
                        log.transaction_hash.unwrap().to_string()
                    );
                    return;
                }

                if let Err(e) = buffer
                    .table("transfers")
                    .and_then(|b| b.column_str("token_address", address.to_string()))
                    .and_then(|b| b.column_str("from", transfer.from.to_string()))
                    .and_then(|b| b.column_str("to", transfer.to.to_string()))
                    .and_then(|b| {
                        b.column_str(
                            "transaction_hash",
                            log.transaction_hash.unwrap().to_string(),
                        )
                    })
                    .and_then(|b| b.column_i64("log_index", log.log_index.unwrap() as i64))
                    .and_then(|b| b.column_i64("chain_id", 1))
                    .and_then(|b| b.column_i64("block_number", log.block_number.unwrap() as i64))
                    .and_then(|b| b.column_str("amount", amount))
                    .and_then(|b| b.at(TimestampMicros::now()))
                {
                    eprintln!("Error writing to buffer: {}", e);
                }

                if let Ok(mut sender_guard) = sender_clone.lock() {
                    if let Err(e) = sender_guard.flush(&mut buffer) {
                        eprintln!("Error flushing buffer: {}", e);
                    }
                } else {
                    eprintln!("Failed to lock sender");
                }

                buffer.clear();
            }
        });
    }

    Ok(())
}

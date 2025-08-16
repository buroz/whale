use questdb::{
    Result,
    ingress::{
        Sender,
        Buffer,
        TimestampNanos,
        ProtocolVersion
    },
};
use chrono::Utc;

fn main() {
    let mut sender = Sender::from_conf("http::addr=localhost:9000;username=admin;password=quest;retry_timeout=20000;")?;

    let mut buffer = Buffer::new(ProtocolVersion::V2);
    let current_datetime = Utc::now();

    let block_timestamp = TimestampNanos::from_datetime(current_datetime)?;

    for _ in 0..100000000 {
        buffer
            .table("transfers")?
            .column_str("from", "0x1234567890123456789012345678901234567890")?
            .column_str("to", "0x1234567890123456789012345678901234567890")?
            .column_str("transaction_hash", "0x1234567890123456789012345678901234567890")?
            .column_str("token_address", "0x1234567890123456789012345678901234567890")?
            .column_i64("chain_id", 1)?
            .column_i64("block_number", 1234567890)?
            .column_ts("block_timestamp", block_timestamp)?
                .column_i64("amount", 6_000_000_000_000_000_000)?
                .at(block_timestamp)?;

        sender.flush(&mut buffer)?;
    }
    
    buffer.clear();

    Ok(())
}
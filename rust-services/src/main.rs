mod discord;
mod models;
mod service;
mod utils;

use crate::discord::client;
use crate::utils::mongo;
use dotenv::dotenv;
use std::error::Error;

/// โหลดและตรวจสอบ env ที่จำเป็นก่อนเริ่มต้น (fail fast)
fn ensure_env() -> Result<(), Box<dyn Error>> {
    std::env::var("DISCORD_TOKEN").map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "ไม่พบ DISCORD_TOKEN ใน environment",
        )
    })?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    ensure_env()?;

    mongo::init().await?;
    client::run().await?;

    Ok(())
}

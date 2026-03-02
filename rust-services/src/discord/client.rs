use crate::discord::handlers::Handlers;
use serenity::all::{Client, GatewayIntents};
use std::env;

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let token = env::var("DISCORD_TOKEN").expect("ไม่พบ DISCORD_TOKEN");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MEMBERS;

    println!("กำลังเริ่มต้นบอท Discord...");

    let mut client = Client::builder(&token, intents)
        .event_handler(Handlers)
        .await?;

    println!("บอทพร้อมทำงานแล้ว กำลังเชื่อมต่อกับ Discord...");

    client.start().await.map_err(|e| {
        eprintln!("เกิดข้อผิดพลาดกับไคลเอนต์: {e:?}");
        e
    })?;

    Ok(())
}

use crate::discord::commands;
use crate::discord::commands::auto_manga_commands::AutoMangaCommands;
use serenity::all::{
    Command, Context, CreateInteractionResponse, CreateInteractionResponseMessage, EventHandler,
    Interaction, Ready,
};
use serenity::async_trait;

pub struct Handlers;

#[async_trait]
impl EventHandler for Handlers {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            // ป้องกัน panic: slash command ต้องมี subcommand
            if command.data.options.is_empty() {
                let _ = command
                    .create_response(
                        &ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content("ไม่พบคำสั่งย่อย กรุณาระบุตัวเลือก")
                                .ephemeral(true),
                        ),
                    )
                    .await;
                return;
            }

            let data_read = ctx.data.read().await;
            let result: Result<(), serenity::Error> = match command.data.name.as_str() {
                // "user" => commands::user_commands::run(&ctx, &command, &data_read).await,
                "manga" => commands::manga_commands::run(&ctx, &command, &data_read).await,
                "channel" => commands::channels_commands::run(&ctx, &command, &data_read).await,
                _ => {
                    // ไม่พบคำสั่ง
                    command
                        .create_response(
                            &ctx.http,
                            CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new()
                                    .content("ไม่พบคำสั่งนี้")
                                    .ephemeral(true),
                            ),
                        )
                        .await
                        .ok();
                    Ok(())
                }
            };

            // จัดการข้อผิดพลาด (ส่งตอบได้เฉพาะเมื่อ command ยังไม่ได้ตอบ)
            if let Err(why) = result {
                eprintln!("เกิดข้อผิดพลาดในคำสั่ง '{}': {:?}", command.data.name, why);
                let _ = command
                    .create_response(
                        &ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content("เกิดข้อผิดพลาดในการประมวลผลคำสั่ง")
                                .ephemeral(true),
                        ),
                    )
                    .await;
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} เชื่อมต่อแล้ว!", ready.user.name);

        // ลงทะเบียนคำสั่งทั้งหมด
        let commands: Vec<serenity::all::CreateCommand> = collect_all_commands();

        // ลงทะเบียนคำสั่งทั้งหมดแบบ global
        match Command::set_global_commands(&ctx.http, commands).await {
            Ok(_) => println!("ลงทะเบียนคำสั่งทั้งหมดสำเร็จ"),
            Err(why) => println!("ลงทะเบียนคำสั่งล้มเหลว: {:?}", why),
        }

        // เริ่มการอัพเดทอัตโนมัติ
        let ctx_clone: Context = ctx.clone();

        tokio::spawn(async move {
            let auto_manga: AutoMangaCommands = AutoMangaCommands::new();
            auto_manga.run_periodic_update(&ctx_clone).await;
        });
    }
}

// ฟังก์ชั่นรวบรวมคำสั่งทั้งหมด
fn collect_all_commands() -> Vec<serenity::all::CreateCommand> {
    vec![
        // commands::user_commands::register(),
        commands::manga_commands::register(),
        commands::channels_commands::register(),
    ]
}

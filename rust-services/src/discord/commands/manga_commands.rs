use crate::models::manga::Manga;
use crate::service::manga_service::MangaService;
use serenity::all::{
    Colour, CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateEmbed, CreateEmbedFooter, CreateInteractionResponse, CreateInteractionResponseMessage,
};

pub fn register() -> CreateCommand {
    CreateCommand::new("manga")
        .description("จัดการการ์ตูน")
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "add", "เพิ่มการ์ตูนใหม่")
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::String,
                        "url",
                        "URL ของการ์ตูนที่ต้องการเพิ่ม",
                    )
                    .required(true),
                ),
        )
}

// UI Utility Function
pub async fn show_manga_info_ui(
    command: &CommandInteraction,
    ctx: &Context,
    title: &str,
    description: &str,
    color: Colour,
) -> serenity::Result<()> {
    let embed = CreateEmbed::new()
        .title(title)
        .description(description)
        .color(color)
        .footer(CreateEmbedFooter::new("ระบบจัดการการ์ตูน"));

    command
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .embed(embed)
                    .ephemeral(true),
            ),
        )
        .await
}

async fn add_manga(ctx: &Context, command: &CommandInteraction) -> serenity::Result<()> {
    let url: &str = command
        .data
        .options
        .iter()
        .find(|opt| opt.name == "add")
        .and_then(|opt| match &opt.value {
            serenity::all::CommandDataOptionValue::SubCommand(sub_opts) => Some(sub_opts),
            _ => None,
        })
        .and_then(|sub_opts| sub_opts.iter().find(|opt| opt.name == "url"))
        .and_then(|opt| opt.value.as_str())
        .unwrap_or("");

    if url.is_empty() {
        return show_manga_info_ui(
            command,
            ctx,
            "ข้อมูลไม่ครบ",
            "กรุณาระบุ URL ของการ์ตูน",
            Colour::RED,
        )
        .await;
    }

    if !url.starts_with("https://") {
        return show_manga_info_ui(
            command,
            ctx,
            "URL ไม่ถูกต้อง",
            "URL ต้องขึ้นต้นด้วย https://",
            Colour::RED,
        )
        .await;
    }

    // แปลง URL ก่อนบันทึกลง MongoDB
    let normalized_url = if url.contains("สดใสเมะ.com") {
        url.replace("สดใสเมะ.com", "xn--l3c0azab5a2gta.com")
    } else {
        url.to_string()
    };

    // ตรวจสอบว่ามีการ์ตูนนี้ในฐานข้อมูลหรือไม่
    match MangaService::get_by_url(&normalized_url).await {
        Ok(Some(_)) => {
            show_manga_info_ui(
                command,
                ctx,
                "การเพิ่มการ์ตูน",
                "การ์ตูนนี้มีอยู่ในระบบแล้ว",
                Colour::GOLD,
            )
            .await
        }
        Ok(None) => {
            // สร้างข้อมูลการ์ตูนใหม่
            let manga: Manga = Manga::new(
                "Untitled".to_string(), // ตั้งชื่อชั่วคราว
                normalized_url.to_string(),
                0,               // เริ่มต้นที่ตอนที่ 0
                normalized_url.to_string(), // ใช้ URL เดิมเป็น chapter URL
                None,            // ไม่มีรูปภาพ
            );

            // บันทึกลงฐานข้อมูล
            match MangaService::create(&manga).await {
                Ok(_) => {
                    let description = format!(
                        "**เพิ่มการ์ตูนสำเร็จ**\n\
                        **ชื่อเรื่อง:** {}\n\
                        **ตอนล่าสุด:** {:?}\n\
                        **URL:** {}",
                        manga.title, manga.latest_chapter, manga.url
                    );

                    show_manga_info_ui(
                        command,
                        ctx,
                        "เพิ่มการ์ตูนสำเร็จ",
                        &description,
                        Colour::DARK_GREEN,
                    )
                    .await
                }
                Err(e) => {
                    show_manga_info_ui(
                        command,
                        ctx,
                        "เกิดข้อผิดพลาด",
                        &format!("เกิดข้อผิดพลาดในการบันทึกข้อมูล: {}", e),
                        Colour::RED,
                    )
                    .await
                }
            }
        }
        Err(e) => {
            show_manga_info_ui(
                command,
                ctx,
                "เกิดข้อผิดพลาด",
                &format!("เกิดข้อผิดพลาดในการตรวจสอบข้อมูล: {}", e),
                Colour::RED,
            )
            .await
        }
    }
}

pub async fn run(
    ctx: &Context,
    command: &CommandInteraction,
    _: &serenity::prelude::TypeMap,
) -> serenity::Result<()> {
    let subcommand_name = command
        .data
        .options
        .first()
        .map(|o| o.name.as_str())
        .unwrap_or("");

    match subcommand_name {
        "add" => add_manga(ctx, command).await,
        _ => show_manga_info_ui(command, ctx, "ไม่รู้จักคำสั่ง", "ไม่รู้จักคำสั่งย่อยนี้", Colour::RED).await,
    }
}

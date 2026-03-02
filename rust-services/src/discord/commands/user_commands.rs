use crate::service::user_service::UserService;
use chrono;
use serenity::all::{
    Colour, CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateEmbed, CreateEmbedFooter, CreateInteractionResponse, CreateInteractionResponseMessage,
};

pub fn register() -> CreateCommand {
    CreateCommand::new("user")
        .description("จัดการข้อมูลผู้ใช้")
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "info",
            "ดูข้อมูลของผู้ใช้",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "register",
            "ลงทะเบียนผู้ใช้",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "update",
            "อัพเดทข้อมูลผู้ใช้",
        ))
}

// UI Utility Function
pub async fn show_user_info_ui(
    command: &CommandInteraction,
    ctx: &Context,
    title: &str,
    description: &str,
    color: Colour,
) -> serenity::Result<()> {
    let embed: CreateEmbed = CreateEmbed::new()
        .title(title)
        .description(description)
        .color(color)
        .footer(CreateEmbedFooter::new("ระบบจัดการผู้ใช้"));

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

// Modified register_user function
async fn register_user(ctx: &Context, command: &CommandInteraction) -> serenity::Result<()> {
    let guild_id = match command.guild_id {
        Some(id) => id,
        None => {
            return show_user_info_ui(
                command,
                ctx,
                "ไม่รองรับ",
                "คำสั่งนี้ใช้ได้เฉพาะในเซิร์ฟเวอร์เท่านั้น",
                Colour::RED,
            )
            .await;
        }
    };
    let guild_id_str = guild_id.to_string();
    let user_id: String = command.user.id.to_string();
    let user_name: String = command.user.name.clone();
    let global_name: String = command.user.global_name.clone().unwrap_or_default();

    let guild = match guild_id.to_partial_guild(&ctx.http).await {
        Ok(g) => g,
        Err(e) => {
            return show_user_info_ui(
                command,
                ctx,
                "เกิดข้อผิดพลาด",
                &format!("ไม่สามารถโหลดข้อมูลเซิร์ฟเวอร์: {}", e),
                Colour::RED,
            )
            .await;
        }
    };
    let guild_name: String = guild.name.clone();

    let member = match guild_id.member(&ctx.http, command.user.id).await {
        Ok(m) => m,
        Err(e) => {
            return show_user_info_ui(
                command,
                ctx,
                "เกิดข้อผิดพลาด",
                &format!("ไม่สามารถโหลดข้อมูลสมาชิก: {}", e),
                Colour::RED,
            )
            .await;
        }
    };
    let guild_user_nickname: &String = member.nick.as_ref().unwrap_or(&member.user.name);

    match UserService::check_user_exists(&user_id, &guild_id_str).await {
        Ok(true) => {
            show_user_info_ui(
                command,
                ctx,
                "การลงทะเบียน",
                &format!("ผู้ใช้ {} ได้ลงทะเบียนไว้แล้ว", guild_user_nickname),
                Colour::GOLD,
            )
            .await
        }
        Ok(false) => {
            match UserService::register_user(
                &guild_id_str,
                &user_id,
                &guild_name,
                guild_user_nickname,
                &global_name,
                &user_name,
            )
            .await
            {
                Ok(_) => {
                    show_user_info_ui(
                        command,
                        ctx,
                        "การลงทะเบียนสำเร็จ",
                        &format!("ลงทะเบียนผู้ใช้ {} สำเร็จ", guild_user_nickname),
                        Colour::DARK_GREEN,
                    )
                    .await
                }
                Err(e) => {
                    show_user_info_ui(
                        command,
                        ctx,
                        "เกิดข้อผิดพลาด",
                        &format!("เกิดข้อผิดพลาด: {}", e),
                        Colour::RED,
                    )
                    .await
                }
            }
        }
        Err(e) => {
            show_user_info_ui(
                command,
                ctx,
                "เกิดข้อผิดพลาด",
                &format!("เกิดข้อผิดพลาดในการตรวจสอบข้อมูลผู้ใช้: {}", e),
                Colour::RED,
            )
            .await
        }
    }
}

// Modified get_user_info function
async fn get_user_info(ctx: &Context, command: &CommandInteraction) -> serenity::Result<()> {
    let guild_id = match command.guild_id {
        Some(id) => id.to_string(),
        None => {
            return show_user_info_ui(
                command,
                ctx,
                "ไม่รองรับ",
                "คำสั่งนี้ใช้ได้เฉพาะในเซิร์ฟเวอร์เท่านั้น",
                Colour::RED,
            )
            .await;
        }
    };
    let user_id: String = command.user.id.to_string();

    match UserService::find_by_user_id(&user_id, &guild_id).await {
        Ok(Some(user)) => {
            let description = format!(
                "**User:** {}\n\
                **ชื่อผู้ใช้:** {}\n\
                **ชื่อในเซิร์ฟเวอร์:** {}\n\
                **เซิร์ฟเวอร์:** {}\n\
                **วันที่ลงทะเบียน:** {}",
                user.user_name,
                user.global_name,
                user.guild_user_nickname,
                user.guild_name,
                chrono::DateTime::<chrono::Utc>::from(user.created_at.to_system_time())
                    .format("%d/%m/%Y %H:%M:%S")
                    .to_string()
            );

            show_user_info_ui(command, ctx, "ข้อมูลผู้ใช้", &description, Colour::BLUE).await
        }
        Ok(None) => {
            show_user_info_ui(command, ctx, "ไม่พบข้อมูล", "ไม่พบข้อมูลผู้ใช้ในระบบ", Colour::RED).await
        }
        Err(e) => {
            show_user_info_ui(
                command,
                ctx,
                "เกิดข้อผิดพลาด",
                &format!("เกิดข้อผิดพลาด: {}", e),
                Colour::RED,
            )
            .await
        }
    }
}

// Modified update_user function
async fn update_user(ctx: &Context, command: &CommandInteraction) -> serenity::Result<()> {
    let guild_id = match command.guild_id {
        Some(id) => id,
        None => {
            return show_user_info_ui(
                command,
                ctx,
                "ไม่รองรับ",
                "คำสั่งนี้ใช้ได้เฉพาะในเซิร์ฟเวอร์เท่านั้น",
                Colour::RED,
            )
            .await;
        }
    };
    let guild_id_str = guild_id.to_string();
    let user_id: String = command.user.id.to_string();

    let member = match guild_id.member(&ctx.http, command.user.id).await {
        Ok(m) => m,
        Err(e) => {
            return show_user_info_ui(
                command,
                ctx,
                "เกิดข้อผิดพลาด",
                &format!("ไม่สามารถโหลดข้อมูลสมาชิก: {}", e),
                Colour::RED,
            )
            .await;
        }
    };
    let guild_user_nickname: String = member
        .nick
        .as_ref()
        .unwrap_or(&member.user.name)
        .to_string();
    let global_name: String = command.user.global_name.clone().unwrap_or_default();
    let user_name: String = command.user.name.clone();

    // ตรวจสอบว่าผู้ใช้มีอยู่ในระบบหรือไม่
    match UserService::find_by_user_id(&user_id, &guild_id_str).await {
        Ok(Some(mut user)) => {
            // อัพเดทข้อมูล
            user.guild_user_nickname = guild_user_nickname;
            user.global_name = global_name;
            user.user_name = user_name;

            // บันทึกข้อมูลลงฐานข้อมูล
            match UserService::update_user(&user).await {
                Ok(_) => {
                    let description = format!(
                        "**อัพเดทข้อมูลสำเร็จ**\n\
                        **User:** {}\n\
                        **ชื่อผู้ใช้:** {}\n\
                        **ชื่อในเซิร์ฟเวอร์:** {}\n\
                        **เซิร์ฟเวอร์:** {}\n\
                        **วันที่อัพเดท:** {}",
                        user.user_name,
                        user.global_name,
                        user.guild_user_nickname,
                        user.guild_name,
                        chrono::DateTime::<chrono::Utc>::from(user.updated_at.to_system_time())
                            .format("%d/%m/%Y %H:%M:%S")
                            .to_string()
                    );

                    show_user_info_ui(
                        command,
                        ctx,
                        "อัพเดทข้อมูลสำเร็จ",
                        &description,
                        Colour::DARK_GREEN,
                    )
                    .await
                }
                Err(e) => {
                    show_user_info_ui(
                        command,
                        ctx,
                        "เกิดข้อผิดพลาด",
                        &format!("เกิดข้อผิดพลาดในการอัพเดทข้อมูล: {}", e),
                        Colour::RED,
                    )
                    .await
                }
            }
        }
        Ok(None) => {
            show_user_info_ui(
                command,
                ctx,
                "ไม่พบข้อมูล",
                "ไม่พบข้อมูลผู้ใช้ในระบบ กรุณาลงทะเบียนก่อน",
                Colour::RED,
            )
            .await
        }
        Err(e) => {
            show_user_info_ui(
                command,
                ctx,
                "เกิดข้อผิดพลาด",
                &format!("เกิดข้อผิดพลาดในการค้นหาข้อมูลผู้ใช้: {}", e),
                Colour::RED,
            )
            .await
        }
    }
}

// Modified run function
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
        "info" => get_user_info(ctx, command).await,
        "register" => register_user(ctx, command).await,
        "update" => update_user(ctx, command).await,
        _ => show_user_info_ui(command, ctx, "ไม่รู้จักคำสั่ง", "ไม่รู้จักคำสั่งย่อยนี้", Colour::RED).await,
    }
}

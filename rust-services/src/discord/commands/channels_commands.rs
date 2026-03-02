use crate::models::channels::Channel;
use crate::service::channels_service::ChannelsService;
use serenity::all::{
    Colour, CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateEmbed, CreateEmbedFooter, CreateInteractionResponse, CreateInteractionResponseMessage,
};

pub fn register() -> CreateCommand {
    CreateCommand::new("channel")
        .description("จัดการช่อง")
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "register", "บันทึกข้อมูลช่อง")
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::Channel,
                        "channel",
                        "ช่องที่ต้องการบันทึก",
                    )
                    .required(true),
                ),
        )
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "list",
            "ดูรายการช่องทั้งหมด",
        ))
}

// UI Utility Function
pub async fn show_channel_info_ui(
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
        .footer(CreateEmbedFooter::new("ระบบจัดการช่อง"));

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

async fn list_channels(ctx: &Context, command: &CommandInteraction) -> serenity::Result<()> {
    let guild_id = match command.guild_id {
        Some(id) => id,
        None => {
            return show_channel_info_ui(
                command,
                ctx,
                "ไม่รองรับ",
                "คำสั่งนี้ใช้ได้เฉพาะในเซิร์ฟเวอร์เท่านั้น",
                Colour::RED,
            )
            .await;
        }
    };

    match ChannelsService::get_channels_by_guild(&guild_id.to_string()).await {
        Ok(channels) => {
            if channels.is_empty() {
                show_channel_info_ui(
                    command,
                    ctx,
                    "รายการช่อง",
                    "ไม่พบช่องที่บันทึกไว้ในระบบ",
                    Colour::RED,
                )
                .await
            } else {
                let description = channels
                    .iter()
                    .map(|channel| {
                        format!(
                            "**{}** ({})\n\
                        ",
                            channel.channel_name, channel.channel_id,
                        )
                    })
                    .collect::<Vec<String>>()
                    .join("\n");

                show_channel_info_ui(command, ctx, "รายการช่องทั้งหมด", &description, Colour::BLUE)
                    .await
            }
        }
        Err(e) => {
            show_channel_info_ui(
                command,
                ctx,
                "เกิดข้อผิดพลาด",
                &format!("เกิดข้อผิดพลาดในการดึงข้อมูล: {}", e),
                Colour::RED,
            )
            .await
        }
    }
}

async fn add_channel(ctx: &Context, command: &CommandInteraction) -> serenity::Result<()> {
    let channel_id = match command
        .data
        .options
        .iter()
        .find(|opt| opt.name == "register")
        .and_then(|opt| match &opt.value {
            serenity::all::CommandDataOptionValue::SubCommand(sub_opts) => Some(sub_opts),
            _ => None,
        })
        .and_then(|sub_opts| sub_opts.iter().find(|opt| opt.name == "channel"))
        .and_then(|opt| opt.value.as_channel_id())
    {
        Some(id) => id,
        None => {
            return show_channel_info_ui(
                command,
                ctx,
                "ข้อมูลไม่ครบ",
                "กรุณาเลือกช่องที่ต้องการบันทึก",
                Colour::RED,
            )
            .await;
        }
    };

    let guild_id = match command.guild_id {
        Some(id) => id,
        None => {
            return show_channel_info_ui(
                command,
                ctx,
                "ไม่รองรับ",
                "คำสั่งนี้ใช้ได้เฉพาะในเซิร์ฟเวอร์เท่านั้น",
                Colour::RED,
            )
            .await;
        }
    };
    let guild = ctx.http.get_guild(guild_id).await?;
    let guild_name: String = guild.name;

    match channel_id.to_channel(&ctx.http).await {
        Ok(channel) => {
            if let serenity::model::channel::Channel::Guild(channel) = channel {
                let existing_channels: Vec<Channel> = match ChannelsService::get_channels_by_guild(&guild_id.to_string()).await {
                    Ok(channels) => channels,
                    Err(e) => {
                        return show_channel_info_ui(
                            command,
                            ctx,
                            "เกิดข้อผิดพลาด",
                            &format!("เกิดข้อผิดพลาดในการดึงข้อมูล: {}", e),
                            Colour::RED,
                        )
                        .await;
                    }
                };

                let channel_doc: Channel = Channel::new(
                    channel.id.to_string(),
                    guild_id.to_string(),
                    channel.name.clone(),
                    guild_name.clone(),
                );

                if existing_channels.iter().any(|c: &Channel| c.guild_id == guild_id.to_string()) {
                    // Update existing channel
                    match ChannelsService::update_channel(&channel_doc).await {
                        Ok(_) => {
                            show_channel_info_ui(
                                command,
                                ctx,
                                "อัพเดทข้อมูลสำเร็จ",
                                &format!("อัพเดทข้อมูลช่อง {} ลงฐานข้อมูลสำเร็จ", channel.name),
                                Colour::DARK_GREEN,
                            )
                            .await
                        }
                        Err(e) => {
                            show_channel_info_ui(
                                command,
                                ctx,
                                "เกิดข้อผิดพลาด",
                                &format!("เกิดข้อผิดพลาดในการอัพเดทข้อมูล: {}", e),
                                Colour::RED,
                            )
                            .await
                        }
                    }
                } else {
                    // Create new channel
                    match ChannelsService::create_channel(channel_doc).await {
                        Ok(_) => {
                            show_channel_info_ui(
                                command,
                                ctx,
                                "เพิ่มข้อมูลสำเร็จ",
                                &format!("เพิ่มข้อมูลช่อง {} ลงฐานข้อมูลสำเร็จ", channel.name),
                                Colour::DARK_GREEN,
                            )
                            .await
                        }
                        Err(e) => {
                            show_channel_info_ui(
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
            } else {
                show_channel_info_ui(
                    command,
                    ctx,
                    "เกิดข้อผิดพลาด",
                    "ไม่สามารถบันทึกข้อมูลช่องส่วนตัวได้",
                    Colour::RED,
                )
                .await
            }
        }
        Err(e) => {
            show_channel_info_ui(
                command,
                ctx,
                "เกิดข้อผิดพลาด",
                &format!("ไม่พบช่องที่ระบุ: {}", e),
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
        "register" => add_channel(ctx, command).await,
        "list" => list_channels(ctx, command).await,
        _ => show_channel_info_ui(command, ctx, "ไม่รู้จักคำสั่ง", "ไม่รู้จักคำสั่งย่อยนี้", Colour::RED).await,
    }
}

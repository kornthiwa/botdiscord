use crate::discord::commands::scrape_manga_commands::scrape_manga_auto;
use crate::models::manga::Manga;
use crate::service::channels_service::ChannelsService;
use crate::service::manga_service::MangaService;
use serenity::all::{ChannelId, Colour, Context, CreateEmbed, CreateEmbedFooter, CreateMessage};
use tokio::time;

// โครงสร้างสำหรับจัดการคำสั่งอัพเดทมังงะอัตโนมัติ
pub struct AutoMangaCommands;

impl AutoMangaCommands {
    pub fn new() -> Self {
        AutoMangaCommands
    }

    // ส่งข้อความอัพเดทไปยังช่องที่กำหนด
    async fn send_update_to_all_channels(ctx: &Context, mangas: Vec<Manga>) {
        // ดึงข้อมูลช่อง
        let channels: Vec<crate::models::channels::Channel> = match ChannelsService::get_all_channels().await {
            Ok(channels) => channels,
            Err(e) => {
                println!("เกิดข้อผิดพลาดในการดึง channels: {:?}", e);
                return;
            }
        };

        let channel_ids: Vec<ChannelId> = channels
            .into_iter()
            .filter_map(|channel| channel.channel_id.parse::<u64>().ok().map(ChannelId::new))
            .collect();

        if channel_ids.is_empty() {
            println!("ไม่พบช่องสำหรับการอัพเดทมังงะในฐานข้อมูล");
            return;
        }
        println!("channel_ids: {:?}", channel_ids);

        // ส่งข้อมูลทุกมังงะ
        for manga in mangas {
            let embed = CreateEmbed::new()
                .title(format!("การอัพเดทมังงะ: {}", manga.title))
                .description(format!("อัพเดทถึงตอนที่ {}", manga.latest_chapter))
                .field("ชื่อมังงะ", &manga.title, true)
                .field("ตอนล่าสุด", format!("ตอนที่ {}", manga.latest_chapter), true)
                .field("ลิงก์ตอนล่าสุด", &manga.latest_chapter_url, false)
                .field(
                    "เวลาอัพเดท",
                    chrono::DateTime::<chrono::Utc>::from(manga.updated_at.to_system_time())
                        .format("%d/%m/%Y %H:%M:%S")
                        .to_string(),
                    true,
                )
                .color(Colour::DARK_GREEN)
                .footer(CreateEmbedFooter::new("ระบบอัพเดทมังงะอัตโนมัติ"));

            let embed = if let Some(image_url) = &manga.image_url {
                embed.thumbnail(image_url)
            } else {
                embed
            };

            for channel_id in &channel_ids {
                let message = CreateMessage::new().add_embed(embed.clone());

                if let Err(why) = channel_id.send_message(&ctx.http, message).await {
                    println!("เกิดข้อผิดพลาดในการส่งข้อความไปยังช่อง {}: {:?}", channel_id, why);
                }
            }
        }
        println!("ส่งข้อความอัพเดทสำเร็จ!");
    }

    // ฟังก์ชันสำหรับการอัพเดทแบบเป็นระยะ (ทุก 4 ชั่วโมง)
    pub async fn run_periodic_update(&self, ctx: &Context) {
        println!("เริ่มการทำงานอัพเดทอัตโนมัติ...");
        let mut interval = time::interval(time::Duration::from_secs(4 * 60 * 60));

        loop {
            interval.tick().await;

            // ดึงข้อมูลมังงะทั้งหมดจากฐานข้อมูล
            match MangaService::get_all().await {
                Ok(mangas) => {
                    for manga in mangas {
                        // เช็คอัพเดทจากเว็บไซต์โดยใช้ฟังก์ชัน auto-detection
                        match scrape_manga_auto(&manga.url).await {
                            Ok((title, latest_chapter, chapter_url, image_url)) => {
                                // ถ้าตอนล่าสุดใหม่กว่าในฐานข้อมูล
                                if latest_chapter > manga.latest_chapter {
                                    println!(
                                        "พบการอัพเดทใหม่สำหรับ {}: ตอนที่ {}",
                                        title, latest_chapter
                                    );

                                    // สร้างข้อมูลมังงะใหม่
                                    let updated_manga = Manga::new(
                                        title,
                                        manga.url,
                                        latest_chapter,
                                        chapter_url,
                                        image_url,
                                    );

                                    // อัพเดทข้อมูลในฐานข้อมูล
                                    if let Err(e) = MangaService::update(&updated_manga).await {
                                        println!("เกิดข้อผิดพลาดในการอัพเดทข้อมูลมังงะ: {:?}", e);
                                        continue;
                                    }

                                    // ส่งการแจ้งเตือน
                                    Self::send_update_to_all_channels(ctx, vec![updated_manga])
                                        .await;
                                } else {
                                    println!(
                                        "{} ยังไม่มีการอัพเดทใหม่ (ตอนล่าสุด: {})",
                                        title, manga.latest_chapter
                                    );
                                }
                            }
                            Err(e) => {
                                println!(
                                    "เกิดข้อผิดพลาดในการเช็คอัพเดทมังงะ {}: {:?} (URL: {})",
                                    manga.title, e, manga.url
                                );
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("เกิดข้อผิดพลาดในการดึงข้อมูลมังงะ: {:?}", e);
                }
            }
        }
    }

    // ฟังก์ชันสำหรับทดสอบการอัพเดท (ทุก 10 วินาที)
    // pub async fn run_test_update(&self, ctx: &Context) {
    //     println!("เริ่มการทดสอบการอัพเดท...");
    //     let mut interval = time::interval(time::Duration::from_secs(10));

    //     // ดึงข้อมูลมังงะทั้งหมดจากฐานข้อมูล
    //     match MangaService::get_all().await {
    //         Ok(mangas) => {
    //             for manga in mangas {
    //                 // ใช้ฟังก์ชัน auto-detection สำหรับทุกเว็บไซต์
    //                 match scrape_manga_auto(&manga.url).await {
    //                     Ok((title, latest_chapter, chapter_url, image_url)) => {
    //                         // เช็คว่าตอนที่ดึงมาใหม่กว่าในฐานข้อมูลหรือไม่
    //                         if latest_chapter > manga.latest_chapter {
    //                             println!(
    //                                 "Test Scrape: {} ตอนที่ {} url: {}",
    //                                 title, latest_chapter, chapter_url
    //                             );
    //                             println!(
    //                                 "พบการอัพเดทใหม่! (ตอนเดิม: {}, ตอนใหม่: {})",
    //                                 manga.latest_chapter, latest_chapter
    //                             );

    //                             // สร้างข้อมูลมังงะใหม่
    //                             let updated_manga = Manga::new(
    //                                 title,
    //                                 manga.url,
    //                                 latest_chapter,
    //                                 chapter_url,
    //                                 image_url,
    //                             );

    //                             // อัพเดทข้อมูลในฐานข้อมูล
    //                             if let Err(e) = MangaService::update(&updated_manga).await {
    //                                 println!("เกิดข้อผิดพลาดในการอัพเดทข้อมูลมังงะ: {:?}", e);
    //                                 continue;
    //                             }

    //                             // ส่งแจ้งเตือนไปยังทุกช่อง
    //                             Self::send_update_to_all_channels(ctx, vec![updated_manga])
    //                                 .await;
    //                         } else {
    //                             println!(
    //                                 "{} ยังไม่มีการอัพเดทใหม่ (ตอนล่าสุด: {})",
    //                                 title, manga.latest_chapter
    //                             );
    //                         }
    //                     }
    //                     Err(e) => {
    //                         println!("เกิดข้อผิดพลาดในการเช็คอัพเดทมังงะ {}: {:?}", manga.title, e);
    //                     }
    //                 }
    //             }
    //         }
    //         Err(e) => {
    //             println!("เกิดข้อผิดพลาดในการดึงข้อมูลมังงะ: {:?}", e);
    //         }
    //     }

    //     // ส่งการอัพเดทอีก 3 ครั้ง (ทดสอบวนซ้ำ)
    //     for _ in 0..3 {
    //         interval.tick().await;
    //         println!("ถึงเวลาอัพเดทแล้ว!");

    //         // ดึงข้อมูลมังงะทั้งหมดอีกครั้ง
    //         match MangaService::get_all().await {
    //             Ok(mangas) => {
    //                 for manga in mangas {
    //                     // ใช้ฟังก์ชัน auto-detection สำหรับทุกเว็บไซต์
    //                     match scrape_manga_auto(&manga.url).await {
    //                         Ok((title, latest_chapter, chapter_url, image_url)) => {
    //                             // เช็คว่าตอนที่ดึงมาใหม่กว่าในฐานข้อมูลหรือไม่
    //                             if latest_chapter > manga.latest_chapter {
    //                                 println!(
    //                                     "Test Scrape: {} ตอนที่ {} url: {}",
    //                                     title, latest_chapter, chapter_url
    //                                 );
    //                                 println!(
    //                                     "พบการอัพเดทใหม่! (ตอนเดิม: {}, ตอนใหม่: {})",
    //                                     manga.latest_chapter, latest_chapter
    //                                 );

    //                                 let updated_manga = Manga::new(
    //                                     title,
    //                                     manga.url,
    //                                     latest_chapter,
    //                                     chapter_url,
    //                                     image_url,
    //                                 );

    //                                 if let Err(e) = MangaService::update(&updated_manga).await {
    //                                     println!("เกิดข้อผิดพลาดในการอัพเดทข้อมูลมังงะ: {:?}", e);
    //                                     continue;
    //                                 }

    //                                 Self::send_update_to_all_channels(ctx, vec![updated_manga])
    //                                     .await;
    //                             } else {
    //                                 println!(
    //                                     "{} ยังไม่มีการอัพเดทใหม่ (ตอนล่าสุด: {})",
    //                                     title, manga.latest_chapter
    //                                 );
    //                             }
    //                         }
    //                         Err(e) => {
    //                             println!(
    //                                 "เกิดข้อผิดพลาดในการเช็คอัพเดทมังงะ {}: {:?}",
    //                                 manga.title, e
    //                             );
    //                         }
    //                     }
    //                 }
    //             }
    //             Err(e) => {
    //                 println!("เกิดข้อผิดพลาดในการดึงข้อมูลมังงะ: {:?}", e);
    //             }
    //         }
    //     }
    // }
}

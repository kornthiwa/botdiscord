use reqwest::Client;
use scraper::{Html, Selector};
use std::time::Duration;

pub async fn scrape_manga_sing_manga(
    url: &str,
) -> Result<(String, i32, String, Option<String>), Box<dyn std::error::Error + Send + Sync>> {
    // Check if URL is from sing-manga.com
    if !url.contains("sing-manga.com") {
        return Err("URL ต้องเป็นของ sing-manga.com เท่านั้น".into());
    }

    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
        .build()?;

    let response = client.get(url)
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8")
        .header("Accept-Language", "en-US,en;q=0.9")
        .header("Cache-Control", "no-cache")
        .header("Pragma", "no-cache")
        .send()
        .await?;

    let html = response.text().await?;
    let document = Html::parse_document(&html);

    // Title: try h1.entry-title, then og:title, then any h1
    let title = document
        .select(&Selector::parse("h1.entry-title").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .or_else(|| {
            document
                .select(&Selector::parse(r##"meta[property="og:title"]"##).unwrap())
                .next()
                .and_then(|el| el.value().attr("content").map(String::from))
        })
        .or_else(|| {
            document
                .select(&Selector::parse("h1").unwrap())
                .next()
                .map(|el| el.text().collect::<String>().trim().to_string())
        })
        .filter(|s| !s.is_empty())
        .ok_or("ไม่พบชื่อการ์ตูน")?;

    let image_selector = Selector::parse("div.thumb img").unwrap();
    let image_url = document
        .select(&image_selector)
        .next()
        .map(|img| img.value().attr("src").unwrap_or_default().to_string());

    let chapter_selector = Selector::parse("div.lastend div.inepcx a").unwrap();
    let chapter_elements: Vec<_> = document.select(&chapter_selector).collect();

    let latest_chapter_element = chapter_elements.last().ok_or("ไม่พบตอนล่าสุด")?;

    let chapter_number_selector = Selector::parse("span.epcurlast").unwrap();
    let chapter_text = latest_chapter_element
        .select(&chapter_number_selector)
        .next()
        .ok_or("ไม่พบหมายเลขตอน")?
        .text()
        .collect::<String>();

    let chapter_number = chapter_text
        .replace("Chapter ", "")
        .trim()
        .replace(|c: char| !c.is_numeric(), "")
        .parse::<i32>()?;

    let chapter_url = latest_chapter_element
        .value()
        .attr("href")
        .ok_or("ไม่พบ URL ของตอนล่าสุด")?
        .to_string();

    // println!("\n=== Scraping Complete ===");
    // println!("Final results:");
    // println!("Title: {}", title);
    // println!("Image URL: {:?}", image_url);
    // println!("Chapter: {}", chapter_number);
    // println!("URL: {}", chapter_url);
    // println!("========================\n");

    Ok((title, chapter_number, chapter_url, image_url))
}

pub async fn scrape_manga_thai_manga(
    url: &str,
) -> Result<(String, i32, String, Option<String>), Box<dyn std::error::Error + Send + Sync>> {
    // Check if URL is from the Thai manga website
    if !url.contains("xn--l3c0azab5a2gta.com") {
        return Err("URL ต้องเป็นของ xn--l3c0azab5a2gta.com เท่านั้น".into());
    }

    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
        .build()?;

    let response = client.get(url)
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8")
        .header("Accept-Language", "th-TH,th;q=0.9,en;q=0.8")
        .header("Cache-Control", "no-cache")
        .header("Pragma", "no-cache")
        .send()
        .await?;

    let html = response.text().await?;
    let document = Html::parse_document(&html);

    // Extract title: try h1.entry-title, then og:title, then any h1
    let title = document
        .select(&Selector::parse("h1.entry-title").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .or_else(|| {
            document
                .select(&Selector::parse(r##"meta[property="og:title"]"##).unwrap())
                .next()
                .and_then(|el| el.value().attr("content").map(String::from))
        })
        .or_else(|| {
            document
                .select(&Selector::parse("h1").unwrap())
                .next()
                .map(|el| el.text().collect::<String>().trim().to_string())
        })
        .filter(|s| !s.is_empty())
        .ok_or("ไม่พบชื่อการ์ตูน")?;

    // Extract image URL
    let image_selector = Selector::parse("div.thumb img").unwrap();
    let image_url = document
        .select(&image_selector)
        .next()
        .map(|img| img.value().attr("src").unwrap_or_default().to_string());

    // Extract latest chapter information
    let chapter_selector = Selector::parse("div.lastend div.inepcx a").unwrap();
    let chapter_elements: Vec<_> = document.select(&chapter_selector).collect();

    let latest_chapter_element = chapter_elements.last().ok_or("ไม่พบตอนล่าสุด")?;

    // Extract chapter number from the span with class "epcurlast"
    let chapter_number_selector = Selector::parse("span.epcurlast").unwrap();
    let chapter_text = latest_chapter_element
        .select(&chapter_number_selector)
        .next()
        .ok_or("ไม่พบหมายเลขตอน")?
        .text()
        .collect::<String>();

    // Parse chapter number - remove "ตอนที่ " prefix and extract numeric part
    let chapter_number = chapter_text
        .replace("ตอนที่ ", "")
        .trim()
        .replace(|c: char| !c.is_numeric(), "")
        .parse::<i32>()?;

    // Extract chapter URL
    let chapter_url = latest_chapter_element
        .value()
        .attr("href")
        .ok_or("ไม่พบ URL ของตอนล่าสุด")?
        .to_string();

    // println!("\n=== Scraping Complete ===");
    // println!("Final results:");
    // println!("Title: {}", title);
    // println!("Image URL: {:?}", image_url);
    // println!("Chapter: {}", chapter_number);
    // println!("URL: {}", chapter_url);
    // println!("========================\n");

    Ok((title, chapter_number, chapter_url, image_url))
}

// Generic function that automatically detects the website and uses appropriate scraper
pub async fn scrape_manga_auto(
    url: &str,
) -> Result<(String, i32, String, Option<String>), Box<dyn std::error::Error + Send + Sync>> {
    if url.contains("sing-manga.com") {
        scrape_manga_sing_manga(url).await
    } else if url.contains("xn--l3c0azab5a2gta.com") || url.contains("สดใสเมะ.com") {
        scrape_manga_thai_manga(url).await
    } else {
        Err("ไม่รองรับเว็บไซต์นี้ กรุณาใช้ sing-manga.com หรือ สดใสเมะ.com".into())
    }
}


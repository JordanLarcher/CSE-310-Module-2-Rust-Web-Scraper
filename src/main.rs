use clap::{Parser, ValueEnum};
use reqwest::blocking::get;
use rusqlite::{params, Connection};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScrapedData {
    title: String,
    content: String,
    link: String,
}

#[derive(Debug, Clone, ValueEnum)]
enum OutputFormat {
    Csv,
    Json,
    Sqlite,
}

#[derive(Parser, Debug)]
#[command(author, version, about = "A basic web scraper in Rust", long_about = None)]
struct Args {
    /// URL to scrape
    #[arg(short, long)]
    url: String,

    /// CSS selector for elements to scrape (e.g., "h2", "a", "div.article")
    #[arg(short, long)]
    selector: String,

    /// Output format
    #[arg(short = 'f', long, value_enum, default_value = "json")]
    format: OutputFormat,

    /// Output file name (without extension)
    #[arg(short, long, default_value = "output")]
    output: String,

    /// Filter results by keyword (case-insensitive)
    #[arg(short = 'k', long)]
    keyword: Option<String>,

    /// Limit number of results
    #[arg(short, long)]
    limit: Option<usize>,
}

fn scrape_website(url: &str, selector_str: &str) -> Result<Vec<ScrapedData>, Box<dyn Error>> {
    println!("Fetching: {}", url);
    
    let body = if url.starts_with("http://") || url.starts_with("https://") {
        // Fetch from URL
        let response = get(url)?;
        response.text()?
    } else if Path::new(url).exists() {
        // Read from local file
        println!("Reading local file...");
        let mut file = File::open(url)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        contents
    } else {
        return Err(format!("Invalid URL or file path: {}", url).into());
    };
    
    let document = Html::parse_document(&body);
    let selector = Selector::parse(selector_str)
        .map_err(|e| format!("Invalid CSS selector: {:?}", e))?;
    
    let mut data = Vec::new();
    
    for element in document.select(&selector) {
        let title = element
            .text()
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string();
        
        let link = element
            .value()
            .attr("href")
            .unwrap_or("")
            .to_string();
        
        let content = element.inner_html();
        
        if !title.is_empty() {
            data.push(ScrapedData {
                title,
                content,
                link,
            });
        }
    }
    
    println!("Scraped {} elements", data.len());
    Ok(data)
}

fn filter_data(data: Vec<ScrapedData>, keyword: Option<String>, limit: Option<usize>) -> Vec<ScrapedData> {
    let mut filtered = data;
    
    if let Some(kw) = keyword {
        let kw_lower = kw.to_lowercase();
        filtered.retain(|item| {
            item.title.to_lowercase().contains(&kw_lower)
                || item.content.to_lowercase().contains(&kw_lower)
        });
        println!("Filtered to {} elements matching keyword '{}'", filtered.len(), kw);
    }
    
    if let Some(limit_val) = limit {
        filtered.truncate(limit_val);
        println!("Limited to {} elements", filtered.len());
    }
    
    filtered
}

fn export_to_csv(data: &[ScrapedData], filename: &str) -> Result<(), Box<dyn Error>> {
    let path = format!("{}.csv", filename);
    let mut writer = csv::Writer::from_path(&path)?;
    
    writer.write_record(&["Title", "Content", "Link"])?;
    
    for item in data {
        writer.write_record(&[&item.title, &item.content, &item.link])?;
    }
    
    writer.flush()?;
    println!("Data exported to {}", path);
    Ok(())
}

fn export_to_json(data: &[ScrapedData], filename: &str) -> Result<(), Box<dyn Error>> {
    let path = format!("{}.json", filename);
    let json = serde_json::to_string_pretty(&data)?;
    
    let mut file = File::create(&path)?;
    file.write_all(json.as_bytes())?;
    
    println!("Data exported to {}", path);
    Ok(())
}

fn export_to_sqlite(data: &[ScrapedData], filename: &str) -> Result<(), Box<dyn Error>> {
    let path = format!("{}.db", filename);
    let conn = Connection::open(&path)?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS scraped_data (
            id INTEGER PRIMARY KEY,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            link TEXT
        )",
        [],
    )?;
    
    for item in data {
        conn.execute(
            "INSERT INTO scraped_data (title, content, link) VALUES (?1, ?2, ?3)",
            params![item.title, item.content, item.link],
        )?;
    }
    
    println!("Data exported to SQLite database: {}", path);
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    
    println!("Web Scraper Started");
    println!("==================");
    
    let scraped_data = scrape_website(&args.url, &args.selector)?;
    
    if scraped_data.is_empty() {
        println!("No data found with the provided selector.");
        return Ok(());
    }
    
    let filtered_data = filter_data(scraped_data, args.keyword, args.limit);
    
    if filtered_data.is_empty() {
        println!("No data remaining after filtering.");
        return Ok(());
    }
    
    match args.format {
        OutputFormat::Csv => export_to_csv(&filtered_data, &args.output)?,
        OutputFormat::Json => export_to_json(&filtered_data, &args.output)?,
        OutputFormat::Sqlite => export_to_sqlite(&filtered_data, &args.output)?,
    }
    
    println!("==================");
    println!("Scraping completed successfully!");
    
    Ok(())
}

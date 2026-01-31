use clap::{Parser, ValueEnum};
use reqwest::header::{HeaderMap, USER_AGENT};
use rusqlite::{params, Connection};
use scraper::{Html, Selector};
use serde::Serialize;
use url::Url;
use std::error::Error;
use std::fs::File;


// 1. A Generic Data Structure
#[derive(Debug, Serialize)]
struct ScrapedItem {
    #[serde(rename = "Source URL")]
    source_url: String,
    #[serde(rename = "Tag Content")]
    content: String,
    #[serde(rename = "Attribute Value")]
    attribute_value: Option<String>,
}

#[derive(Debug, Clone, ValueEnum)]
enum OutputFormat {
    Csv,
    Json,
    Sqlite,
}

#[derive(Parser, Debug)]
#[command(author, version, about = "A Generic Web Scraper", long_about = None)]
struct Args {
    /// The full URL of the website to scrape
    #[arg(short, long)]
    url: String,

    /// CSS selector for elements (e.g., "h1", ".product-title", "a")
    #[arg(short, long)]
    selector: String,

    /// Optional: Specific attribute to extract (e.g., "href", "src")
    #[arg(short, long)]
    attribute: Option<String>,

    /// Output format
    #[arg(short = 'f', long, value_enum, default_value = "json")]
    format: OutputFormat,

    /// Output file name
    #[arg(short, long, default_value = "scraped_data")]
    output: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    
    // Validate the URL
    let target_url = Url::parse(&args.url).map_err(|_| "Invalid URL provided")?;

    println!("Scraping: {}", target_url);
    println!("Selector: {}", args.selector);

    // 2. Fetch
    let html_content = fetch_html(target_url.as_str()).await?;
    
    // 3. Generic Extraction
    let results = extract_generic(&html_content, &args.selector, args.attribute.as_deref(), target_url.as_str());

    if results.is_empty() {
        println!("No elements found matching that selector.");
        return Ok(());
    }

    println!("Found {} items.", results.len());

    // 4. Export
    match args.format {
        OutputFormat::Csv => export_to_csv(&results, &args.output)?,
        OutputFormat::Json => export_to_json(&results, &args.output)?,
        OutputFormat::Sqlite => export_to_sqlite(&results, &args.output)?,
    }

    Ok(())
}

/// Fetches the HTML from any URL
pub async fn fetch_html(url: &str) -> Result<String, reqwest::Error> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36".parse().unwrap());
    headers.insert("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8".parse().unwrap());

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .redirect(reqwest::redirect::Policy::limited(3))
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let response = client.get(url).send().await?.error_for_status()?;
    let final_url = response.url().to_string();
    println!("Final URL after redirects: {}", final_url);
    response.text().await
}

/// Performs generic extraction based on user-provided CSS selectors
fn extract_generic(html: &str, selector_str: &str, attr_name: Option<&str>, source: &str) -> Vec<ScrapedItem> {
    let document = Html::parse_document(html);
    
    // Attempt to parse the user's CSS selector
    let selector = match Selector::parse(selector_str) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Error: Invalid CSS selector '{}'", selector_str);
            return vec![];
        }
    };

    document.select(&selector)
        .map(|element| {
            let content = element.text().collect::<Vec<_>>().join(" ").trim().to_string();
            
            // If the user asked for an attribute (like href), get it
            let attribute_value = attr_name.and_then(|a| element.value().attr(a).map(|v| v.to_string()));

            ScrapedItem {
                source_url: source.to_string(),
                content,
                attribute_value,
            }
        })
        .filter(|item| !item.content.is_empty() || item.attribute_value.is_some())
        .collect()
}

// --- Generic Export Logic ---

fn export_to_csv(data: &[ScrapedItem], filename: &str) -> Result<(), Box<dyn Error>> {
    let path = format!("{}.csv", filename);
    let mut writer = csv::Writer::from_path(&path)?;
    for item in data {
        writer.serialize(item)?;
    }
    writer.flush()?;
    println!("Saved to {}", path);
    Ok(())
}

fn export_to_json(data: &[ScrapedItem], filename: &str) -> Result<(), Box<dyn Error>> {
    let path = format!("{}.json", filename);
    let file = File::create(&path)?;
    serde_json::to_writer_pretty(file, data)?;
    println!("Saved to {}", path);
    Ok(())
}

fn export_to_sqlite(data: &[ScrapedItem], filename: &str) -> Result<(), Box<dyn Error>> {
    let path = format!("{}.db", filename);
    let conn = Connection::open(path)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS scraped_items (
            id INTEGER PRIMARY KEY,
            source TEXT,
            content TEXT,
            attribute TEXT
        )",
        [],
    )?;

    for item in data {
        conn.execute(
            "INSERT INTO scraped_items (source, content, attribute) VALUES (?1, ?2, ?3)",
            params![item.source_url, item.content, item.attribute_value],
        )?;
    }
    Ok(())
}
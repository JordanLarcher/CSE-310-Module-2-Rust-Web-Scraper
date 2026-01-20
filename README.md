# CSE-310-Module-2-Rust-Web-Scraper

A basic web scraper written in Rust that extracts data from websites and converts it into different formats (CSV, JSON, SQLite). It supports filtering and organizing information based on user-defined criteria.

## Features

- 🌐 Scrape web pages using CSS selectors
- 📊 Export data to multiple formats:
  - CSV (spreadsheet)
  - JSON
  - SQLite database
- 🔍 Filter results by keyword
- 📏 Limit number of results
- ⚡ Fast and efficient with Rust

## Installation

Make sure you have Rust installed. If not, install it from [rustup.rs](https://rustup.rs/).

```bash
cargo build --release
```

## Usage

Basic syntax:

```bash
cargo run -- --url <URL> --selector <CSS_SELECTOR> [OPTIONS]
```

### Options

- `-u, --url <URL>` - URL to scrape (required)
- `-s, --selector <SELECTOR>` - CSS selector for elements to scrape (required)
- `-f, --format <FORMAT>` - Output format: csv, json, or sqlite (default: json)
- `-o, --output <NAME>` - Output file name without extension (default: output)
- `-k, --keyword <KEYWORD>` - Filter results by keyword (case-insensitive)
- `-l, --limit <NUMBER>` - Limit number of results

### Examples

**Scrape headings from a website and save as JSON:**
```bash
cargo run -- --url "https://example.com" --selector "h2"
```

**Scrape links and save as CSV:**
```bash
cargo run -- --url "https://example.com" --selector "a" --format csv --output links
```

**Scrape articles, filter by keyword, and save to SQLite:**
```bash
cargo run -- --url "https://news.ycombinator.com" --selector ".titleline > a" --format sqlite --keyword "rust" --limit 10
```

**Scrape paragraphs and limit results:**
```bash
cargo run -- --url "https://example.com" --selector "p" --limit 5 --output paragraphs
```

## Output Files

- **CSV**: `<output_name>.csv` - Spreadsheet format with columns: Title, Content, Link
- **JSON**: `<output_name>.json` - JSON array of scraped objects
- **SQLite**: `<output_name>.db` - SQLite database with `scraped_data` table

## Common CSS Selectors

- `h1`, `h2`, `h3` - Headings
- `p` - Paragraphs
- `a` - Links
- `.classname` - Elements with specific class
- `#id` - Element with specific ID
- `div.article` - Div with class "article"
- `li > a` - Links that are direct children of list items

## Dependencies

- `reqwest` - HTTP client for fetching web pages
- `scraper` - HTML parsing and CSS selector support
- `csv` - CSV file generation
- `serde` & `serde_json` - JSON serialization
- `rusqlite` - SQLite database support
- `clap` - Command-line argument parsing

## License

This project is open source and available under the MIT License.

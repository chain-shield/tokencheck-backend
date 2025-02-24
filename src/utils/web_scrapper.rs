//! Utility functions for web scraping.
//! This module contains functions to fetch a website, parse its HTML, extract text,
//! and process/filter the content to return a cleaned string.

use std::collections::HashSet;

use reqwest;
use scraper::{Html, Selector};

use crate::app_config::WEBSITE_MAX_CHARACTER_LENGTH;

/// Scrapes a website given its URL and extracts textual content from the HTML.
///
/// This function performs the following steps:
/// 1. Fetches the webpage content as a String.
/// 2. Parses the HTML document.
/// 3. Uses a CSS selector to extract text from common textual elements (e.g., paragraphs,
///    headings, list items, etc.).
/// 4. Filters out lines that likely contain non-relevant or code-like tokens.
/// 5. Trims and removes empty lines.
/// 6. Deduplicates the text to remove repeated lines.
/// 7. Truncates the final text to a maximum character length defined by
///    `WEBSITE_MAX_CHARACTER_LENGTH`.
///
/// # Arguments
///
/// * `website_url` - A string slice representing the URL of the website to scrape.
///
/// # Returns
///
/// * `Ok(String)` containing the cleaned and processed website text.
/// * `Err` if any error occurs during fetching or processing the webpage.
///
/// # Errors
///
/// This function will return an error if the HTTP request fails, the HTML cannot be parsed,
/// or if the CSS selector fails to compile.
///
/// # Examples
///
/// ```rust
/// # use crate::utils::web_scrapper::scrape_site_and_get_text;
/// # use anyhow::Result;
/// # #[tokio::main]
/// # async fn main() -> Result<()> {
/// let text = scrape_site_and_get_text("https://www.example.com").await?;
/// println!("{}", text);
/// # Ok(())
/// # }
/// ```

pub async fn scrape_site_and_get_text(website_url: &str) -> anyhow::Result<String> {
    // 1) Fetch the webpage content as a String.
    let response_text = reqwest::get(website_url).await?.text().await?;

    // 2) Wrap the parsing and text extraction logic in a blocking task.
    let final_text = tokio::task::spawn_blocking(move || {
        // Parse the HTML document.
        let document = Html::parse_document(&response_text);

        // Define a CSS selector for common textual elements.
        let text_selector = Selector::parse(
            "p, h1, h2, h3, h4, h5, h6, li, span, div, a, blockquote, section, article, main",
        )
        .map_err(|e| anyhow::anyhow!("Failed to parse selector: {}", e))?;

        // Extract text from the selected elements.
        let raw_text = document
            .select(&text_selector)
            .flat_map(|element| element.text())
            .collect::<Vec<_>>()
            .join(" ");

        // Filter out lines that are likely to contain non-relevant or code-like patterns.
        let body_text = raw_text
            .lines()
            .filter(|line| {
                !(line.contains("function")
                    || line.contains("var ")
                    || line.contains("if (")
                    || line.contains("if(")
                    || line.contains("then(")
                    || line.contains("} else")
                    || line.contains("else {")
                    || line.contains("for(")
                    || line.contains("for (")
                    || line.contains("i=")
                    || line.contains("{")
                    || line.contains("}")
                    || line.contains("()")
                    || line.contains("]")
                    || line.contains("[")
                    || line.contains("[i]")
                    || line.ends_with(";")
                    || line.contains("// ")
                    || line.contains("/*")
                    || line.contains("*/")
                    || line.contains("-wrapper")
                    || line.contains("webkit")
                    || line.contains(":hover")
                    || line.contains("const ")
                    || line.contains("+ i")
                    || line.contains("console.")
                    || line.contains("document.")
                    || line.contains("$("))
            })
            .collect::<Vec<_>>()
            .join("\n");

        // Clean up the text by trimming each line and removing empty lines.
        let final_text = body_text
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n");

        // Deduplicate the text by removing repeated lines.
        let final_text = dedup_text(final_text);

        // Limit the final text to a maximum specified number of characters.
        let final_text = first_n_chars(&final_text, WEBSITE_MAX_CHARACTER_LENGTH as usize);

        Ok(final_text)
    })
    .await?;

    final_text
}

/// Removes duplicate lines from the provided text while preserving the original order.
///
/// # Arguments
///
/// * `text` - A string whose lines will be checked for duplicates.
///
/// # Returns
///
/// A new `String` with duplicate lines removed.
fn dedup_text(text: String) -> String {
    let mut seen = HashSet::new();
    let mut deduped = Vec::new();

    for line in text.lines() {
        if !seen.contains(line) {
            seen.insert(line);
            deduped.push(line);
        }
    }

    deduped.join("\n")
}

/// Returns a new string containing the first \(`n`\) characters of the given string.
///
/// # Arguments
///
/// * `s` - The input string.
/// * `n` - The number of characters to include in the result.
///
/// # Returns
///
/// A `String` composed of the first \(`n`\) characters of \(`s`\).
///
/// # Examples
///
/// ```rust
/// # use crate::utils::web_scrapper::first_n_chars;
/// let result = first_n_chars("Hello, world!", 5);
/// assert_eq!(result, "Hello");
/// ```
pub fn first_n_chars(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

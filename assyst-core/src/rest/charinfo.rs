use reqwest::Client;

static CHARINFO_URL: &str = "https://www.fileformat.info/info/unicode/char/";

pub async fn get_char_info(client: &Client, ch: char) -> anyhow::Result<(String, String)> {
    let url = format!("{}{:x}", CHARINFO_URL, ch as u32);

    Ok((client.get(&url).send().await?.text().await?, url))
}

/// Attempts to extract the page title for charingo
pub fn extract_page_title(input: &str) -> Option<String> {
    let dom = tl::parse(input, tl::ParserOptions::default()).ok()?;
    let parser = dom.parser();

    let tag = dom.query_selector("title")?.next()?.get(parser)?;

    Some(tag.inner_text(parser).into_owned())
}

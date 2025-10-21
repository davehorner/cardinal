use scraper::{Html, Selector};

/// If the given HTML contains a LinkedIn-style redirect, extract the real URL.
/// Returns Some(real_url) if found, else None.
pub fn extract_linkedin_redirect(html: &str) -> Option<String> {
    if html.contains("data-tracking-will-navigate") {
        let doc = Html::parse_document(html);
        if let Ok(sel) = Selector::parse("a[data-tracking-will-navigate][href]") {
            if let Some(a) = doc.select(&sel).next() {
                if let Some(href) = a.value().attr("href") {
                    return Some(href.to_string());
                }
            }
        }
    }
    None
}

//! Brave Search API Connector.
//!
//! Queries the Brave Search API (`https://api.search.brave.com/res/v1/web/search`)
//! to retrieve real-time, privacy-preserving web results in `< 600ms`.

/// An individual web search result from Brave.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchHit {
    /// Webpage title.
    pub title: String,
    /// Webpage destination URL.
    pub url: String,
    /// Text snippet / summary.
    pub description: String,
}

/// Brave Search Connector.
#[derive(Debug, Clone)]
pub struct BraveSearchClient {
    api_key: String,
}

impl BraveSearchClient {
    /// Creates a new Brave search client with the provided API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
        }
    }

    /// Generates the request URL and headers for a query.
    pub fn build_request(&self, query: &str, count: usize) -> (String, Vec<(&'static str, String)>) {
        let count = count.clamp(1, 20);
        let encoded_query = url_encode(query);
        let url = format!(
            "https://api.search.brave.com/res/v1/web/search?q={}&count={}&safesearch=moderate",
            encoded_query, count
        );
        let headers = vec![
            ("Accept", "application/json".to_string()),
            ("Accept-Encoding", "gzip".to_string()),
            ("X-Subscription-Token", self.api_key.clone()),
        ];
        (url, headers)
    }

    /// Parses a JSON response string into structured search hits.
    pub fn parse_response(&self, json_body: &str) -> Result<Vec<SearchHit>, &'static str> {
        let mut hits = Vec::new();
        let mut cursor = 0;
        while let Some(title_pos) = json_body[cursor..].find("\"title\"") {
            let abs_title_pos = cursor + title_pos;
            let title = extract_value_after(&json_body[abs_title_pos..], ":");

            let url = if let Some(url_pos) = json_body[abs_title_pos..].find("\"url\"") {
                extract_value_after(&json_body[abs_title_pos + url_pos..], ":")
            } else {
                String::new()
            };

            let description = if let Some(desc_pos) = json_body[abs_title_pos..].find("\"description\"") {
                extract_value_after(&json_body[abs_title_pos + desc_pos..], ":")
            } else {
                String::new()
            };

            if !title.is_empty() && !url.is_empty() {
                hits.push(SearchHit {
                    title,
                    url,
                    description,
                });
            }
            cursor = abs_title_pos + 7;
        }

        Ok(hits)
    }

    /// Executes a live web search against the Brave Search API.
    pub fn search_live(&self, query: &str, count: usize) -> Result<Vec<SearchHit>, String> {
        let (url, headers) = self.build_request(query, count);
        let header_refs: Vec<(&str, &str)> = headers.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let res = kineti_core::http_get(&url, &header_refs)
            .map_err(|e| format!("Brave API network error: {}", e))?;
        if !res.success {
            return Err(format!("Brave API HTTP error {}: {}", res.status, res.body));
        }
        self.parse_response(&res.body).map_err(|e| e.to_string())
    }

    /// Builds a shopping-specific search request (adds "buy" + "price" to query).
    pub fn build_shopping_request(&self, product: &str, count: usize) -> (String, Vec<(&'static str, String)>) {
        let query = format!("{} buy price", product);
        self.build_request(&query, count)
    }

    /// Executes a live shopping search against the Brave Search API.
    pub fn search_shopping_live(&self, product: &str, count: usize) -> Result<Vec<SearchHit>, String> {
        let (url, headers) = self.build_shopping_request(product, count);
        let header_refs: Vec<(&str, &str)> = headers.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let res = kineti_core::http_get(&url, &header_refs)
            .map_err(|e| format!("Brave Shopping network error: {}", e))?;
        if !res.success {
            return Err(format!("Brave Shopping HTTP error {}: {}", res.status, res.body));
        }
        self.parse_response(&res.body).map_err(|e| e.to_string())
    }

    /// Builds a ticket-search request (adds "tickets" + "near me" to query).
    pub fn build_ticket_request(&self, event: &str, count: usize) -> (String, Vec<(&'static str, String)>) {
        let query = format!("{} tickets buy", event);
        self.build_request(&query, count)
    }

    /// Executes a live ticket search against the Brave Search API.
    pub fn search_tickets_live(&self, event: &str, count: usize) -> Result<Vec<SearchHit>, String> {
        let (url, headers) = self.build_ticket_request(event, count);
        let header_refs: Vec<(&str, &str)> = headers.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let res = kineti_core::http_get(&url, &header_refs)
            .map_err(|e| format!("Brave Ticket search network error: {}", e))?;
        if !res.success {
            return Err(format!("Brave Ticket search HTTP error {}: {}", res.status, res.body));
        }
        self.parse_response(&res.body).map_err(|e| e.to_string())
    }
}

fn extract_value_after(s: &str, delimiter: &str) -> String {
    if let Some(pos) = s.find(delimiter) {
        let rest = &s[pos + delimiter.len()..];
        if let Some(start_q) = rest.find('"') {
            let after_q = &rest[start_q + 1..];
            let mut out = String::new();
            let mut escaping = false;
            for c in after_q.chars() {
                if escaping {
                    out.push(c);
                    escaping = false;
                } else if c == '\\' {
                    escaping = true;
                } else if c == '"' {
                    break;
                } else {
                    out.push(c);
                }
            }
            return out;
        }
    }
    String::new()
}

/// Standard percent-encoding for query string parameters.
pub fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 2);
    for b in s.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            b' ' => out.push('+'),
            _ => {
                out.push_str(&format!("%{:02X}", b));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brave_request_builder() {
        let client = BraveSearchClient::new("BSA_test_key_123");
        let (url, headers) = client.build_request("SpaceX Starship launch update", 5);

        assert!(url.contains("SpaceX+Starship+launch+update"));
        assert!(url.contains("count=5"));
        assert_eq!(headers[2].0, "X-Subscription-Token");
        assert_eq!(headers[2].1, "BSA_test_key_123");
    }

    #[test]
    fn test_brave_response_parser() {
        let client = BraveSearchClient::new("dummy");
        let sample_json = r#"{
            "query": {"original": "SpaceX"},
            "web": {
                "results": [
                    {
                        "title": "SpaceX - Official Starship Updates",
                        "url": "https://spacex.com/launches",
                        "description": "Starship Flight 6 target date set for Thursday."
                    }
                ]
            }
        }"#;

        let hits = client.parse_response(sample_json).expect("Parse succeeds");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].title, "SpaceX - Official Starship Updates");
        assert_eq!(hits[0].url, "https://spacex.com/launches");
        assert!(hits[0].description.contains("Starship Flight 6"));
    }

    #[test]
    fn test_brave_shopping_request() {
        let client = BraveSearchClient::new("BSA_key");
        let (url, _) = client.build_shopping_request("Sony WH-1000XM5", 5);
        assert!(url.contains("Sony+WH-1000XM5+buy+price"));
        assert!(url.contains("count=5"));
    }

    #[test]
    fn test_brave_ticket_request() {
        let client = BraveSearchClient::new("BSA_key");
        let (url, _) = client.build_ticket_request("Hans Zimmer Live NYC", 10);
        assert!(url.contains("Hans+Zimmer+Live+NYC+tickets+buy"));
        assert!(url.contains("count=10"));
    }
}

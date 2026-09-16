//! Notion REST API Connector.
//!
//! Provides database queries and page creation for notes, roadmaps, and tasks
//! using official Notion integration tokens.

/// A Notion database query result item.
#[derive(Debug, Clone, PartialEq)]
pub struct NotionPageItem {
    /// Notion page UUID.
    pub id: String,
    /// Page title.
    pub title: String,
    /// Direct URL to Notion page.
    pub url: String,
}

/// Parameters to append a new page or row in Notion.
#[derive(Debug, Clone)]
pub struct CreateNotionPageRequest {
    /// Target Parent Database ID.
    pub parent_database_id: String,
    /// Page title.
    pub title: String,
    /// Notes or description content.
    pub content: String,
}

/// Notion API Connector.
#[derive(Debug, Clone)]
pub struct NotionClient {
    api_key: String,
}

impl NotionClient {
    /// Creates a new Notion client.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
        }
    }

    /// Builds a database query request URL and headers.
    pub fn build_query_request(&self, database_id: &str) -> (String, Vec<(&'static str, String)>) {
        let url = format!("https://api.notion.com/v1/databases/{}/query", database_id);
        let headers = vec![
            ("Authorization", format!("Bearer {}", self.api_key)),
            ("Notion-Version", "2022-06-28".to_string()),
            ("Content-Type", "application/json".to_string()),
        ];
        (url, headers)
    }

    /// Builds JSON payload for page creation.
    pub fn build_create_page_payload(&self, req: &CreateNotionPageRequest) -> String {
        let title_escaped = req.title.replace('"', "\\\"").replace('\n', " ");
        let content_escaped = req.content.replace('"', "\\\"").replace('\n', " ");
        format!(
            "{{\"parent\":{{\"database_id\":\"{}\"}},\"properties\":{{\"Name\":{{\"title\":[{{\"text\":{{\"content\":\"{}\"}}}}]}}}},\"children\":[{{\"object\":\"block\",\"type\":\"paragraph\",\"paragraph\":{{\"rich_text\":[{{\"type\":\"text\",\"text\":{{\"content\":\"{}\"}}}}]}}}}]}}",
            req.parent_database_id, title_escaped, content_escaped
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notion_query_builder() {
        let client = NotionClient::new("ntn_test_key");
        let (url, headers) = client.build_query_request("db_abc123");

        assert!(url.contains("databases/db_abc123/query"));
        assert_eq!(headers[0].1, "Bearer ntn_test_key");
        assert_eq!(headers[1].1, "2022-06-28");
    }

    #[test]
    fn test_notion_create_page_payload() {
        let client = NotionClient::new("ntn_test_key");
        let req = CreateNotionPageRequest {
            parent_database_id: "db_xyz".to_string(),
            title: "Beta Launch Milestone".to_string(),
            content: "Target date confirmed for April 15.".to_string(),
        };
        let payload = client.build_create_page_payload(&req);
        assert!(payload.contains("\"database_id\":\"db_xyz\""));
        assert!(payload.contains("\"content\":\"Beta Launch Milestone\""));
    }
}

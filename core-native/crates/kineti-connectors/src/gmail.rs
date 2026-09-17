//! Google Workspace & Gmail REST API Connector.
//!
//! Provides inbox search, thread summarization, and outbound draft creation
//! using standard Google OAuth 2.0 access tokens.

/// An email message summary retrieved from Gmail.
#[derive(Debug, Clone, PartialEq)]
pub struct EmailSummary {
    /// Gmail message ID.
    pub id: String,
    /// Thread ID.
    pub thread_id: String,
    /// Sender address and name.
    pub from: String,
    /// Email subject line.
    pub subject: String,
    /// Received timestamp (Unix ms).
    pub internal_date_ms: u64,
    /// Short preview snippet.
    pub snippet: String,
}

/// Draft message parameters.
#[derive(Debug, Clone)]
pub struct DraftEmailRequest {
    /// Recipient email address.
    pub to: String,
    /// Email subject.
    pub subject: String,
    /// Email body (plain text).
    pub body: String,
    /// Optional thread ID to reply in-thread.
    pub thread_id: Option<String>,
}

/// Gmail API Connector.
#[derive(Debug, Clone)]
pub struct GmailClient {
    access_token: String,
}

impl GmailClient {
    /// Creates a new Gmail client with the user's OAuth access token.
    pub fn new(access_token: impl Into<String>) -> Self {
        Self {
            access_token: access_token.into(),
        }
    }

    /// Builds a search request URL and headers.
    pub fn build_search_request(&self, query: &str, max_results: u32) -> (String, Vec<(&'static str, String)>) {
        let max_results = max_results.clamp(1, 10);
        let encoded_q = query.replace(' ', "+");
        let url = format!(
            "https://gmail.googleapis.com/gmail/v1/users/me/messages?q={}&maxResults={}",
            encoded_q, max_results
        );
        let headers = vec![
            ("Authorization", format!("Bearer {}", self.access_token)),
            ("Accept", "application/json".to_string()),
        ];
        (url, headers)
    }

    /// Builds the raw MIME draft payload.
    pub fn build_draft_payload(&self, draft: &DraftEmailRequest) -> String {
        let raw_mime = format!(
            "To: {}\r\nSubject: {}\r\nContent-Type: text/plain; charset=UTF-8\r\n\r\n{}",
            draft.to, draft.subject, draft.body
        );
        let encoded = base64_url_encode(raw_mime.as_bytes());
        format!("{{\"message\":{{\"raw\":\"{}\"}}}}", encoded)
    }

    /// Executes a live inbox search against Gmail REST API.
    pub fn search_inbox_live(&self, query: &str, max_results: u32) -> Result<String, String> {
        let (url, headers) = self.build_search_request(query, max_results);
        let header_refs: Vec<(&str, &str)> = headers.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let res = kineti_core::http_get(&url, &header_refs)
            .map_err(|e| format!("Gmail API network error: {}", e))?;
        if !res.success {
            return Err(format!("Gmail API HTTP error {}: {}", res.status, res.body));
        }
        Ok(res.body)
    }

    /// Creates a draft directly via Gmail REST API.
    pub fn create_draft_live(&self, draft: &DraftEmailRequest) -> Result<String, String> {
        let url = "https://gmail.googleapis.com/gmail/v1/users/me/drafts";
        let auth_val = format!("Bearer {}", self.access_token);
        let header_refs = [
            ("Authorization", auth_val.as_str()),
            ("Content-Type", "application/json"),
        ];
        let payload = self.build_draft_payload(draft);
        let res = kineti_core::http_post_json(url, &header_refs, &payload)
            .map_err(|e| format!("Gmail create draft network error: {}", e))?;
        if !res.success {
            return Err(format!("Gmail create draft error {}: {}", res.status, res.body));
        }
        Ok(res.body)
    }

    /// Sends an email directly via Gmail REST API.
    pub fn send_email_live(&self, draft: &DraftEmailRequest) -> Result<String, String> {
        let url = "https://gmail.googleapis.com/gmail/v1/users/me/messages/send";
        let auth_val = format!("Bearer {}", self.access_token);
        let header_refs = [
            ("Authorization", auth_val.as_str()),
            ("Content-Type", "application/json"),
        ];
        let payload = self.build_draft_payload(draft);
        let res = kineti_core::http_post_json(url, &header_refs, &payload)
            .map_err(|e| format!("Gmail send email network error: {}", e))?;
        if !res.success {
            return Err(format!("Gmail send email error {}: {}", res.status, res.body));
        }
        Ok(res.body)
    }
}

fn base64_url_encode(data: &[u8]) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::new();
    let mut i = 0;
    while i < data.len() {
        let b0 = data[i] as usize;
        let b1 = if i + 1 < data.len() { data[i + 1] as usize } else { 0 };
        let b2 = if i + 2 < data.len() { data[i + 2] as usize } else { 0 };

        out.push(TABLE[(b0 >> 2) & 0x3F] as char);
        out.push(TABLE[((b0 << 4) | (b1 >> 4)) & 0x3F] as char);
        if i + 1 < data.len() {
            out.push(TABLE[((b1 << 2) | (b2 >> 6)) & 0x3F] as char);
        }
        if i + 2 < data.len() {
            out.push(TABLE[b2 & 0x3F] as char);
        }
        i += 3;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gmail_search_request_builder() {
        let client = GmailClient::new("ya29.test_token");
        let (url, headers) = client.build_search_request("from:sarah budget", 5);

        assert!(url.contains("q=from:sarah+budget"));
        assert_eq!(headers[0].0, "Authorization");
        assert_eq!(headers[0].1, "Bearer ya29.test_token");
    }

    #[test]
    fn test_gmail_draft_payload_builder() {
        let client = GmailClient::new("ya29.test_token");
        let req = DraftEmailRequest {
            to: "sarah@company.com".to_string(),
            subject: "Budget Proposal Review".to_string(),
            body: "Looks good, let's proceed.".to_string(),
            thread_id: None,
        };
        let payload = client.build_draft_payload(&req);
        assert!(payload.contains("\"message\":{\"raw\":"));
    }
}

//! Real HTTP execution engine using standard system curl.
//!
//! Enforces user-agent "OpenAI File Downloader, XaiImageApiFetch/1.0", timeout,
//! status code capture, and zero-external-dependency execution.

use std::process::Command;

/// Represents an outgoing HTTP request.
#[derive(Debug, Clone)]
pub struct HttpRequest {
    /// HTTP method (GET, POST, PATCH, DELETE, PUT).
    pub method: String,
    /// Destination URL.
    pub url: String,
    /// Header key-value pairs.
    pub headers: Vec<(String, String)>,
    /// Optional request body string.
    pub body: Option<String>,
    /// Max execution time in seconds.
    pub timeout_secs: u64,
}

/// Represents the response received from an HTTP execution.
#[derive(Debug, Clone)]
pub struct HttpResponse {
    /// HTTP numeric status code (e.g. 200, 201, 400, 404).
    pub status: u16,
    /// Response body text.
    pub body: String,
    /// Whether status is in 200..=299 range.
    pub success: bool,
}

/// Standard User-Agent header enforced across all curl calls.
pub const KINETI_USER_AGENT: &str = "OpenAI File Downloader, XaiImageApiFetch/1.0";

/// Executes an HTTP request using system curl and returns the status and response body.
pub fn execute_http(req: &HttpRequest) -> Result<HttpResponse, String> {
    let mut cmd = Command::new("curl");
    cmd.arg("-s"); // Silent mode
    cmd.arg("-S"); // Show error message if it fails
    cmd.arg("-w").arg("\n%{http_code}"); // Write HTTP status code on trailing line
    cmd.arg("-X").arg(&req.method);
    cmd.arg("-A").arg(KINETI_USER_AGENT);
    cmd.arg("--max-time").arg(req.timeout_secs.max(1).to_string());

    for (k, v) in &req.headers {
        cmd.arg("-H").arg(format!("{}: {}", k, v));
    }

    if let Some(ref body) = req.body {
        cmd.arg("--data-raw").arg(body);
    }

    cmd.arg(&req.url);

    let output = cmd.output().map_err(|e| format!("Failed to execute curl: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("curl failed with code {:?}: {}", output.status.code(), stderr.trim()));
    }

    let raw_out = String::from_utf8_lossy(&output.stdout);
    if let Some(pos) = raw_out.rfind('\n') {
        let (body, status_str) = raw_out.split_at(pos);
        let status_num = status_str.trim().parse::<u16>().unwrap_or(0);
        Ok(HttpResponse {
            status: status_num,
            body: body.to_string(),
            success: (200..=299).contains(&status_num),
        })
    } else {
        let status_num = raw_out.trim().parse::<u16>().unwrap_or(0);
        Ok(HttpResponse {
            status: status_num,
            body: String::new(),
            success: (200..=299).contains(&status_num),
        })
    }
}

/// Helper for HTTP GET requests.
pub fn http_get(url: &str, headers: &[(&str, &str)]) -> Result<HttpResponse, String> {
    let req = HttpRequest {
        method: "GET".to_string(),
        url: url.to_string(),
        headers: headers.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
        body: None,
        timeout_secs: 15,
    };
    execute_http(&req)
}

/// Helper for HTTP POST JSON requests.
pub fn http_post_json(url: &str, headers: &[(&str, &str)], json_body: &str) -> Result<HttpResponse, String> {
    let mut hdrs: Vec<(String, String)> = headers.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
    if !hdrs.iter().any(|(k, _)| k.eq_ignore_ascii_case("content-type")) {
        hdrs.push(("Content-Type".to_string(), "application/json".to_string()));
    }
    let req = HttpRequest {
        method: "POST".to_string(),
        url: url.to_string(),
        headers: hdrs,
        body: Some(json_body.to_string()),
        timeout_secs: 15,
    };
    execute_http(&req)
}

/// Helper for HTTP PATCH JSON requests.
pub fn http_patch_json(url: &str, headers: &[(&str, &str)], json_body: &str) -> Result<HttpResponse, String> {
    let mut hdrs: Vec<(String, String)> = headers.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
    if !hdrs.iter().any(|(k, _)| k.eq_ignore_ascii_case("content-type")) {
        hdrs.push(("Content-Type".to_string(), "application/json".to_string()));
    }
    let req = HttpRequest {
        method: "PATCH".to_string(),
        url: url.to_string(),
        headers: hdrs,
        body: Some(json_body.to_string()),
        timeout_secs: 15,
    };
    execute_http(&req)
}

/// Helper for HTTP DELETE requests.
pub fn http_delete(url: &str, headers: &[(&str, &str)]) -> Result<HttpResponse, String> {
    let req = HttpRequest {
        method: "DELETE".to_string(),
        url: url.to_string(),
        headers: headers.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
        body: None,
        timeout_secs: 15,
    };
    execute_http(&req)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_agent_format() {
        assert_eq!(KINETI_USER_AGENT, "OpenAI File Downloader, XaiImageApiFetch/1.0");
    }
}

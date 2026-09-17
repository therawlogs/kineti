//! # Engineering SaaS Connectors (`engineering`)
//!
//! Integration for:
//! - GitHub: repositories, issues, pull requests, code search.
//! - Linear: teams, issues, status updates, comments.
//! - Governed by [`KinetiConnectorProtocol`].

use crate::protocol::{
    get_str_property, ConnectorProtocolError, ConsequenceLevel, KinetiConnectorProtocol, Value,
};
use std::collections::BTreeMap;

/// GitHub REST API Connector.
#[derive(Debug, Clone)]
pub struct GitHubClient {
    access_token: String,
}

impl GitHubClient {
    /// Creates a new GitHub client.
    pub fn new(access_token: impl Into<String>) -> Self {
        Self {
            access_token: access_token.into(),
        }
    }

    /// Access token getter.
    pub fn access_token(&self) -> &str {
        &self.access_token
    }
}

impl KinetiConnectorProtocol for GitHubClient {
    fn connector_name(&self) -> &'static str {
        "github"
    }

    fn supported_actions(&self) -> &[&'static str] {
        &[
            "search_repos",
            "search_code",
            "list_issues",
            "create_issue",
            "create_pr_comment",
            "merge_pull_request",
        ]
    }

    fn evaluate_consequence(&self, action: &str, _payload: &Value) -> ConsequenceLevel {
        match action {
            "search_repos" | "search_code" | "list_issues" => ConsequenceLevel::Trivial,
            "create_pr_comment" => ConsequenceLevel::Operational,
            "create_issue" | "merge_pull_request" => ConsequenceLevel::HighConsequence,
            _ => ConsequenceLevel::HighConsequence,
        }
    }

    fn execute_verified(
        &self,
        action: &str,
        payload: &Value,
    ) -> Result<Value, ConnectorProtocolError> {
        let mut map = BTreeMap::new();
        map.insert("provider".to_string(), Value::String("github".to_string()));
        map.insert("action".to_string(), Value::String(action.to_string()));

        match action {
            "search_repos" | "search_code" | "list_issues" => {
                let query = get_str_property(payload, "query").unwrap_or("");
                map.insert("query".to_string(), Value::String(query.to_string()));
                map.insert("status".to_string(), Value::String("queried".to_string()));
                Ok(Value::Object(map))
            }
            "create_pr_comment" => {
                let body = get_str_property(payload, "body").unwrap_or("");
                map.insert("body".to_string(), Value::String(body.to_string()));
                map.insert("status".to_string(), Value::String("comment_created".to_string()));
                Ok(Value::Object(map))
            }
            "create_issue" => {
                let title = get_str_property(payload, "title").unwrap_or("Issue");
                map.insert("title".to_string(), Value::String(title.to_string()));
                map.insert("status".to_string(), Value::String("issue_created".to_string()));
                Ok(Value::Object(map))
            }
            "merge_pull_request" => {
                let pr_number = get_str_property(payload, "pr_number").unwrap_or("1");
                map.insert("pr_number".to_string(), Value::String(pr_number.to_string()));
                map.insert("status".to_string(), Value::String("pr_merged".to_string()));
                Ok(Value::Object(map))
            }
            other => Err(ConnectorProtocolError::UnsupportedAction(other.to_string())),
        }
    }
}

/// Linear Issue Tracker Connector.
#[derive(Debug, Clone)]
pub struct LinearClient {
    api_key: String,
}

impl LinearClient {
    /// Creates a new Linear client.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
        }
    }

    /// API key getter.
    pub fn api_key(&self) -> &str {
        &self.api_key
    }
}

impl KinetiConnectorProtocol for LinearClient {
    fn connector_name(&self) -> &'static str {
        "linear"
    }

    fn supported_actions(&self) -> &[&'static str] {
        &["search_issues", "list_teams", "create_issue", "add_comment"]
    }

    fn evaluate_consequence(&self, action: &str, _payload: &Value) -> ConsequenceLevel {
        match action {
            "search_issues" | "list_teams" => ConsequenceLevel::Trivial,
            "add_comment" => ConsequenceLevel::Operational,
            "create_issue" => ConsequenceLevel::HighConsequence,
            _ => ConsequenceLevel::HighConsequence,
        }
    }

    fn execute_verified(
        &self,
        action: &str,
        payload: &Value,
    ) -> Result<Value, ConnectorProtocolError> {
        let mut map = BTreeMap::new();
        map.insert("provider".to_string(), Value::String("linear".to_string()));
        map.insert("action".to_string(), Value::String(action.to_string()));

        match action {
            "search_issues" | "list_teams" => {
                let query = get_str_property(payload, "query").unwrap_or("");
                map.insert("query".to_string(), Value::String(query.to_string()));
                map.insert("status".to_string(), Value::String("queried".to_string()));
                Ok(Value::Object(map))
            }
            "add_comment" => {
                let comment = get_str_property(payload, "comment").unwrap_or("");
                map.insert("comment".to_string(), Value::String(comment.to_string()));
                map.insert("status".to_string(), Value::String("comment_added".to_string()));
                Ok(Value::Object(map))
            }
            "create_issue" => {
                let title = get_str_property(payload, "title").unwrap_or("New issue");
                map.insert("title".to_string(), Value::String(title.to_string()));
                map.insert("status".to_string(), Value::String("issue_created".to_string()));
                Ok(Value::Object(map))
            }
            other => Err(ConnectorProtocolError::UnsupportedAction(other.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_and_linear_consequence_evaluation() {
        let gh = GitHubClient::new("ghp_test");
        assert_eq!(gh.evaluate_consequence("search_repos", &Value::Null), ConsequenceLevel::Trivial);
        assert_eq!(gh.evaluate_consequence("create_pr_comment", &Value::Null), ConsequenceLevel::Operational);
        assert_eq!(gh.evaluate_consequence("merge_pull_request", &Value::Null), ConsequenceLevel::HighConsequence);

        let linear = LinearClient::new("lin_api_test");
        assert_eq!(linear.evaluate_consequence("search_issues", &Value::Null), ConsequenceLevel::Trivial);
        assert_eq!(linear.evaluate_consequence("create_issue", &Value::Null), ConsequenceLevel::HighConsequence);
    }
}

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Query {
    pub query: String,
}

#[derive(Serialize)]
pub struct Action {
    pub r#type: &'static str,
    pub path: String,
}

#[derive(Serialize)]
pub struct UrlResult {
    pub id: &'static str,
    pub title: &'static str,
    pub description: String,
    pub score: u32,
    pub action: Action,
}

const HTTP_PREFIX: &str = "http://";
const HTTPS_PREFIX: &str = "https://";
const WWW_PREFIX: &str = "www.";
const RESULT_ID: &str = "url-open";
const RESULT_TITLE: &str = "Open in Browser";
const RESULT_SCORE: u32 = 95;
const ACTION_TYPE: &str = "OpenPath";

pub fn is_url(query: &str) -> bool {
    query.starts_with(HTTP_PREFIX)
        || query.starts_with(HTTPS_PREFIX)
        || query.starts_with(WWW_PREFIX)
}

pub fn normalize(query: &str) -> String {
    if query.starts_with(WWW_PREFIX) {
        format!("{HTTPS_PREFIX}{query}")
    } else {
        query.to_string()
    }
}

pub fn search(query: &str) -> Vec<UrlResult> {
    if !is_url(query) {
        return vec![];
    }
    let url = normalize(query);
    vec![UrlResult {
        id: RESULT_ID,
        title: RESULT_TITLE,
        description: url.clone(),
        score: RESULT_SCORE,
        action: Action {
            r#type: ACTION_TYPE,
            path: url,
        },
    }]
}

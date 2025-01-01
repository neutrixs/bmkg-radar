use reqwest::RequestBuilder;
use std::env;
use url::{ParseError, Url};

pub fn auto_proxy(client: reqwest::Client, full_url: &str) -> Result<RequestBuilder, ParseError> {
    let (proxy_url, use_proxy) = match env::var("PROXY_URL") {
        Ok(content) if !content.is_empty() => (content, true),
        _ => ("".to_string(), false),
    };

    let url = match use_proxy {
        true => {
            let mut proxy = Url::parse(&proxy_url)?;
            proxy.query_pairs_mut().append_pair("url", full_url);

            proxy.to_string()
        }
        false => full_url.to_string(),
    };

    Ok(client.get(&url))
}
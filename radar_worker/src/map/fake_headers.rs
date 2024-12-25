use reqwest::header::{HeaderMap, HeaderValue};

pub(crate) fn _fake_headers() -> HeaderMap {
    let mut fake_headers = HeaderMap::new();

    fake_headers.insert(
        "User-Agent",
        HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
        AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36",
        ),
    );

    fake_headers.insert(
        "Referer",
        HeaderValue::from_static("https://www.openstreetmap.org/"),
    );

    fake_headers.insert(
        "Accept",
        HeaderValue::from_static("application/json,text/html, image/svg+xml, image/*,*/*;q=0.8"),
    );

    fake_headers.insert(
        "Accept-Encoding",
        HeaderValue::from_static("gzip, deflate, br, zstd"),
    );

    fake_headers.insert(
        "Sec-Ch-Ua",
        HeaderValue::from_static(
            "\"Google Chrome\";v=\"123\", \"Not:A-Brand\";v=\"8\", \"Chromium\";v=\"123\"",
        ),
    );

    fake_headers
}

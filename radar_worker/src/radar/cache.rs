use crate::radar::radar_data::{get_radar_data, RawAPIRadarlist};
use crate::radar::Image;
use chrono::{DateTime, Duration, Utc};
use std::error::Error;
use std::time::Duration as StdDuration;

const CACHE_EXPIRE_MINS: i64 = 3;

pub(crate) struct ImageryCache {
    last_fetch: DateTime<Utc>,
    data: Vec<Image>,
    // this only contains radar metadata
    // so keep this forever
    raw_api: RawAPIRadarlist,
}

pub(crate) async fn imagery_cache_builder(timeout: StdDuration) -> Result<ImageryCache, Box<dyn Error +
Send + Sync>> {
    let mut raw_api: Option<RawAPIRadarlist> = None;
    let mut error: Option<Box<dyn Error + Send + Sync>> = None;
    // try to fetch 10 times, if it fails
    // yeah it's over
    for _ in 0..10 {
        match get_radar_data(timeout).await {
            Ok(data) => {
                raw_api = Some(data);
                break;
            }
            Err(e) => error = Some(e),
        }
    }

    if let Some(e) = error {
        return Err(e);
    }
    let raw_api = raw_api.unwrap();
    let last_fetch = Utc::now();

    Ok(ImageryCache {
        raw_api,
        last_fetch,
        data: vec![],
    })
}

impl ImageryCache {
    pub fn get_data(&self, code: String) -> Result<&Image, Box<dyn Error + Send + Sync>> {
        let elapsed = Utc::now() - self.last_fetch;
        let expired = elapsed > Duration::minutes(CACHE_EXPIRE_MINS);
        if let Some(data) = self.data.iter().find(|d| { d.data.code == code }) {
            if !expired { return Ok(data); }
        }
        Err(format!("Radar code {} does not exist", code).into())
    }
}
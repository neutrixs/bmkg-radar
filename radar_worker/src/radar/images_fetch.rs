use crate::radar::{Image, ImageCache, RadarData, RadarImagery};
use crate::util::{auto_proxy, gen_connection_err};
use futures::StreamExt;
use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::time::Duration as StdDuration;

// for now, we default to latest image
// TODO: don't clone the image, if even possible, likely not
async fn process_fetch(radar: RadarData, cache: &mut HashMap<String, VecDeque<ImageCache>>,
                       timeout: StdDuration) -> Result<Option<Image>, Box<dyn Error + Send +
Sync>> {
    let newest = radar.images.last();
    if let None = newest {
        return Ok(None);
    }
    let newest = newest.unwrap();
    let newest_date = newest.time;

    if let Some(radar_cache) = cache.get(&radar.code) {
        if let Some(image_data) = radar_cache.iter().find(|r| r.date == newest_date) {
            return Ok(Some(Image {
                image: image_data.image.clone(),
                data: radar,
            }));
        }
    } else {
        cache.insert(radar.code.clone(), VecDeque::new());
    }

    // now, the cache entry for the current radar will exist
    // no matter what
    let radar_cache = cache.get_mut(&radar.code).unwrap();

    let client = reqwest::Client::builder()
        .timeout(timeout)
        .build()?;

    let response = auto_proxy(client, &newest.url)?.send().await;

    if let Err(e) = response {
        return Err(gen_connection_err(e));
    }
    // if let Err(e) = response {
    //     return Err(format!("Error while fetching radar image {}: {}", data.code, e).into());
    // }
    let response = response.unwrap();
    let bytes = response.bytes().await;
    if let Err(e) = bytes {
        return Err(format!("Error on parsing bytes while fetching radar image {}: {}", &radar.code, e).into());
    }
    let bytes = bytes.unwrap();

    let result = Image {
        data: radar,
        image: bytes,
    };

    radar_cache.push_back(ImageCache {
        date: newest_date,
        image: result.image.clone(),
    });

    Ok(Some(result))
}

impl RadarImagery {
    pub(crate) async fn fetch_images(&mut self, radars: Vec<RadarData>) -> Result<Vec<Image>,
        Box<dyn Error + Send + Sync>> {
        let mut result = Vec::new();

        // TODO: concurrency, with rayon maybe
        for radar in radars {
            if let Some(data) = process_fetch(radar, &mut self.cached_images, self.timeout_duration).await? {
                result.push(data);
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::common::Coordinate;
    use crate::radar::RadarImagery;

    #[tokio::test]
    async fn test_fetch() {
        let bounds = [Coordinate { lat: 0., lon: 0. }, Coordinate { lat: 0., lon: 0. }];
        let mut radar_imagery = RadarImagery::builder().build();
        let radars = radar_imagery.get_radar_data().await.unwrap();

        let images = radar_imagery.fetch_images(radars).await;
        if let Err(e) = &images {
            eprintln!("{}", e);
            assert!(false);
        }
        assert!(true);
    }
}
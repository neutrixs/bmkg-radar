use crate::radar::{Image, RadarData, RadarImagery};
use crate::util::{auto_proxy, gen_connection_err};
use futures::future;
use std::error::Error;

impl RadarImagery {
    pub async fn fetch_images(&self, radars: Vec<RadarData>) -> Result<Vec<Image>, Box<dyn Error +
    Send + Sync>> {
        let mut radars_to_be_used = Vec::new();
        let mut result = Vec::new();
        let mut async_requests = Vec::new();

        for radar in radars {
            let latest_image_url = radar.images.last();
            // this would never happen, actually
            // however, if it does, we can just skip it
            if let None = latest_image_url {
                continue;
            }
            let latest_image_url = latest_image_url.unwrap().url.clone();

            let client = reqwest::Client::builder()
                .timeout(self.timeout_duration)
                .build()?;

            let request = auto_proxy(client, &latest_image_url)?.send();
            async_requests.push(request);
            radars_to_be_used.push(radar);
        }

        let images = future::join_all(async_requests).await;
        let mut radars_iter = radars_to_be_used.into_iter();
        let mut images_iter = images.into_iter();

        while let (Some(data), Some(response)) = (radars_iter.next(), images_iter.next()) {
            if let Err(e) = response {
                return Err(gen_connection_err(e));
            }
            // if let Err(e) = response {
            //     return Err(format!("Error while fetching radar image {}: {}", data.code, e).into());
            // }
            let response = response.unwrap();
            let bytes = response.bytes().await;
            if let Err(e) = bytes {
                return Err(format!("Error on parsing bytes while fetching radar image {}: {}", data
                    .code, e).into());
            }
            let bytes = bytes.unwrap();

            result.push(Image {
                data,
                image: bytes,
            })
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
        let radar_imagery = RadarImagery::builder(bounds).build();
        let radars = radar_imagery.get_radar_data().await.unwrap();

        let images = radar_imagery.fetch_images(radars).await;
        if let Err(e) = &images {
            eprintln!("{}", e);
            assert!(false);
        }
        assert!(true);
    }
}
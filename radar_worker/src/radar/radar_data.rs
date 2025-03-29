use crate::common::{Coordinate, Distance};
use crate::radar::color_scheme::hex_to_rgb;
use crate::radar::{RadarData, RadarImagesData, DEFAULT_RANGE};
use crate::radar::{RadarImagery, DEFAULT_PRIORITY};
use crate::util::{auto_proxy, gen_connection_err};
use chrono::{NaiveDateTime, TimeZone, Utc};
use image::Rgba;
use serde::Deserialize;
use std::env;
use std::error::Error;
use time::Duration;
use url::Url;

const RADAR_LIST_API_URL: &str = "https://radar.bmkg.go.id:8090/radarlist";
const RADAR_DETAIL_API_URL: &str = "https://radar.bmkg.go.id:8090/sidarmaimage";
const RADAR_DETAIL_API_URL_NO_TOKEN: &str = "https://api-apps.bmkg.go.id/api/radar-image";

#[derive(Deserialize, Debug)]
struct RawAPIRadar {
    // unprofessional API!
    // unacceptable!!
    #[serde(rename = "overlayTLC")]
    overlay_tlc: Vec<String>,
    #[serde(rename = "overlayBRC")]
    overlay_brc: Vec<String>,
    // #[serde(rename = "_id")]
    // id: String,
    #[serde(rename = "Kota")]
    city: String,
    #[serde(rename = "Stasiun")]
    station: String,
    #[serde(rename = "kode")]
    code: String,
    lat: f64,
    lon: f64,
}

#[derive(Deserialize, Debug)]
struct RawAPIRadarlist {
    // success: bool,
    // message: String,
    #[serde(rename = "datas")]
    data: Vec<RawAPIRadar>,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct APILegends {
    pub(crate) levels: Vec<i32>,
    pub(crate) colors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Legends {
    pub levels: Vec<i32>,
    pub colors: Vec<Rgba<u8>>,
}

// #[derive(Deserialize, Debug)]
// struct APILatest {
//     #[serde(rename = "timeUTC")]
//     time_utc: String,
//     file: String,
// }

#[derive(Deserialize, Debug)]
struct LastOneHour {
    #[serde(rename = "timeUTC")]
    time_utc: Vec<String>,
    file: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct RawAPIDetailedData {
    // #[serde(rename = "changeStatus")]
    // change_status: String,
    legends: APILegends,
    // #[serde(rename = "Latest")]
    // latest: APILatest,
    #[serde(rename = "LastOneHour")]
    last_one_hour: LastOneHour,
}

fn parse_legend(from: APILegends) -> Legends {
    let colors = from.colors;
    let levels = from.levels;
    let mut colors_new = Vec::new();

    for color in colors {
        let parsed = hex_to_rgb(&color).unwrap_or_else(|_| Rgba([0, 0, 0, 0]));
        colors_new.push(parsed);
    }

    Legends {
        levels,
        colors: colors_new,
    }
}

impl RadarImagery {
    fn is_overlapping(&self, range: &[Coordinate; 2]) -> bool {
        let x = range;
        let y = &self.bounds;

        let x_start = &x[0];
        let y_start = &y[0];
        let x_end = &x[1];
        let y_end = &y[1];

        let lat_overlap = x_start.lat.min(y_start.lat) > x_end.lat.max(y_end.lat);
        let lon_overlap = x_start.lon.max(y_start.lon) < x_end.lon.min(y_end.lon);

        lat_overlap && lon_overlap
    }

    async fn get_radar_images_data(&self, radar: &RawAPIRadar) -> Result<(Vec<RadarImagesData>,
                                                                          Legends),
        Box<dyn
        Error + Send + Sync>> {
        let token = env::var("BMKG_APIKEY");
        let base_url: String;
        if let Ok(_) = token {
            base_url = String::from(RADAR_DETAIL_API_URL);
        } else {
            base_url = String::from(RADAR_DETAIL_API_URL_NO_TOKEN);
        }

        let mut url = Url::parse(&base_url)?;
        url.query_pairs_mut()
            .append_pair("radar", &radar.code)
            .append_pair("token", &token.unwrap_or("".to_string()));

        // well well well
        // who's got an invalid cert here??
        // anyway I have to do the same in cURL too so... yeah...
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(self.timeout_duration)
            .build()?;

        let response = auto_proxy(client, &url.to_string())?.send().await?.text().await?;
        let response: RawAPIDetailedData = serde_json::from_str(&response)?;
        let recent = response.last_one_hour;
        let legends = parse_legend(response.legends);

        if recent.time_utc.len() != recent.file.len() {
            return Err("Broken API! time_utc.len() is NOT the same as file.len()!".into());
        }

        let mut images: Vec<RadarImagesData> = Vec::new();
        for i in 0..recent.file.len() {
            let url = &recent.file[i];
            let time = &recent.time_utc[i];
            let time = NaiveDateTime::parse_from_str(time, "%Y-%m-%d %H:%M UTC");
            if let Err(_) = time {
                // you would think that when there are no images,
                // the API would return an empty array
                // BUT NOPE, WRONG!
                // it still returns array of 5 strings with the content "No Data"
                // ?????????
                // mark as UNPROFESSIONAL API
                // let's not bother
                continue;
            }
            let time = time.unwrap();
            let time = Utc.from_utc_datetime(&time);

            images.push(RadarImagesData {
                time,
                url: url.clone(),
            })
        }

        Ok((images, legends))
    }

    pub(crate) async fn get_radar_data(&self) -> Result<Vec<RadarData>, Box<dyn Error + Send + Sync>> {
        let mut container: Vec<RadarData> = Vec::new();
        // well well well
        // who's got an invalid cert here??
        // anyway I have to do the same in cURL too so... yeah...
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(self.timeout_duration)
            .build()?;

        let response = auto_proxy(client, RADAR_LIST_API_URL)?.send().await;
        if let Err(e) = response {
            return Err(gen_connection_err(e));
        }

        let response = response.unwrap().text().await?;
        let response: RawAPIRadarlist = serde_json::from_str(&response)?;

        for radar in response.data {
            if radar.overlay_tlc.len() < 2 {
                return Err(format!(
                    "overlayTLC returned invalid length: {}. Expected: 2",
                    radar.overlay_tlc.len()
                )
                    .into());
            }
            let start = Coordinate {
                lat: radar.overlay_tlc[0].parse()?,
                lon: radar.overlay_tlc[1].parse()?,
            };
            if radar.overlay_brc.len() < 2 {
                return Err(format!(
                    "overlayBRC returned invalid length: {}. Expected: 2",
                    radar.overlay_brc.len()
                )
                    .into());
            }
            let end = Coordinate {
                lat: radar.overlay_brc[0].parse()?,
                lon: radar.overlay_brc[1].parse()?,
            };

            if !self.is_overlapping(&[start, end]) {
                continue;
            }

            let is_omitted_radar = self.omit_radar.iter().position(|r| r == &radar.code);
            if let Some(_) = is_omitted_radar {
                continue;
            }

            let (images, legends) = self.get_radar_images_data(&radar).await?;

            if images.len() == 0 {
                continue;
            }

            let mut priority: i32;
            let range: Distance;

            if let Some(p) = self.priorities.get(&radar.code) {
                priority = *p;
            } else {
                priority = DEFAULT_PRIORITY;
            }

            if let Some(r) = self.ranges.get(&radar.code) {
                range = *r;
            } else {
                range = DEFAULT_RANGE;
            }

            let image_time = images.last().unwrap().time;
            let elapsed = Utc::now() - image_time;
            let elapsed = Duration::seconds(elapsed.num_seconds());
            let mut striped = false;

            if elapsed > self.age_threshold && self.enforce_age_threshold {
                striped = true;
                priority = -1;
            }

            // // DEBUG ONLY
            // let mut images = images;
            // if radar.code == "PWK" {
            //     priority = -1;
            //     striped = true;
            //     images[5].url = String::from("https://fs.neutrixs.my.id/PWK_TEST.png");
            // }
            // if radar.code == "JAK" {
            //     images[5].url = String::from("https://fs.neutrixs.my.id/JAK_TEST.png");
            // }

            let formatted = RadarData {
                bounds: [start, end],
                city: radar.city,
                code: radar.code,
                station: radar.station,
                center: Coordinate {
                    lat: radar.lat,
                    lon: radar.lon,
                },
                priority,
                striped,
                range,
                images,
                legends,
            };

            container.push(formatted);
        }

        Ok(container)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_radar_data() {
        let bounds = [
            Coordinate {
                lat: -6.0404882,
                lon: 106.618351,
            },
            Coordinate {
                lat: -6.4975978,
                lon: 107.144467,
            }
        ];
        let im = RadarImagery::builder(bounds).build();
        let result = im.get_radar_data().await;
        if let Err(e) = result {
            panic!("{}", e);
        }
        let result = result.unwrap();

        assert_ne!(result.len(), 0);
    }

    #[test]
    fn test_overlap() {
        let reference = [
            Coordinate {
                lat: -6.0404882,
                lon: 106.618351,
            },
            Coordinate {
                lat: -6.4975978,
                lon: 107.144467,
            }
        ];
        let bounds1 = [
            Coordinate {
                lat: -6.1436955,
                lon: 106.4568831,
            },
            Coordinate {
                lat: -6.4405944,
                lon: 106.9040865,
            }
        ];
        let bounds2 = [
            Coordinate {
                lat: -6.4055851,
                lon: 107.4918877,
            },
            Coordinate {
                lat: -6.5101027,
                lon: 107.55644,
            }
        ];

        let im = RadarImagery::builder(reference).build();
        assert!(im.is_overlapping(&bounds1));
        assert!(!im.is_overlapping(&bounds2));
    }
}

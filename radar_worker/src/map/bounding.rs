use crate::common::Coordinate;
use crate::map::fake_headers::_fake_headers;
use crate::util::gen_connection_err;
use serde::Deserialize;
use std::error::Error;
use std::time::Duration;
use url::form_urlencoded::Serializer;

const NOMINATIM_SEARCH_BASE_URL: &str = "https://nominatim.openstreetmap.org/search";

//noinspection SpellCheckingInspection
#[derive(Deserialize, Debug)]
struct APISearchResult {
    pub place_id: u64,
    pub osm_type: String,
    pub osm_id: u64,
    pub lat: String,
    pub lon: String,
    pub class: String,
    #[serde(rename = "type")]
    pub typ: String,
    pub place_rank: i32,
    pub importance: f32,
    #[serde(rename = "addresstype")]
    pub address_type: String,
    pub name: String,
    pub display_name: String,
    #[serde(rename = "boundingbox")]
    pub bounding_box: [String; 4],
}

impl APISearchResult {
    fn into_api_result(self) -> Result<APIResult, Box<dyn Error + Send + Sync>> {
        let lat = self.lat.parse::<f64>()?;
        let lon = self.lon.parse::<f64>()?;

        let bounding0_lat = self.bounding_box[1].parse::<f64>()?;
        let bounding0_lon = self.bounding_box[2].parse::<f64>()?;
        let bounding1_lat = self.bounding_box[0].parse::<f64>()?;
        let bounding1_lon = self.bounding_box[3].parse::<f64>()?;

        Ok(APIResult {
            place_id: self.place_id,
            osm_type: self.osm_type,
            osm_id: self.osm_id,
            coordinate: Coordinate { lat, lon },
            class: self.class,
            place_type: self.typ,
            place_rank: self.place_rank,
            importance: self.importance,
            address_type: self.address_type,
            name: self.name,
            display_name: self.display_name,
            bounding_box: [
                Coordinate {
                    lat: bounding0_lat,
                    lon: bounding0_lon,
                },
                Coordinate {
                    lat: bounding1_lat,
                    lon: bounding1_lon,
                },
            ],
        })
    }
}

#[derive(Debug)]
pub(crate) struct APIResult {
    pub place_id: u64,
    pub osm_type: String,
    pub osm_id: u64,
    pub coordinate: Coordinate,
    pub class: String,
    pub place_type: String,
    pub place_rank: i32,
    pub importance: f32,
    pub address_type: String,
    pub name: String,
    pub display_name: String,
    pub bounding_box: [Coordinate; 2],
}

pub(crate) async fn search(place: &String) -> Result<Vec<APIResult>, Box<dyn Error + Send + Sync>> {
    let escaped = Serializer::new(String::new())
        .append_pair("q", place.as_str())
        .append_pair("format", "json")
        .finish();
    let fake_headers = _fake_headers();

    let url = format!("{}?{}", NOMINATIM_SEARCH_BASE_URL, escaped);
    let response = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?
        .get(&url)
        .headers(fake_headers)
        .send()
        .await;
    if let Err(e) = response {
        return Err(gen_connection_err(e));
    }
    let response = response.unwrap().text().await?;
    let api_data: Vec<APISearchResult> = serde_json::from_str(&response)?;
    let mut result: Vec<APIResult> = vec![];

    for item in api_data.into_iter() {
        result.push(item.into_api_result()?);
    }

    Ok(result)
}

pub async fn bounding_box(place: String) -> Result<[Coordinate; 2], Box<dyn Error + Send + Sync>> {
    let result = search(&place).await?;

    if result.len() == 0 {
        let message = format!("No such location {}", &place);
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, message)));
    }

    Ok(result[0].bounding_box)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    #[tokio::test]
    async fn test_bounding_box() {
        let bound = bounding_box(String::from("Bandung")).await;
        let bound = bound.unwrap();
        // checks for mix up between the first and the second bound
        assert!(bound[0].lat > bound[1].lat);
        assert!(bound[0].lon < bound[1].lon);
        // checks for mix up between lat and lon
        assert!(bound[0].lon > 100. && bound[1].lon > 100.);
        assert!(bound[0].lat < 0. && bound[1].lat < 0.);
    }
}

use std::time::Instant;
use std::{collections::HashMap, io::BufRead};

use bevy::log::{error, info};
use chrono::{DateTime, Utc};
use ehttp::Request;
use url::Url;

use crate::adsb::math::Degrees;

#[derive(Debug, Clone)]
pub struct PlaneData {
    pub icao: String,
    pub lat: Degrees,
    pub lon: Degrees,
    pub ac_type: String,
    pub registration: String,
    pub track_degrees: Degrees,
}

fn get_url(start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Url {
    let mut url = Url::parse("http://localhost:18123").unwrap();

    let mut pairs = url.query_pairs_mut();
    pairs.clear();
    pairs.append_pair("default_format", "CSV");
    pairs.append_pair("user", "default");
    pairs.append_pair("password", "not-secret");
    pairs.append_pair("param_limit", 5_000.to_string().as_str());
    pairs.append_pair(
        "param_start_time",
        start_time.timestamp().to_string().as_str(),
    );
    pairs.append_pair("param_end_time", end_time.timestamp().to_string().as_str());
    drop(pairs);

    url
}

pub async fn get_planes(
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
) -> HashMap<String, PlaneData> {
    let start = Instant::now();

    let request = Request::post(
        get_url(start_time, end_time).as_str(),
        "SELECT icao, lat, lon, t, r, track_degrees FROM planes_mercator WHERE time > {start_time:DateTime64} AND time < {end_time:DateTime64} ORDER BY time LIMIT {limit:UInt32}"
            .as_bytes()
            .into(),
    );

    let response = ehttp::fetch_async(request)
        .await
        .expect("Failed to fetch tile image");
    let mut lines = std::io::Cursor::new(response.bytes).lines();

    // let mut file_out = std::fs::File::create("assets/clickhouse.csv").unwrap();
    // std::io::copy(&mut std::io::Cursor::new(response.bytes), &mut file_out).unwrap();

    let mut lookup = HashMap::<String, PlaneData>::new();
    while let Some(Ok(line)) = lines.next() {
        let els = line.split(',').collect::<Vec<&str>>();

        if els.len() != 6 {
            error!("Unexpected data from clickhouse: {}", line);
        }
        let icao: String = els[0].into();

        let data = PlaneData {
            icao: icao.clone(),
            lat: Degrees(els[1].parse().unwrap()),
            lon: Degrees(els[2].parse().unwrap()),
            ac_type: els[3].parse().unwrap(),
            registration: els[4].parse().unwrap(),
            track_degrees: Degrees(els[5].parse().unwrap()),
        };
        lookup.entry(icao).insert_entry(data);
    }

    info!(
        "Running get_planes() took {} milliseconds. Current time: {}",
        start.elapsed().as_millis(),
        start_time.format("%Y-%m-%dT%H:%M:%S")
    );
    lookup
}

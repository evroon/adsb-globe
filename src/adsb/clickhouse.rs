use std::collections::HashMap;
use std::time::Instant;

use ::serde::Deserialize;
use chrono::{DateTime, Utc};
use clickhouse::Client;
use clickhouse::Row;

#[derive(Row, Deserialize, Debug)]
pub struct PlaneData {
    pub icao: String,
    pub lat: f64,
    pub lon: f64,
    pub t: String,
    pub r: String,
    pub track_degrees: f32,
}

fn get_client() -> Client {
    Client::default()
        .with_url("http://localhost:18123")
        .with_user("default")
        .with_password("not-secret")
}

pub fn get_planes(
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
) -> HashMap<String, PlaneData> {
    let mut cursor = get_client()
        .query(
            "SELECT ?fields FROM planes_mercator WHERE time > ? AND time < ? ORDER BY time, icao",
        )
        .bind(start_time.format("%Y-%m-%dT%H:%M:%S").to_string())
        .bind(end_time.format("%Y-%m-%dT%H:%M:%S").to_string())
        .fetch::<PlaneData>()
        .unwrap();

    println!("{}", start_time.format("%Y-%m-%dT%H:%M:%S"));
    let now = Instant::now();

    let lookup = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            let mut lookup = HashMap::<String, PlaneData>::new();
            while let Ok(Some(plane)) = cursor.next().await {
                lookup.entry(plane.icao.to_string()).insert_entry(plane);
            }
            lookup
        });

    let elapsed_time = now.elapsed();
    println!(
        "Running get_planes() took {} milliseconds.",
        elapsed_time.as_micros() as f32 / 1000.0
    );
    lookup
}

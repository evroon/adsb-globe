use std::collections::{HashMap, VecDeque};

use bevy::prelude::*;
use chrono::{DateTime, Duration, TimeDelta, Utc};

use crate::{
    adsb::{
        math::{Coordinate, Degrees, coordinate_to_point, get_rotation, point_to_coordinate},
        task_pool::DataFetch,
    },
    earth::EARTH_RADIUS,
};

pub mod clickhouse;
pub mod math;
pub mod task_pool;

#[derive(Component, Clone, Copy, PartialEq)]
pub struct AircraftState {
    pub coordinate: Coordinate,
    pub heading: Degrees,
    pub ground_speed: f32,
    pub altitude: f32,
    pub last_relevant_time: DateTime<Utc>,
}

#[derive(Resource, Clone)]
pub struct ADSBManager {
    pub time: DateTime<Utc>,
    pub ticks: u32,
    pub target_delta: Duration,
    pub lookup: HashMap<String, Entity>,
    pub planes: u32,
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

#[derive(Component)]
pub struct Aircraft {
    pub icao: String,
    pub state: AircraftState,
    pub history: VecDeque<AircraftState>,
}

pub fn init_adsb(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(ADSBManager {
        planes: 0,
        lookup: HashMap::new(),
        target_delta: TimeDelta::seconds(10),
        ticks: 0,
        time: DateTime::parse_from_rfc3339("2025-12-28T00:00:00Z")
            .unwrap()
            .to_utc(),
        mesh: meshes.add(Cone::new(0.02, 0.1)),
        material: materials.add(Color::srgb_u8(124, 144, 0)),
    });
}

pub fn move_aircraft(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Aircraft)>,
    mut adsb: ResMut<ADSBManager>,
    data: Res<DataFetch>,
) {
    adsb.ticks += 1;

    let mut lookup = data.0.clone();

    for (entity, mut transform, mut aircraft) in query.iter_mut() {
        if let Some(data) = lookup.get(&aircraft.icao) {
            let altitude = EARTH_RADIUS + aircraft.state.altitude;
            let coord = Coordinate {
                latitude: data.lat,
                longitude: Degrees(90.0 - data.lon.0),
            };

            let old_state = aircraft.state;
            let old_coordinates = transform.translation;
            let cartesian = coordinate_to_point(&coord, altitude);
            // let new_cartesian =
            //     cartesian + transform.up() * timer.delta_secs() * aircraft.state.ground_speed;

            aircraft.state.heading = data.track_degrees;
            aircraft.state.coordinate = point_to_coordinate(cartesian.normalize());
            transform.translation = coordinate_to_point(&aircraft.state.coordinate, altitude);
            transform.rotation = get_rotation(transform.translation, &aircraft.state.heading);

            lookup.remove(&aircraft.icao);

            aircraft.history.push_front(old_state);

            if aircraft.history.len() > 10 {
                aircraft.history.pop_back();
            }

            if aircraft.state.altitude > 0.0 && transform.translation != old_coordinates {
                aircraft.state.last_relevant_time = adsb.time;
            }
        }

        if aircraft.state.last_relevant_time < adsb.time - Duration::minutes(10) {
            commands.get_entity(entity).unwrap().clear();
            commands.get_entity(entity).unwrap().despawn();
            adsb.planes -= 1;
        }
    }

    for (icao, data) in lookup {
        if adsb.planes >= 5_000 {
            continue;
        }
        adsb.planes += 1;
        commands.spawn((
            Mesh3d(adsb.mesh.clone()),
            Aircraft {
                icao,
                state: AircraftState {
                    heading: data.track_degrees,
                    ground_speed: 0.1,
                    altitude: 0.01,
                    coordinate: Coordinate {
                        latitude: data.lat,
                        longitude: data.lon,
                    },
                    last_relevant_time: adsb.time,
                },
                history: VecDeque::new(),
            },
            MeshMaterial3d(adsb.material.clone()),
            Transform::from_xyz(0.0, 0.5, 0.0),
        ));
    }
}

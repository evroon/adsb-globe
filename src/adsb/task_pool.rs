use std::collections::HashMap;

use bevy::{
    ecs::{system::SystemState, world::CommandQueue},
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future},
};

use crate::adsb::{
    ADSBManager,
    clickhouse::{PlaneData, get_planes},
};

#[derive(Component)]
pub struct ComputeTransform(pub Task<CommandQueue>);

#[derive(Resource, Clone)]
pub struct DataFetch(pub HashMap<String, PlaneData>);

pub fn spawn_task(mut commands: Commands, adsb: Res<ADSBManager>) {
    let thread_pool = AsyncComputeTaskPool::get();
    let entity = commands.spawn_empty().id();
    let time = adsb.time;
    let target_delta = adsb.target_delta;

    let task = thread_pool.spawn(async move {
        let mut command_queue = CommandQueue::default();

        let data_fetch_result = get_planes(time, time + target_delta).await;

        command_queue.push(move |world: &mut World| {
            SystemState::<ResMut<DataFetch>>::new(world)
                .get_mut(world)
                .0 = data_fetch_result;

            let mut manager = SystemState::<ResMut<ADSBManager>>::new(world).get_mut(world);

            manager.time = manager.time + manager.target_delta;

            world.entity_mut(entity).remove::<ComputeTransform>();
        });

        command_queue
    });

    commands.entity(entity).insert(ComputeTransform(task));
}

pub fn handle_tasks(mut commands: Commands, mut transform_tasks: Query<&mut ComputeTransform>) {
    for mut task in &mut transform_tasks {
        if let Some(mut commands_queue) = block_on(future::poll_once(&mut task.0)) {
            // append the returned command queue to have it execute later
            commands.append(&mut commands_queue);
        }
    }
}

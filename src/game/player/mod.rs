// // import bevy crates
// use bevy::prelude::*;

// // import modules
// pub mod components;
// pub mod systems;
// pub mod resources;

// // use modules
// use systems::*;
// use resources::*;
// use crate::game::player::components::PlayerState;

// // Player System Sets
// #[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
// pub struct MovementSystemSet;

// #[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
// pub struct ConfinementSystemSet;

// pub struct PlayerPlugin;

// impl Plugin for PlayerPlugin {
//     fn build(&self, app: &mut App) {
//         app.init_state::<PlayerState>()
//             .init_resource::<JumpTimer>()
//             .configure_sets(Update, MovementSystemSet.before(ConfinementSystemSet))
//             .add_systems(Startup, (spawn_player,))
//             .add_systems(
//                 Update,
//                 (
//                     // acceleration_changes_velocity
//                     //     .in_set(MovementSystemSet)
//                     //     .after(velocity_changes_position),
//                     // velocity_changes_position.in_set(MovementSystemSet),
//                     // player_movement_control.in_set(MovementSystemSet),
//                     // player_grounded_or_airborne,
//                     // player_confinement.in_set(ConfinementSystemSet),
//                 )
//                 // player_confinement.in_set(ConfinementSystemSet),
//                 // velocity_changes_position.in_set(MovementSystemSet),
//                 // acceleration_changes_velocity.in_set(MovementSystemSet),
//                 // player_movement_wasd.in_set(MovementSystemSet),
//                 // player_grounded_or_airborne.in_set(MovementSystemSet),
//             );
//     }
// }

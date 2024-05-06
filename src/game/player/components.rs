// use bevy::prelude::*;

// // -- COMPONENTS --

// // dont need positional variables,
// // can just use Transform.translation.(x/y) instead
// #[derive(Component)]
// pub struct Player {}

// #[derive(Component)]
// pub struct Velocity {
//     pub velocity: Vec3,
// }

// impl Default for Velocity {
//     fn default() -> Velocity {
//         Velocity {
//             velocity: Vec3::new(0.0, 0.0, 0.0),
//         }
//     }
// }

// #[derive(Component)]
// pub struct Acceleration {
//     pub change: Vec3,
// }

// impl Default for Acceleration {
//     fn default() -> Acceleration {
//         Acceleration {
//             change: Vec3::new(0.0, 0.0, 0.0),
//         }
//     }
// }

// // -- ENUMS --

// #[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
// pub enum PlayerState {
//     #[default]
//     Grounded,
//     Airborne,
// }

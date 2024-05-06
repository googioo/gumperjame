// // use crates
// use bevy::prelude::*;
// use bevy::window::PrimaryWindow;
// use bevy::math::*;

// use super::components::*;
// use super::resources::*;

// pub const PLAYER_SIZE: f32 = 64.0;
// pub const PLAYER_SPEED: f32 = 500.0;
// pub const GRAVITY: f32 = 9.8;
// // pub const JUMP_HEIGHT: f32 = 30.0;
// // pub const TIME_TO_JUMP_HEIGHT: f32 = 2.0; // 2 seconds to reach peak of jump

// // -- SPAWNS PLAYER --

// pub fn spawn_player(
//     mut commands: Commands,
//     window_query: Query<&Window, With<PrimaryWindow>>,
//     asset_server: Res<AssetServer>
// ) {
//     let window: &Window = window_query.get_single().unwrap();

//     commands
//         .spawn(SpriteBundle {
//             texture: asset_server.load("sprites/ball_blue_large.png"),
//             transform: Transform {
//                 translation: Vec3::new(window.width() / 2.0, window.height() / 2.0, 0.0),
//                 ..default()
//             },
//             ..default()
//         })
//         .insert((
//             Player {},
//             Velocity::default(),
//             Acceleration { change: Vec3::new(0.0, -GRAVITY, 0.0) },
//         ));
// }

// // -- BASIC PLAYER MOVEMENTS --

// pub fn velocity_changes_position

// // pub fn velocity_changes_position(
// //     mut player_query: Query<(&Velocity, &mut Transform), With<Player>>,
// //     time: Res<Time>
// // ) {
// //     if let Ok((velocity, mut transform)) = player_query.get_single_mut() {
// //         // move player based on player's velocity
// //         transform.translation += velocity.velocity * PLAYER_SPEED * time.delta_seconds();
// //     }
// // }

// // pub fn acceleration_changes_velocity(
// //     mut player_query: Query<(&mut Velocity, &mut Acceleration), With<Player>>,
// //     current_player_state: Res<State<PlayerState>>,
// //     mut jump_timer: ResMut<JumpTimer>,
// //     time: Res<Time>
// // ) {
// //     if let Ok((mut velocity, mut acceleration)) = player_query.get_single_mut() {
// //         // if the player is grounded then reset player's airtime and set gravity back to normal
// //         if current_player_state.get() == &PlayerState::Grounded {
// //             jump_timer.total_airtime.reset();
// //             acceleration.change.y = -GRAVITY;
// //             println!("GROUNDED");
// //         }

// //         if current_player_state.get() == &PlayerState::Airborne {
// //             // if player is airborne then tick player's airtime and increase gravity over time
// //             jump_timer.total_airtime.tick(time.delta());
// //             acceleration.change.y +=
// //                 acceleration.change.y * jump_timer.total_airtime.elapsed_secs();
// //             println!("AIRBORNE");
// //         }

// //         // adds acceleration over time to velocity
// //         velocity.velocity += acceleration.change;
// //     }
// // }

// // // confine player to the screen
// // pub fn player_confinement(
// //     mut player_query: Query<&mut Transform, With<Player>>,
// //     window_query: Query<&Window, With<PrimaryWindow>>
// // ) {
// //     if let Ok(mut transform) = player_query.get_single_mut() {
// //         let window: &Window = window_query.get_single().unwrap();
// //         let mut translation = transform.translation;
// //         let half_player_size = PLAYER_SIZE / 2.0;

// //         // Confines player movement to the screen
// //         if translation.x < half_player_size {
// //             translation.x = half_player_size;
// //         } else if translation.x > window.width() - half_player_size {
// //             translation.x = window.width() - half_player_size;
// //         }
// //         if translation.y < half_player_size {
// //             translation.y = half_player_size;
// //         } else if translation.y > window.height() - half_player_size {
// //             translation.y = window.height() - half_player_size;
// //         }

// //         transform.translation = translation;
// //     }
// // }

// // // Basic WASD movement
// // pub fn player_movement_control(
// //     mut player_query: Query<&mut Velocity, With<Player>>,
// //     keyboard_input: Res<ButtonInput<KeyCode>>
// // ) {
// //     if let Ok(mut velocity) = player_query.get_single_mut() {
// //         velocity.velocity.x = {
// //             if keyboard_input.pressed(KeyCode::ArrowLeft) {
// //                 -1.0
// //             } else if keyboard_input.pressed(KeyCode::ArrowRight) {
// //                 1.0
// //             } else {
// //                 0.0
// //             }
// //         };
// //         velocity.velocity.y = {
// //             if keyboard_input.pressed(KeyCode::ArrowDown) {
// //                 -1.0
// //             } else if keyboard_input.pressed(KeyCode::ArrowUp) {
// //                 1.0
// //             } else {
// //                 0.0
// //             }
// //         };
// //     }
// // }

// // // Updates player state to be grounded or airborne
// // pub fn player_grounded_or_airborne(
// //     mut player_query: Query<(&Transform, &Velocity, &mut Acceleration), With<Player>>,
// //     current_player_state: Res<State<PlayerState>>,
// //     mut next_player_state: ResMut<NextState<PlayerState>>,
// //     mut jump_timer: ResMut<JumpTimer>,
// //     time: Res<Time>
// // ) {
// //     if let Ok((transform, velocity, mut acceleration)) = player_query.get_single_mut() {
// //         // If there's something directly below the sprite's position (psuedo-detection)
// //         // then player is grounded. Else, they're airborne
// //         if transform.translation.y + velocity.velocity.y == transform.translation.y {
// //             next_player_state.set(PlayerState::Grounded);
// //         } else {
// //             next_player_state.set(PlayerState::Airborne);
// //         }

// //         // if the player is grounded then reset player's airtime and set gravity back to normal
// //         // if current_player_state.get() == &PlayerState::Grounded {
// //         //     jump_timer.total_airtime.reset();
// //         //     acceleration.change.y = -GRAVITY;
// //         // }

// //         // if player is airborne then tick player's airtime and increase gravity over time
// //         // if current_player_state.get() == &PlayerState::Airborne {
// //         // jump_timer.total_airtime.tick(time.delta());
// //         // acceleration.change.y +=
// //         // acceleration.change.y * jump_timer.total_airtime.elapsed_secs() * 0.2;
// //         // acceleration.change.y = acceleration.change.y * jump_timer.time.elapsed_secs();
// //         // println!(
// //         //     "delta: {0}, elapsed: {1}",
// //         //     time.delta_seconds(),
// //         //     jump_timer.time.elapsed_secs()
// //         // )
// //         // }
// //     }
// // }

// // // pub fn player_jump(
// // //     mut player_query: Query<&mut Velocity, With<Player>>,
// // //     keyboard_input: Res<ButtonInput<KeyCode>>,
// // //     time: Res<Time>
// // // ) {
// // //     if let Ok(mut velocity) = player_query.get_single_mut() {
// // //         // if keyboard_input.just_pressed(KeyCode::Space) {
// // //         //     velocity.y += MAX_JUMP_HEIGHT;
// // //         // }
// // //         velocity.y += {
// // //             if keyboard_input.just_pressed(KeyCode::Space) {
// // //                 MAX_JUMP_HEIGHT * time.delta_seconds()
// // //             } else {
// // //                 0.0
// // //             }
// // //         };
// // //     }
// // // }

// // // -- BASIC PLAYER MOVEMENTS --

// // // Updates player state to be grounded or airborne
// // pub fn player_grounded_or_aiborne(
// //     mut player_query: Query<(&Transform, &Velocity), With<Player>>,
// //     mut next_player_state: ResMut<NextState<PlayerState>>
// // ) {
// //     if let Ok((transform, velocity)) = player_query.get_single_mut() {
// //         // If there's something directly below the sprite's position (psuedo-detection)
// //         // then player is grounded. Else, they're airborne
// //         if transform.translation.y + velocity.velocity.y == transform.translation.y {
// //             next_player_state.set(PlayerState::Grounded);
// //             // println!("Player GROUNDED");
// //         } else {
// //             next_player_state.set(PlayerState::Airborne);
// //             // println!("Player AIRBORNE");
// //         }
// //     }
// // }

// // // Basic WASD movement
// // pub fn player_movement_wasd(
// //     mut player_query: Query<&mut Velocity, With<Player>>,
// //     keyboard_input: Res<ButtonInput<KeyCode>>
// // ) {
// //     if let Ok(mut velocity) = player_query.get_single_mut() {
// //         velocity.velocity.x = {
// //             if keyboard_input.pressed(KeyCode::ArrowLeft) {
// //                 -1.0
// //             } else if keyboard_input.pressed(KeyCode::ArrowRight) {
// //                 1.0
// //             } else {
// //                 0.0
// //             }
// //         };
// //         velocity.velocity.y = {
// //             if keyboard_input.pressed(KeyCode::ArrowDown) {
// //                 -1.0
// //             } else if keyboard_input.pressed(KeyCode::ArrowUp) {
// //                 1.0
// //             } else {
// //                 0.0
// //             }
// //         };
// //     }
// // }

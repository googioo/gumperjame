// -- EXTERNAL IMPORTS --
use bevy::prelude::*;
use bevy::sprite::*;
use bevy_rapier2d::prelude::*;
use bevy::math::*;
use bevy::time::*;
use bevy::window::{ PrimaryWindow, WindowResolution };
use bevy::app::AppExit;

const WINDOW_WIDTH: f32 = 1024.0;
const WINDOW_HEIGHT: f32 = 720.0;
const HALF_PLAYER: f32 = 25.0;
const TIME_TO_JUMP_EXPIRE: f32 = 0.4;
const TIME_TO_DASH_EXPIRE: f32 = 0.3;
const PLAYER_COLOR: Color = Color::GREEN;
const PLATFORM_COLOR: Color = Color::GRAY;
const SPIKE_COLOR: Color = Color::WHITE;

const PLAYER_GRAVITY: f32 = 15.0;
const PLAYER_SPEED: f32 = 8.0;
const JUMP_SPEED: f32 = 15.0;
const DASH_SPEED: f32 = 20.0;


fn main() {
    App::new()
        // window plugin settings
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Rust/Bevy Capstone Platformer".to_string(),
                    resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
                    resizable: false,
                    ..default()
                }),
                ..default()
            })
        )
        // add rapier physics plugin
        .add_plugins(RapierPhysicsPlugin::<()>::default())
        // add rendering to everything, showing 'hitboxes'
        // *** NOTE THIS PLUGIN WILL (SOMETIMES?) DISTORT COLORS OF EVERYTHING THAT USES RAPIER ***
        .add_plugins(RapierDebugRenderPlugin {
            mode: DebugRenderMode::all(),
            ..default()
        })
        .init_resource::<JumpTimer>()
        .init_resource::<DashTimer>()
        .init_state::<Direction>()
        .init_state::<GravitySwitch>()
        .init_state::<SimulationState>()
        .init_state::<AppState>()
        .add_systems(Startup, (
            spawn_camera,
            spawn_platforms.before(spawn_player),
            spawn_spikes.after(spawn_platforms),
            spawn_player.after(spawn_camera),
        ))
        .add_systems(Update, (
            player_movement,
            camera_follow.after(player_gravity),
            player_gravity.after(player_movement),
            check_grounded.after(player_gravity),
            reset_player_to_spawn.after(check_grounded),
            exit_game,
        ))
        .run()
}

// -- COMPONENTS --
#[derive(Component)]
pub struct Player {}

#[derive(Component)]
pub struct Jumps {
    pub has_grounded_jump: bool,
    pub is_jumping: bool,
}

#[derive(Resource, Default)]
pub struct JumpTimer {
    pub jump_expire: Stopwatch,
    pub coyote_time: Stopwatch,
}

#[derive(Component)]
pub struct Dash {
    pub has_dash: bool,
    pub is_dashing: bool,
}

#[derive(Resource, Default)]
pub struct DashTimer {
    pub dash_expire: Stopwatch,
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum Direction {
    #[default]
    Right,
    Left,
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GravitySwitch {
    #[default]
    On,
    Off,
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    MainMenu,
    Game,
    GameOver,
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum SimulationState {
    #[default]
    Paused,
    Running,
}

// -- EVENTS --

#[derive(Event)]
pub struct GameOver {
    pub score: u32,
}

// This PlatformBundle uses drawn shapes rather than sprites
// should be more flexible once it works
// #[derive(Bundle, Clone)]
// pub struct PlatformBundle<M: Material2d> {
//     mesh_bundle: MaterialMesh2dBundle<M>,
//     body: RigidBody,
//     collider: Collider,
// }

// Currently used PlatformBundle. Creates Rectangles using sprites
// is stiff and can only make rectangles
#[derive(Bundle)]
pub struct PlatformBundle {
    sprite_bundle: SpriteBundle,
    body: RigidBody,
    collider: Collider,
}

impl PlatformBundle {
    fn new(width: f32, height: f32, x_coord: f32, y_coord: f32) -> Self {
        Self {
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    color: PLATFORM_COLOR,
                    custom_size: Some(Vec2::new(width, height)),
                    ..default()
                },
                transform: Transform::from_xyz(x_coord, y_coord, 0.0),
                ..default()
            },
            body: RigidBody::Fixed,
            collider: Collider::cuboid(width / 2.0, height / 2.0),
        }
    }
}

// -- SETUP --
fn spawn_camera(mut commands: Commands, window_query: Query<&Window, With<PrimaryWindow>>) {
    let window: &Window = window_query.get_single().unwrap();

    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
        ..default()
    });
}

fn spawn_player(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    let window: &Window = window_query.get_single().unwrap();

    //spawn player
    commands.spawn((
        MaterialMesh2dBundle {
            // mesh: shapes[0].clone(),
            mesh: Mesh2dHandle(
                meshes.add(Rectangle::new(HALF_PLAYER * 2.0, HALF_PLAYER * 2.0))
            ),
            material: materials.add(PLAYER_COLOR),
            ..default()
        },
        Player {},
        Jumps { has_grounded_jump: false, is_jumping: false },
        Dash { has_dash: false, is_dashing: false },
        RigidBody::Dynamic,
    ))
    .insert((
        TransformBundle::from(
            Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0)
        ),
        Collider::cuboid(HALF_PLAYER, HALF_PLAYER),
        LockedAxes::ROTATION_LOCKED,
        GravityScale(0.0),
        Sleeping::disabled(),
        Ccd::enabled(),
        ActiveEvents::COLLISION_EVENTS,
        KinematicCharacterController {
            autostep: None,
            snap_to_ground: None,
            ..default()
        },
    ));
}

fn spawn_platforms(mut commands: Commands, window_query: Query<&Window, With<PrimaryWindow>>) {
    let window: &Window = window_query.get_single().unwrap();

    let half_width = window.width() / 2.0;
    let half_height = window.height() / 2.0;

    // spawn platform
    commands.spawn(PlatformBundle::new(400.0, 40.0, half_width, half_height - 120.0));

    // left platform
    commands.spawn(PlatformBundle::new(200.0, 20.0, half_width - 300.0, half_height));

    // right platform
    commands.spawn(PlatformBundle::new(200.0, 20.0, half_width + 300.0, half_height));

    // top center platform
    commands.spawn(PlatformBundle::new(200.0, 20.0, half_width, half_height + 100.0));

    commands.spawn(PlatformBundle::new(500.0, 40.0, half_width + 800.0, half_height - 120.0));
}

fn spawn_spikes(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    let window: &Window = window_query.get_single().unwrap();

    // top center spike
    commands
        .spawn((
            MaterialMesh2dBundle {
                mesh: Mesh2dHandle(
                    meshes.add(
                        Triangle2d::new(
                            Vec2::new(0.0, 0.0),
                            Vec2::new(50.0, 0.0),
                            Vec2::new(25.0, 75.0)
                        )
                    )
                ),
                material: materials.add(SPIKE_COLOR),
                ..default()
            },
            RigidBody::Fixed,
        ))
        .insert((
            TransformBundle::from(
                Transform::from_xyz(window.width() / 2.0 - 25.0, window.height() / 2.0 + 110.0, 0.0)
            ),
            Collider::triangle(Vec2::new(0.0, 0.0), Vec2::new(50.0, 0.0), Vec2::new(25.0, 75.0)),
            Sensor,
        ));
}

fn camera_follow(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>
) {
    if let Ok(player_transform) = player_query.get_single() {
        let pos = player_transform.translation;

        if let Ok(mut camera_transform) = camera_query.get_single_mut() {
            camera_transform.translation.x = pos.x;
            camera_transform.translation.y = pos.y;
        }
    }
}

fn player_gravity(
    mut controllers: Query<&mut KinematicCharacterController>,
    current_gravity_switch: Res<State<GravitySwitch>>
) {
    if current_gravity_switch.get() == &GravitySwitch::On {
        if let Ok(mut controller) = controllers.get_single_mut() {
            // gravity;
            let mut translation = controller.translation.unwrap();
            translation.y -= PLAYER_GRAVITY;
            controller.translation = Some(translation);
        }
    }
}

fn reset_player_to_spawn(mut player_query: Query<&mut Transform, With<Player>>) {
    if let Ok(mut player_position) = player_query.get_single_mut() {
        if player_position.translation.y <= 0.0 {
            player_position.translation = Vec3::new(WINDOW_WIDTH / 2.0, WINDOW_HEIGHT / 2.0, 0.0);
        }
    }
}

fn player_movement(
    keyboard_input: ResMut<ButtonInput<KeyCode>>,
    mut controllers: Query<(&mut KinematicCharacterController, &mut Jumps, &mut Dash)>,
    time: Res<Time>,
    mut jump_timer: ResMut<JumpTimer>,
    mut dash_timer: ResMut<DashTimer>,
    mut next_direction: ResMut<NextState<Direction>>,
    current_direction: Res<State<Direction>>,
    mut next_gravity_switch: ResMut<NextState<GravitySwitch>>
) {
    if let Ok((mut controller, mut jumps, mut dash)) = controllers.get_single_mut() {
        next_gravity_switch.set(GravitySwitch::On);

        let mut translation: Vec2 = match controller.translation {
            Some(vec) => {vec}
            None => {Vec2::new(0.0, 0.0)},
        };
        
        if !dash.is_dashing {
            //horizontal
            if keyboard_input.pressed(KeyCode::ArrowRight) || keyboard_input.pressed(KeyCode::KeyD)
            {
                //right
                translation.x = PLAYER_SPEED;
                next_direction.set(Direction::Right);
            } else if keyboard_input.pressed(KeyCode::ArrowLeft) || keyboard_input.pressed(KeyCode::KeyA)
            {
                // left
                translation.x = -PLAYER_SPEED;
                next_direction.set(Direction::Left);
            }

            // vertical
            if keyboard_input.pressed(KeyCode::ArrowUp) || keyboard_input.pressed(KeyCode::KeyW) {
                // up
                translation.y = PLAYER_SPEED;
            } else if
                // down
                (keyboard_input.pressed(KeyCode::ArrowDown) || keyboard_input.pressed(KeyCode::KeyS)) &&
                jumps.is_jumping
            {
                translation.y = -PLAYER_SPEED;
            }

            // jump
            // if player isn't jumping but can and pressed jump then jump
            if
                jumps.has_grounded_jump &&
                keyboard_input.pressed(KeyCode::Space)
            {
                jumps.is_jumping = true;
                // Set velocity y to jump speed
                translation.y = JUMP_SPEED;
                // also turn off gravity during jump
                next_gravity_switch.set(GravitySwitch::Off);
            }

            // player dash
            // if player isn't currently dashing or jumping, has dash and presses dash key, then dash
            if
                dash.has_dash &&
                (keyboard_input.just_pressed(KeyCode::ShiftLeft) ||
                    keyboard_input.just_pressed(KeyCode::ShiftRight)) &&
                !jumps.is_jumping
            {
                dash.is_dashing = true;
            }
        }

        if dash.is_dashing && dash_timer.dash_expire.elapsed_secs() < TIME_TO_DASH_EXPIRE {
            let direction_mult = match current_direction.get() {
                Direction::Left => -1.0,
                Direction::Right => 1.0,
            };
            translation.x = direction_mult * DASH_SPEED;
            dash_timer.dash_expire.tick(time.delta());
        }

        // if player holds jump and has jump time then keep jumping
        if jumps.is_jumping && jump_timer.jump_expire.elapsed_secs() < TIME_TO_JUMP_EXPIRE {
            // translation.y = JUMP_SPEED;
            jump_timer.jump_expire.tick(time.delta());
        }

        // if player stops jumping or jump time expires then stop jumping
        if
            jump_timer.jump_expire.elapsed_secs() >= TIME_TO_JUMP_EXPIRE ||
            keyboard_input.just_released(KeyCode::Space)
        {
            jumps.is_jumping = false;
            jumps.has_grounded_jump = false;
            jump_timer.jump_expire.reset();
        }

        if dash_timer.dash_expire.elapsed_secs() >= TIME_TO_DASH_EXPIRE {
            dash.is_dashing = false;
            dash.has_dash = false;
            dash_timer.dash_expire.reset()
        }

        // Apply changes
        controller.translation = Some(translation)
    }
}

fn check_grounded(
    mut player_query: Query<
        (&KinematicCharacterControllerOutput, &mut Jumps, &mut Dash),
        With<Player>
    >
) {
    for (player, mut jumps, mut dash) in player_query.iter_mut() {
        if player.grounded {
            // reset jumps when grounded
            jumps.has_grounded_jump = true;
            dash.has_dash = true;
            // println!("GROUNDED");
        }/* else {
            // println!("AIRBORNE");
        }*/
    }
}

// -- GAME STATES--

// fn pause_simulation(mut next_simulation_state: ResMut<NextState<SimulationState>>) {
//     next_simulation_state.set(SimulationState::Paused)
// }

// fn resume_simulation(mut next_simulation_state: ResMut<NextState<SimulationState>>) {
//     next_simulation_state.set(SimulationState::Running)
// }

// fn toggle_simulation(
//     keyboard_input: Res<ButtonInput<KeyCode>>,
//     current_simulation_state: Res<State<SimulationState>>,
//     mut next_simulation_state: ResMut<NextState<SimulationState>>
// ) {
//     if keyboard_input.just_pressed(KeyCode::KeyP) {
//         match current_simulation_state.get() {
//             SimulationState::Paused => { next_simulation_state.set(SimulationState::Running) }
//             SimulationState::Running => { next_simulation_state.set(SimulationState::Paused) }
//         }
//     }
// }

// fn transition_to_game_state(
//     keyboard_input: Res<ButtonInput<KeyCode>>,
//     current_app_state: Res<State<AppState>>,
//     mut next_app_state: ResMut<NextState<AppState>>
// ) {
//     if keyboard_input.just_pressed(KeyCode::KeyG) {
//         if current_app_state.get() != &AppState::Game {
//             next_app_state.set(AppState::Game)
//         }
//     }
// }

// fn transition_to_main_menu_state(
//     keyboard_input: Res<ButtonInput<KeyCode>>,
//     current_app_state: Res<State<AppState>>,
//     mut next_app_state: ResMut<NextState<AppState>>
// ) {
//     if keyboard_input.just_pressed(KeyCode::KeyM) {
//         if current_app_state.get() != &AppState::MainMenu {
//             next_app_state.set(AppState::MainMenu)
//         }
//     }
// }

// fn handle_game_over(
//     _game_over_event_reader: EventReader<GameOver>,
//     mut next_app_state: ResMut<NextState<AppState>>
// ) {
//     next_app_state.set(AppState::GameOver);
// }

// -- EXIT GAME --
fn exit_game(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut app_exit_event_writer: EventWriter<AppExit>
) {
    if keyboard_input.just_pressed(KeyCode::Backspace) || keyboard_input.just_pressed(KeyCode::Escape) {
        app_exit_event_writer.send(AppExit);
    }
}

// fn jump(
//     mut player_query: Query<(&mut KinematicCharacterController, &mut Jumps)>,
//     mut keyboard_input: ResMut<ButtonInput<KeyCode>>,
//     time: Res<Time>,
//     mut jump_timer: ResMut<JumpTimer>
// ) {
//     if let Ok((mut controller, mut jumps)) = player_query.get_single_mut() {
//         let jump_speed = 500.0 * time.delta_seconds();
//         if jumps.has_grounded_jump == true && keyboard_input.pressed(KeyCode::Space) {
//             jump_timer.hold_jump.tick(time.delta());
//             controller.translation = match controller.translation {
//                 Some(mut v) => {
//                     v.y = jump_speed;
//                     Some(v)
//                 }
//                 None => Some(Vec2::new(0.0, jump_speed)),
//             };
//         }
//         // if jump_timer expires or player lets go of space then end the jump, resetting values and reacitvating gravity
//         if
//             jump_timer.hold_jump.elapsed_secs() >= TIME_TO_JUMP_HEIGHT ||
//             keyboard_input.just_released(KeyCode::Space)
//         {
//             jumps.has_grounded_jump = false;
//             keyboard_input.release(KeyCode::Space);
//             jump_timer.hold_jump.reset();
//             // // cuts off vertical velocity once jump expires
//         }
//         println!("{}", jumps.has_grounded_jump)
//     }
// }

// // -- COMPONENTS --
// #[derive(Component)]
// pub struct Player {}

// #[derive(Component)]
// pub struct Acceleration {
//     pub change: Vec2,
// }

// // #[derive(Component)]
// // pub struct Jumper {
// //     pub jump_impulse: f32,
// // }

// #[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
// pub enum GroundedAirborne {
//     #[default]
//     Grounded,
//     Airborne,
// }

// #[derive(Component)]
// pub struct Jumps {
//     pub has_grounded_jump: bool,
// }

// #[derive(Resource, Default)]
// pub struct JumpTimer {
//     pub total_airtime: Stopwatch,
//     pub hold_jump: Stopwatch,
// }

// // #[derive(Bundle)]
// // pub struct PlatformBundle {
// //     mesh_bundle: MaterialMesh2dBundle,
// //     body: RigidBody,
// //     collider: Collider,
// // }

// // impl PlatformBundle {
// //     fn new(half_width: f32, half_height: f32);
// // }

// fn main() {
//     App::new()
//         // window plugin settings
//         .add_plugins(
//             DefaultPlugins.set(WindowPlugin {
//                 primary_window: Some(Window {
//                     title: "Rust/Bevy Capstone Platformer".to_string(),
//                     resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
//                     resizable: false,
//                     ..default()
//                 }),
//                 ..default()
//             })
//         )
//         // add rapier physics plugin
//         .add_plugins(RapierPhysicsPlugin::<()>::default())
//         // add rendering to everything, showing 'hitboxes'
//         // *** NOTE THIS PLUGIN WILL (SOMETIMES?) DISTORT COLORS OF EVERYTHING THAT USES RAPIER ***
//         .add_plugins(RapierDebugRenderPlugin {
//             mode: DebugRenderMode::all(),
//             ..default()
//         })
//         .init_resource::<JumpTimer>()
//         .init_state::<GroundedAirborne>()
//         .add_systems(Startup, (spawn_camera, spawn_platform, spawn_player))
//         .add_systems(Update, (
//             player_gravity,
//             player_grounded_check.after(player_gravity),
//             player_jump.after(player_grounded_check),
//             // player_jump,
//             movement_wasd,
//             return_player_jump,
//             exit_game,
//         ))
//         .run()
// }

// // -- CONSTANTS --
// const WINDOW_WIDTH: f32 = 1024.0;
// const WINDOW_HEIGHT: f32 = 720.0;
// const PLATFORM_COLOR: Color = Color::WHITE;
// const PLAYER_COLOR: Color = Color::BLUE;
// const HALF_PLAYER: f32 = 25.0;
// const PLAYER_SPEED: f32 = 500.0;
// const JUMP_IMPULSE: f32 = 500.0;
// const TIME_TO_JUMP_HEIGHT: f32 = 0.5;
// const GRAVITY: f32 = -350.0;

// // -- SETUP --
// fn spawn_camera(mut commands: Commands, window_query: Query<&Window, With<PrimaryWindow>>) {
//     let window: &Window = window_query.get_single().unwrap();

//     commands.spawn(Camera2dBundle {
//         transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
//         ..default()
//     });
// }

// fn spawn_player(
//     mut commands: Commands,
//     window_query: Query<&Window, With<PrimaryWindow>>,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<ColorMaterial>>
// ) {
//     let window: &Window = window_query.get_single().unwrap();

//     //spawn player
//     commands
//         .spawn((
//             MaterialMesh2dBundle {
//                 // mesh: shapes[0].clone(),
//                 mesh: Mesh2dHandle(
//                     meshes.add(Rectangle::new(HALF_PLAYER * 2.0, HALF_PLAYER * 2.0))
//                 ),
//                 material: materials.add(PLAYER_COLOR),
//                 ..default()
//             },
//             Player {},
//             Acceleration { change: Vec2::new(0.0, GRAVITY) },
//             Jumps { has_grounded_jump: true },
//             RigidBody::Dynamic,
//         ))
//         .insert((
//             TransformBundle::from(
//                 Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0)
//             ),
//             Collider::cuboid(HALF_PLAYER, HALF_PLAYER),
//             Velocity {
//                 linvel: Vec2::new(0.0, 0.0),
//                 angvel: 0.0,
//             },
//             LockedAxes::ROTATION_LOCKED,
//             GravityScale(0.0),
//             Sleeping::disabled(),
//             Ccd::enabled(),
//             ActiveEvents::COLLISION_EVENTS,
//         ))
//         .with_children(|parent| {
//             parent
//                 .spawn(Collider::cuboid(HALF_PLAYER * 2.0, 1.0))
//                 .insert((
//                     ActiveCollisionTypes::DYNAMIC_STATIC,
//                     TransformBundle::from(Transform::from_xyz(0.0, -HALF_PLAYER - 10.0, 0.0)),
//                     Ccd::enabled(),
//                     ActiveEvents::COLLISION_EVENTS,
//                 ));
//         });
// }

// fn spawn_platform(
//     mut commands: Commands,
//     window_query: Query<&Window, With<PrimaryWindow>>,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<ColorMaterial>>
// ) {
//     let window: &Window = window_query.get_single().unwrap();

//     commands
//         .spawn((
//             MaterialMesh2dBundle {
//                 mesh: Mesh2dHandle(meshes.add(Rectangle::new(500.0, 20.0))),
//                 material: materials.add(PLATFORM_COLOR),
//                 ..default()
//             },
//             RigidBody::Fixed,
//         ))
//         .insert((
//             TransformBundle::from(
//                 Transform::from_xyz(window.width() / 2.0, window.height() / 2.0 - 100.0, 0.0)
//             ),
//             Collider::cuboid(250.0, 10.0),
//         ));
// }

// // -- UPDATE FUNCTIONS --
// fn player_gravity(mut player_query: Query<(&mut Velocity, &Acceleration)>) {
//     if let Ok((mut velocity, acceleration)) = player_query.get_single_mut() {
//         velocity.linvel += acceleration.change;
//     }
// }

// fn movement_wasd(
//     mut player_query: Query<(&mut Velocity, &mut Acceleration), With<Player>>,
//     keyboard_input: Res<ButtonInput<KeyCode>>
// ) {
//     if let Ok((mut velocity, mut acceleration)) = player_query.get_single_mut() {
//         velocity.linvel.x = {
//             if keyboard_input.pressed(KeyCode::ArrowLeft) {
//                 -PLAYER_SPEED
//             } else if keyboard_input.pressed(KeyCode::ArrowRight) {
//                 PLAYER_SPEED
//             } else {
//                 0.0
//             }
//         };

//         velocity.linvel.y = {
//             if keyboard_input.pressed(KeyCode::ArrowDown) {
//                 -PLAYER_SPEED
//             } else if keyboard_input.pressed(KeyCode::ArrowUp) {
//                 acceleration.change.y = 0.0;
//                 PLAYER_SPEED
//             } else {
//                 acceleration.change.y = GRAVITY;
//                 0.0
//             }
//         };
//     }
// }

// fn player_jump(
//     mut player_query: Query<(&mut Velocity, &mut Jumps, &mut Acceleration), With<Player>>,
//     mut keyboard_input: ResMut<ButtonInput<KeyCode>>,
//     time: Res<Time>,
//     mut jump_timer: ResMut<JumpTimer>
// ) {
//     if let Ok((mut velocity, mut jumps, mut acceleration)) = player_query.get_single_mut() {
//         // if jump_count more than one, tick jump timer, deactivate gravity, and apply jump_velocity
//         if jumps.has_grounded_jump == true && keyboard_input.pressed(KeyCode::Space) {
//             jump_timer.hold_jump.tick(time.delta());
//             // disables gravity while jumping
//             acceleration.change.y = 0.0;
//             velocity.linvel.y = JUMP_IMPULSE;
//         }
//         // if jump_timer expires or player lets go of space then end the jump, resetting values and reacitvating gravity
//         if
//             jump_timer.hold_jump.elapsed_secs() >= TIME_TO_JUMP_HEIGHT ||
//             keyboard_input.just_released(KeyCode::Space)
//         {
//             jumps.has_grounded_jump = false;
//             acceleration.change.y = GRAVITY;
//             keyboard_input.release(KeyCode::Space);
//             jump_timer.hold_jump.reset();
//             // // cuts off vertical velocity once jump expires
//             velocity.linvel.y = 0.0;
//         }

//         println!("{0}", jumps.has_grounded_jump);
//     }
// }

// // fn player_grounded_check(
// //     rapier_context: Res<RapierContext>,
// //     children_query: Query<&Children, With<Player>>,
// //     wall_query: Query<Entity, (Without<Player>, Without<Children>)>,
// //     mut next_grounded_airborne: ResMut<NextState<GroundedAirborne>>
// // ) {}

// fn player_grounded_check(
//     rapier_context: Res<RapierContext>,
//     parent_query: Query<&Children, With<Player>>,
//     wall_query: Query<Entity, (Without<Player>, Without<Children>)>,
//     mut next_grounded_airborne: ResMut<NextState<GroundedAirborne>>
// ) {
//     if let Ok(children) = parent_query.get_single() {
//         for &child in children.iter() {
//             for wall in wall_query.iter() {
//                 /* Find the contact pair, if it exists, between two colliders. */
//                 if let Some(contact_pair) = rapier_context.contact_pair(child, wall) {
//                     // The contact pair exists meaning that the broad-phase identified a potential contact.
//                     if contact_pair.has_any_active_contacts() {
//                         // The contact pair has active contacts, meaning that it
//                         // contains contacts for which contact forces were computed.
//                         next_grounded_airborne.set(GroundedAirborne::Grounded);
//                     }
//                     if !contact_pair.has_any_active_contacts() {
//                         next_grounded_airborne.set(GroundedAirborne::Airborne);
//                     }
//                 }
//             }
//         }
//     }
// }

// fn return_player_jump(
//     current_grounded_airborne: Res<State<GroundedAirborne>>,
//     mut player_query: Query<&mut Jumps, With<Player>>
// ) {
//     if let Ok(mut jumps) = player_query.get_single_mut() {
//         if current_grounded_airborne.get() == &GroundedAirborne::Grounded {
//             jumps.has_grounded_jump = true;
//         }
//     }
// }

// fn player_grounded_check(
//     rapier_context: Res<RapierContext>,
//     children_query: Query<&Children, With<Player>>,
//     wall_query: Query<Entity, (Without<Player>, Without<Children>)>,
//     mut next_grounded_airborne: ResMut<NextState<GroundedAirborne>>
// ) {
//     if let Ok((_, child)) = children_query.get_single() {
//         for wall in wall_query.iter() {
//             /* Find the contact pair, if it exists, between two colliders. */
//             if let Some(contact_pair) = rapier_context.contact_pair(child, wall) {
//                 // The contact pair exists meaning that the broad-phase identified a potential contact.
//                 if contact_pair.has_any_active_contacts() {
//                     // The contact pair has active contacts, meaning that it
//                     // contains contacts for which contact forces were computed.
//                     next_grounded_airborne.set(GroundedAirborne::Grounded);
//                 }
//                 if !contact_pair.has_any_active_contacts() {
//                     next_grounded_airborne.set(GroundedAirborne::Airborne);
//                 }
//             }
//         }
//     }
// }

// fn player_grounded_check(
//     rapier_context: Res<RapierContext>,
//     children_query: Query<(Entity, &Children), With<Children>>,
//     wall_query: Query<Entity, (Without<Player>, Without<Children>)>,
//     mut next_grounded_airborne: ResMut<NextState<GroundedAirborne>>
// ) {
//     if let Ok((_, child)) = children_query.get_single() {
//         for wall in wall_query.iter() {
//             /* Find the contact pair, if it exists, between two colliders. */
//             if let Some(contact_pair) = rapier_context.contact_pair(child, wall) {
//                 // The contact pair exists meaning that the broad-phase identified a potential contact.
//                 if contact_pair.has_any_active_contacts() {
//                     // The contact pair has active contacts, meaning that it
//                     // contains contacts for which contact forces were computed.
//                     next_grounded_airborne.set(GroundedAirborne::Grounded);
//                 }
//                 if !contact_pair.has_any_active_contacts() {
//                     next_grounded_airborne.set(GroundedAirborne::Airborne);
//                 }
//             }
//         }
//     }
// }

// fn player_grounded_check(
//     rapier_context: Res<RapierContext>,
//     children_query: Query<Entity, With<Children>>,
//     wall_query: Query<Entity, (Without<Player>, Without<Children>)>,
//     mut next_grounded_airborne: ResMut<NextState<GroundedAirborne>>
// ) {
//     if let Ok(child) = children_query.get_single() {
//         for wall in wall_query.iter() {
//             /* Find the contact pair, if it exists, between two colliders. */
//             if let Some(contact_pair) = rapier_context.contact_pair(child, wall) {
//                 // The contact pair exists meaning that the broad-phase identified a potential contact.
//                 if contact_pair.has_any_active_contacts() {
//                     // The contact pair has active contacts, meaning that it
//                     // contains contacts for which contact forces were computed.
//                     next_grounded_airborne.set(GroundedAirborne::Grounded);
//                 } else {

//                 }
//             }
//         }
//     }
// }

// use bevy::prelude::*;
// use bevy::window::PrimaryWindow;
// use bevy::math::*;
// use bevy::time::*;
// use bevy::app::AppExit;

// fn main() {
//     App::new()
//         // Plugins
//         .add_plugins(DefaultPlugins)
//         .init_resource::<JumpTimer>()
//         .init_state::<GroundedAirborne>()
//         // Startup Systems
//         .add_systems(Startup, (spawn_camera, spawn_player, spawn_map))
//         // Update Systems
//         .add_systems(Update, (
//             grounded_or_airborne.before(player_wasd_movement),
//             player_replenishes_jumps_grounded.after(grounded_or_airborne),
//             (player_wasd_movement, player_jump).before(acceleration_changes_velocity),
//             // ANY changes to position go through velocity
//             acceleration_changes_velocity.before(velocity_changes_position),
//             velocity_changes_position,
//             // run confinement after position change
//             confine_player_to_screen.after(velocity_changes_position),
//             exit_game,
//         ))
//         .run()
// }

// // -- COMPONENTS --

// // dont need positional variables,
// // can just use Transform.translation.(x/y) instead
// #[derive(Component)]
// pub struct Player {}

// #[derive(Component)]
// pub struct Jumps {
//     pub jumps_left: f32,
// }

// #[derive(Component)]
// pub struct Velocity {
//     pub velocity: Vec3,
//     pub jump_velocity: f32,
// }

// impl Default for Velocity {
//     fn default() -> Velocity {
//         Velocity {
//             velocity: Vec3::new(0.0, 0.0, 0.0),
//             jump_velocity: 0.0,
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

// #[derive(Debug, Component, Clone, Copy)]
// pub struct HitBox(Vec2);

// // -- RESOURCES --

// #[derive(Resource, Default)]
// pub struct JumpTimer {
//     pub total_airtime: Stopwatch,
//     pub hold_jump: Stopwatch,
// }

// // -- ENUMS --

// #[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
// pub enum GroundedAirborne {
//     #[default]
//     Grounded,
//     Airborne,
// }

// pub enum PlayerActions {
//     Dash
// }

// // -- CONSTANTS --

// pub const PLAYER_SIZE: f32 = 64.0;
// pub const PLAYER_SPEED_X: f32 = 250.0;
// pub const PLAYER_SPEED_Y: f32 = 450.0;
// pub const JUMP_HEIGHT: f32 = 600.0;
// pub const TIME_TO_JUMP_HEIGHT: f32 = 1.0; // seconds
// pub const GRAVITY: f32 = 300.0;

// // -- SYSTEMS --
// // -- STARTUP SYSTEMS --

// pub fn spawn_camera(mut commands: Commands, window_query: Query<&Window, With<PrimaryWindow>>) {
//     let window: &Window = window_query.get_single().unwrap();

//     commands.spawn(Camera2dBundle {
//         transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
//         ..default()
//     });
// }

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
//             Jumps { jumps_left: 1.0 },
//             Velocity::default(),
//             Acceleration {
//                 change: Vec3::new(0.0, -GRAVITY, 0.0),
//             },
//             HitBox(Vec2::new(8.0, 8.0)),
//         ));
// }

// pub fn spawn_map(mut commands: Commands, window_query: Query<&Window, With<PrimaryWindow>>) {
//     let window: &Window = window_query.get_single().unwrap();

//     commands.spawn((
//         SpriteBundle {
//             transform: Transform::from_translation(
//                 Vec3::new(window.width() / 2.0, window.height() / 2.0 - 150.0, 0.0)
//             ),
//             sprite: Sprite {
//                 color: Color::WHITE,
//                 custom_size: Some(Vec2::new(400.0, 20.0)),
//                 ..default()
//             },
//             ..default()
//         },
//         HitBox(Vec2::new(400.0, 20.0)),
//     ));
// }

// // -- UPDATE SYSTEMS

// pub fn velocity_changes_position(
//     mut player_query: Query<(&mut Transform, &Velocity), With<Player>>,
//     time: Res<Time>
// ) {
//     if let Ok((mut transform, velocity)) = player_query.get_single_mut() {
//         transform.translation.x += velocity.velocity.x * time.delta_seconds();
//         transform.translation.y +=
//             (velocity.velocity.y + velocity.jump_velocity) * time.delta_seconds();
//         println!("x coord: {0}, y coord: {1}", transform.translation.x, transform.translation.y) // player coords
//     }
// }

// pub fn acceleration_changes_velocity(
//     mut player_query: Query<(&mut Velocity, &Acceleration, &Transform), With<Player>>
// ) {
//     if let Ok((mut velocity, acceleration, transform)) = player_query.get_single_mut() {
//         if transform.translation.y > PLAYER_SIZE / 2.0 {
//             velocity.velocity += acceleration.change;
//         }
//     }
// }

// // -- BASIC PLAYER MOVEMENTS --

// pub fn player_wasd_movement(
//     mut player_query: Query<&mut Velocity, With<Player>>,
//     keyboard_input: Res<ButtonInput<KeyCode>>
// ) {
//     if let Ok(mut velocity) = player_query.get_single_mut() {
//         velocity.velocity.x = {
//             if keyboard_input.pressed(KeyCode::ArrowLeft) {
//                 -PLAYER_SPEED_X
//             } else if keyboard_input.pressed(KeyCode::ArrowRight) {
//                 PLAYER_SPEED_X
//             } else {
//                 0.0
//             }
//         };

//         velocity.velocity.y = {
//             if keyboard_input.pressed(KeyCode::ArrowDown) {
//                 -PLAYER_SPEED_Y
//             } else if keyboard_input.pressed(KeyCode::ArrowUp) {
//                 PLAYER_SPEED_Y
//             } else {
//                 0.0
//             }
//         };
//     }
// }

// pub fn player_collision_with_level(mut player: Query<(&mut Transform, &HitBox), With<Player>>) {}

// pub fn player_jump(
//     mut player_query: Query<(Entity, &mut Velocity, &mut Jumps, &mut Acceleration), With<Player>>,
//     mut keyboard_input: ResMut<ButtonInput<KeyCode>>,
//     mut commands: Commands,
//     time: Res<Time>,
//     mut jump_timer: ResMut<JumpTimer>
// ) {
//     if let Ok((player, mut velocity, mut jumps, mut acceleration)) = player_query.get_single_mut() {
//         // if jump_count more than one, tick jump timer, deactivate gravity, and apply jump_velocity
//         if jumps.jumps_left >= 1.0 && keyboard_input.pressed(KeyCode::Space) {
//             jump_timer.hold_jump.tick(time.delta());
//             velocity.jump_velocity = JUMP_HEIGHT;
//             // disables gravity while jumping
//             acceleration.change.y = 0.0;
//         }
//         // if jump_timer expires or player lets go of space then end the jump, resetting values and reacitvating gravity
//         if
//             jump_timer.hold_jump.elapsed_secs() >= TIME_TO_JUMP_HEIGHT ||
//             keyboard_input.just_released(KeyCode::Space)
//         {
//             keyboard_input.release(KeyCode::Space);
//             jump_timer.hold_jump.reset();
//             // cuts off vertical velocity once jump expires
//             velocity.jump_velocity = 0.0;
//             // returns gravity
//             acceleration.change.y = -GRAVITY;
//             // subtract from jumps and if jumps are zero then remove them from the player entirely
//             jumps.jumps_left -= 1.0;
//             if jumps.jumps_left == 0.0 {
//                 commands.entity(player).remove::<Jumps>();
//             }
//         }
//         println!("{0}", jumps.jumps_left)
//     }
// }

// fn player_replenishes_jumps_grounded(
//     mut player_query: Query<Entity, With<Player>>,
//     mut commands: Commands,
//     current_grounded_airborne: Res<State<GroundedAirborne>>
// ) {
//     if let Ok(player) = player_query.get_single_mut() {
//         if current_grounded_airborne.get() == &GroundedAirborne::Grounded {
//             commands.entity(player).insert(Jumps { jumps_left: 1.0 });
//         }
//     }
// }

// // pub fn wall_collision_detection(
// //     mut player_query: Query<&mut Transform, With<Player>>,
// //     wall_query: Query<&HitBox>
// // ) {
// //     if let Ok(mut player_transform) = player_query.get_single_mut() {
// //         for wall_transform in wall_query.iter() {
// //             let player_radius = PLAYER_SIZE / 2.0;
// //             let hitbox_size = wall_transform.collision.y / 2.0;
// //             // let other_hitbox_size =
// //             // let half_wall_width = wall_hitbox.collision.x / 2.0;
// //             // let half_wall_height = wall_hitbox.collision.y / 2.0;
// //         }
// //     }
// // }

// pub fn check_hit(hitbox: HitBox, offset: Vec3, other_hitbox: HitBox, other_offset: Vec3) -> bool {
//     let hitbox_height = hitbox.0.y / 2.0;
//     let other_hitbox_height = other_hitbox.0.y / 2.0;
//     let hitbox_width = hitbox.0.x / 2.0;
//     let other_hitbox_width = other_hitbox.0.x / 2.0;

//     offset.x + hitbox_width > other_offset.x - other_hitbox_width &&
//         offset.x - hitbox_width < other_offset.x + other_hitbox_width &&
//         offset.y + hitbox_height > other_offset.y - other_hitbox_height &&
//         offset.y - hitbox_height < other_offset.y + other_hitbox_height
// }

// pub fn confine_player_to_screen(
//     mut player_query: Query<&mut Transform, With<Player>>,
//     window_query: Query<&Window, With<PrimaryWindow>>
// ) {
//     let window: &Window = window_query.get_single().unwrap();
//     if let Ok(mut player) = player_query.get_single_mut() {
//         let half_player_size = PLAYER_SIZE / 2.0;
//         let x_min = half_player_size;
//         let x_max = window.width() - half_player_size;
//         let y_min = half_player_size;
//         let y_max = window.height() - half_player_size;

//         if player.translation.x < x_min {
//             player.translation.x = x_min;
//         }
//         if player.translation.x > x_max {
//             player.translation.x = x_max;
//         }
//         if player.translation.y < y_min {
//             player.translation.y = y_min;
//         }
//         if player.translation.y > y_max {
//             player.translation.y = y_max;
//         }
//     }
// }

// pub fn grounded_or_airborne(
//     mut player_query: Query<(&Transform, &Velocity), With<Player>>,
//     mut last: Local<Transform>,
//     mut next_grounded_airborne: ResMut<NextState<GroundedAirborne>>,
//     current_grounded_airborne: Res<State<GroundedAirborne>>
// ) {
//     if let Ok((player_position, velocity)) = player_query.get_single_mut() {
//         // if player vertical position hasn't changed then set GroundedAirborne to Grounded. Else, Airborne
//         // if player's y position hasn't changed and player isn't moving up then player must be standing still, grounded
//         if player_position.translation.y == last.translation.y && velocity.velocity.y <= 0.0 {
//             next_grounded_airborne.set(GroundedAirborne::Grounded);
//         } else {
//             next_grounded_airborne.set(GroundedAirborne::Airborne);
//         }
//         *last = *player_position;
//         println!("{:#?}", current_grounded_airborne.get())
//     }
// }

// // -- EXIT GAME --

// pub fn exit_game(
//     keyboard_input: Res<ButtonInput<KeyCode>>,
//     mut app_exit_event_writer: EventWriter<AppExit>
// ) {
//     if keyboard_input.just_pressed(KeyCode::Backspace) {
//         app_exit_event_writer.send(AppExit);
//     }
// }
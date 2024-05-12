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

        match controller.translation {
            Some(_) => {}
            None => {
                controller.translation = Some(Vec2::new(0.0, 0.0));
                next_direction.set(Direction::Right);
            },
        }

        let mut translation: Vec2 = match controller.translation {
            Some(vec) => {vec}
            None => {
                next_direction.set(Direction::Right);
                Vec2::new(0.0, 0.0)
            },
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
                // also turn off gravity during jump
                next_gravity_switch.set(GravitySwitch::Off);
            }

            // player dash
            // if player isn't currently dashing or jumping, has dash and presses dash key, then dash
            if
                dash.has_dash &&
                (keyboard_input.just_pressed(KeyCode::ShiftLeft) ||
                    keyboard_input.just_pressed(KeyCode::ShiftRight)) &&
                jumps.is_jumping == false
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
            translation.y = JUMP_SPEED;
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

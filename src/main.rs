use bevy::core_pipeline::{
    bloom::{BloomCompositeMode, BloomSettings},
    tonemapping::Tonemapping,
};
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::sprite::MaterialMesh2dBundle;
use bevy::text::scale_value;
use bevy::window::{CursorGrabMode, PresentMode, WindowLevel, WindowTheme};
use bevy::{prelude::*, time};
use itertools::Itertools;
use rand::Rng;
use std::f32::consts::PI;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(1.0, 0.0, 0.0)))
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "I am a window!".into(),
                    resolution: (1024., 576.).into(),
                    present_mode: PresentMode::AutoVsync,
                    // Tells wasm to resize the window according to the available canvas
                    fit_canvas_to_parent: true,
                    // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                    prevent_default_event_handling: false,
                    window_theme: Some(WindowTheme::Dark),
                    enabled_buttons: bevy::window::EnabledButtons {
                        maximize: false,
                        ..Default::default()
                    },
                    // This will spawn an invisible window
                    // The window will be made visible in the make_visible() system after 3 frames.
                    // This is useful when you want to avoid the white window that shows up before the GPU is ready to render the app.
                    visible: true,
                    ..default()
                }),
                ..default()
            }),
            LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, move_coin)
        .add_systems(Update, move_cube)
        .add_systems(Update, intersections)
        .add_systems(Update, text_update_system)
        .run();
}
#[derive(Component)]
struct FpsText;
#[derive(Component)]
struct CameraState {}

#[derive(Component)]
struct CoinState {}

#[derive(Component)]
struct CubeState {
    bounds_lower: (f32, f32),
    bounds_upper: (f32, f32),
}

#[derive(Component)]
struct Score {
    score: u32,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(Score { score: 0 });
    //            transform: Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
    // Spawn a camera looking at the entities to show what's happening in this example.
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                hdr: true, // 1. HDR is required for bloom
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface, // 2. Using a tonemapper that desaturates to white is recommended
            transform: Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        BloomSettings::default(), // 3. Enable bloom for the camera
    ));

    // Text with multiple sections
    commands.spawn((
        // Create a TextBundle that has a Text with a list of sections.
        TextBundle::from_sections([TextSection::new(
            "FPS: ",
            TextStyle {
                // This font is loaded and will be used instead of the default font.
                font_size: 20.0,
                ..default()
            },
        )])
        .with_text_alignment(TextAlignment::Right)
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            right: Val::Px(5.0),
            ..default()
        }),
        FpsText,
    ));
    let material_emissive1 = standard_materials.add(StandardMaterial {
        emissive: Color::rgb_linear(13.99, 5.32, 2.0), // 4. Put something bright in a dark environment to see the effect
        ..default()
    });

    let custom_gltf = asset_server.load("penis/penis.gltf#Scene0");
    let coin_array = (-5..5).cartesian_product(-5..5);
    for coin in coin_array {
        let mut toy_transform = Transform::from_xyz((coin.0 * 2) as f32, 2.0, (coin.1 * 2) as f32)
            .with_scale(Vec3::new(0.01, 0.01, 0.01));
        //toy_transform.rotate_x(-45.0);
        commands.spawn((
            SceneBundle {
                scene: custom_gltf.clone(),
                transform: toy_transform,

                ..default()
            },
            CoinState {},
        ));
    }

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, -4.0),
        ..default()
    });
    // ground plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(50.0).into()),
        material: standard_materials.add(Color::SILVER.into()),
        ..default()
    });

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Cube::new(5.0).into()),
            transform: Transform::from_xyz(12.0, 2.5, 0.0),
            material: standard_materials.add(Color::SILVER.into()),
            ..default()
        },
        CubeState {
            bounds_lower: (-2.5, -2.5),
            bounds_upper: (2.5, 2.5),
        },
    ));
}

fn text_update_system(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
    mut commands: Commands,
    mut spheres: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut score: Query<&mut Score>,
) {
    for mut text in &mut query {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                // Update the value of the second section
                for mut a in &mut score {
                    let value = a.score;
                    text.sections[0].value = format!("Player Score: {value:.2}");
                }

                //println!("{:?}", text.sections);
            }
        }
    }
}

fn move_coin(
    input: Res<Input<KeyCode>>,
    mut coins: Query<(&mut Transform, &mut CoinState)>,
    timer: Res<Time>,
) {
    for (mut transform, coin) in &mut coins {
        //println!("{:?}", timer.delta_seconds());
        //transform.rotation.x = (2.0);
        transform.rotate_y(timer.delta_seconds() * 2.0);
        //transform.rotate_y += transform.rotate_y(timer.delta_seconds()) * 0.1;
        //transform.rotation.y += 0.1 * timer.delta_seconds();
        transform.translation.y +=
            ((3.0 * f32::sin(3.0 * timer.elapsed().as_secs_f32())) * timer.delta_seconds());

        //1.0 * timer.delta_seconds().sin();
    }
}

fn move_cube(
    input: Res<Input<KeyCode>>,
    mut cubes: Query<(&mut Transform, &mut CubeState)>,
    timer: Res<Time>,
) {
    if input.pressed(KeyCode::Left) {
        for (mut transform, cube) in &mut cubes {
            transform.translation.x -= 10.0 * timer.delta_seconds();
        }
    }
    if input.pressed(KeyCode::Right) {
        for (mut transform, cube) in &mut cubes {
            transform.translation.x += 10.0 * timer.delta_seconds();
        }
    }
    if input.pressed(KeyCode::Up) {
        for (mut transform, cube) in &mut cubes {
            transform.translation.z -= 10.0 * timer.delta_seconds();
        }
    }
    if input.pressed(KeyCode::Down) {
        for (mut transform, cube) in &mut cubes {
            transform.translation.z += 10.0 * timer.delta_seconds();
        }
    }
}

fn intersections(
    mut commands: Commands,
    mut cubes: Query<(&mut Transform, &mut CubeState), (With<CubeState>, Without<CoinState>)>,
    mut score: Query<&mut Score>,
    mut coins: Query<
        (Entity, &mut Transform, &mut CoinState),
        (Without<CubeState>, With<CoinState>),
    >,
    timer: Res<Time>,
) {
    let (player_transform, player) = cubes.iter().next().unwrap();
    let bounds_lower = player.bounds_lower;
    let bounds_upper = player.bounds_upper;
    for (mut entity, mut transform, mut coin) in &mut coins {
        if transform.translation.x.ceil() == player_transform.translation.x.ceil()
            && transform.translation.z.ceil() == player_transform.translation.z.ceil()
        {
            commands.entity(entity).despawn_recursive();
            for mut a in &mut score {
                a.score += 1
            }
        }
    }
}

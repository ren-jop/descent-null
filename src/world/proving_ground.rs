use avian2d::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;

pub struct ProvingGroundPlugin;

impl Plugin for ProvingGroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_proving_ground);
    }
}

fn spawn_proving_ground(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let rock = Color::srgb(0.32, 0.30, 0.28);
    let darker = Color::srgb(0.22, 0.20, 0.18);
    let slope = Color::srgb(0.38, 0.34, 0.30);

    // Main cavern floor
    spawn_static_rect(&mut commands, Vec2::new(0.0, -220.0), Vec2::new(2200.0, 60.0), rock);
    // Left wall / right wall
    spawn_static_rect(&mut commands, Vec2::new(-1120.0, 200.0), Vec2::new(80.0, 800.0), darker);
    spawn_static_rect(&mut commands, Vec2::new(1120.0, 200.0), Vec2::new(80.0, 800.0), darker);
    // Ceiling
    spawn_static_rect(&mut commands, Vec2::new(0.0, 620.0), Vec2::new(2200.0, 80.0), darker);

    // Starting ledge
    spawn_static_rect(&mut commands, Vec2::new(-420.0, -40.0), Vec2::new(280.0, 28.0), rock);
    // Mid platforms
    spawn_static_rect(&mut commands, Vec2::new(-40.0, 80.0), Vec2::new(220.0, 24.0), rock);
    spawn_static_rect(&mut commands, Vec2::new(280.0, 160.0), Vec2::new(180.0, 24.0), rock);
    // High perch — falling from here should register a hard landing
    spawn_static_rect(&mut commands, Vec2::new(620.0, 320.0), Vec2::new(200.0, 24.0), rock);
    // Drop shaft lip
    spawn_static_rect(&mut commands, Vec2::new(900.0, -40.0), Vec2::new(160.0, 24.0), rock);

    // Pillars that create narrow-ish passages
    spawn_static_rect(&mut commands, Vec2::new(120.0, -140.0), Vec2::new(40.0, 100.0), darker);
    spawn_static_rect(&mut commands, Vec2::new(480.0, -120.0), Vec2::new(48.0, 140.0), darker);

    spawn_ramp(
        &mut commands,
        &mut meshes,
        &mut materials,
        slope,
        Vec3::new(-180.0, -190.0, 0.0),
        [
            Vec2::new(-160.0, 90.0),
            Vec2::new(-160.0, 0.0),
            Vec2::new(160.0, 0.0),
        ],
    );
    spawn_ramp(
        &mut commands,
        &mut meshes,
        &mut materials,
        slope,
        Vec3::new(760.0, -190.0, 0.0),
        [Vec2::new(20.0, 0.0), Vec2::new(20.0, 110.0), Vec2::new(-140.0, 0.0)],
    );

    // Pushable debris — basic object physics
    let crate_color = Color::srgb(0.55, 0.38, 0.22);
    for (i, pos) in [Vec2::new(-300.0, 20.0), Vec2::new(40.0, -160.0), Vec2::new(540.0, -160.0)]
        .into_iter()
        .enumerate()
    {
        let size = 28.0 + (i as f32) * 6.0;
        commands.spawn((
            Sprite {
                color: crate_color,
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, 0.5),
            RigidBody::Dynamic,
            Collider::rectangle(size, size),
            Friction::new(0.6),
            Restitution::new(0.05),
            TransformInterpolation,
        ));
    }

    // A heavier barrel
    commands.spawn((
        Sprite {
            color: Color::srgb(0.42, 0.28, 0.18),
            custom_size: Some(Vec2::new(36.0, 48.0)),
            ..default()
        },
        Transform::from_xyz(200.0, -160.0, 0.5),
        RigidBody::Dynamic,
        Collider::capsule(18.0, 12.0),
        ColliderDensity(4.0),
        TransformInterpolation,
    ));
}

fn spawn_static_rect(commands: &mut Commands, pos: Vec2, size: Vec2, color: Color) {
    commands.spawn((
        Sprite {
            color,
            custom_size: Some(size),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 0.0),
        RigidBody::Static,
        Collider::rectangle(size.x, size.y),
        Friction::new(0.85),
    ));
}

fn spawn_ramp(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    color: Color,
    translation: Vec3,
    verts: [Vec2; 3],
) {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        verts
            .iter()
            .map(|v| [v.x, v.y, 0.0])
            .collect::<Vec<[f32; 3]>>(),
    );
    commands.spawn((
        Mesh2d(meshes.add(mesh)),
        MeshMaterial2d(materials.add(color)),
        Transform::from_translation(translation),
        RigidBody::Static,
        Collider::triangle(verts[0], verts[1], verts[2]),
        Friction::new(0.9),
    ));
}

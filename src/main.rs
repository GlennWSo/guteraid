use bevy::{
    camera::{CameraOutputMode, Hdr, visibility::RenderLayers},
    color::palettes::css::RED,
    prelude::*,
    render::render_resource::BlendState,
    sprite::Anchor,
    window::PrimaryWindow,
};
use bevy_firefly::{data::NormalMode, prelude::*};
use bevy_inspector_egui::{
    bevy_egui::{EguiGlobalSettings, EguiPlugin, PrimaryEguiContext},
    quick::WorldInspectorPlugin,
};

#[derive(Component)]
struct Player;

#[derive(Component, Debug, Copy, Clone, Deref, DerefMut, Reflect)]
// #[reflect(Component)]
pub struct MoveSpeed(pub f32);

impl Default for MoveSpeed {
    fn default() -> Self {
        Self(20.0)
    }
}

#[derive(Resource)]
struct PlayerIdleSheet(Handle<TextureAtlasLayout>);

impl PlayerIdleSheet {
    fn clone_inner(&self) -> Handle<TextureAtlasLayout> {
        self.0.clone()
    }
}

impl FromWorld for PlayerIdleSheet {
    fn from_world(world: &mut World) -> Self {
        let player_atlas = TextureAtlasLayout::from_grid((48, 64).into(), 8, 6, None, None);
        let mut texture_atlases = world
            .get_resource_mut::<Assets<TextureAtlasLayout>>()
            .unwrap();
        let texture_atlas_handle = texture_atlases.add(player_atlas);
        Self(texture_atlas_handle)
    }
}

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()));
    app.add_plugins((
        FireflyPlugin, /*FireflyGizmosPlugin*/
        EguiPlugin::default(),
        WorldInspectorPlugin::new(),
    ));
    app.register_type::<MoveSpeed>();

    app.init_resource::<Dragged>();
    app.init_resource::<PlayerIdleSheet>();

    app.add_systems(Startup, (setup_cameras, setup, spawn_player));
    app.add_systems(Update, (z_sorting, drag_objects, player_movement));

    app.run();
}

const UI_RENDER_LAYER: usize = 10;
fn setup_cameras(mut commands: Commands, mut egui_global_settings: ResMut<EguiGlobalSettings>) {
    // Disable the automatic creation of a primary context to set it up manually for every camera.
    egui_global_settings.auto_create_primary_context = false;

    let mut proj = OrthographicProjection::default_2d();
    proj.scale = 0.15;

    commands.spawn((
        Name::new("World Camera"),
        Camera2d,
        Hdr,
        Projection::Orthographic(proj),
        FireflyConfig {
            // normal maps need to be explicitly enabled
            normal_mode: NormalMode::TopDownY,
            enable_32bit_stencils: true,
            ..default()
        },
    ));

    commands.spawn((
        Name::new("UI Camera"),
        Camera2d,
        PrimaryEguiContext,
        RenderLayers::layer(UI_RENDER_LAYER),
        Camera {
            order: 10,
            clear_color: ClearColorConfig::Custom(Color::NONE),
            output_mode: CameraOutputMode::Write {
                blend_state: Some(BlendState::ALPHA_BLENDING),
                clear_color: ClearColorConfig::Custom(Color::NONE),
            },
            ..Default::default()
        },
    ));
}
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Name::new("Crate"),
        Sprite::from_image(asset_server.load("crate.png")),
        Anchor(vec2(0.0, -0.5 + 3.0 / 18.0)),
        NormalMap::from_file("crate_normal.png", &asset_server),
        Transform::from_translation(vec3(0., -20., 20.)),
        Occluder2d::rectangle(12., 5.1),
        // component added to simulate height for the normal maps. Could be useful if the object is floating above the ground.
        // this can safely not be added, and it defaults to 0.
        SpriteHeight(0.),
    ));

    commands.spawn((
        Name::new("Crate"),
        Sprite::from_image(asset_server.load("crate.png")),
        Anchor(vec2(0.0, -0.5 + 3.0 / 18.0)),
        NormalMap::from_file("crate_normal.png", &asset_server),
        Transform::from_translation(vec3(-20., 20., 0.)),
        Occluder2d::rectangle(12., 5.1),
    ));

    commands.spawn((
        Sprite::from_image(asset_server.load("vase.png")),
        Anchor(vec2(0.0, -0.5 + 5.0 / 19.0)),
        NormalMap::from_file("vase_normal.png", &asset_server),
        Transform::from_translation(vec3(0., 20., 0.)),
        Occluder2d::round_rectangle(5.4, 0.5, 3.),
        Name::new("Vase"),
    ));

    commands.spawn((
        Sprite::from_image(asset_server.load("vase.png")),
        Anchor(vec2(0.0, -0.5 + 5.0 / 19.0)),
        NormalMap::from_file("vase_normal.png", &asset_server),
        Transform::from_translation(vec3(10., -20., 0.)),
        Occluder2d::round_rectangle(5.4, 0.5, 3.),
        Name::new("Vase"),
    ));

    commands.spawn((
        Name::new("Bonfire"),
        Sprite::from_image(asset_server.load("bonfire.png")),
        PointLight2d {
            intensity: 3.,
            radius: 100.,
            core: LightCore {
                radius: 20.0,
                ..default()
            },
            color: Color::srgb(1.0, 0.8, 0.6),
            ..default()
        },
        // component added to simulate height for the normal maps.
        // you can see the lamp lighting up the top of the sprites because it has a greater height than the bonfire.
        LightHeight(3.),
    ));

    commands.spawn((
        Name::new("Street Lamp"),
        Sprite::from_image(asset_server.load("lamp.png")),
        Anchor(vec2(0.0, -0.5 + 5.0 / 32.0)),
        Transform::from_translation(vec3(20., 0., 0.)),
        PointLight2d {
            intensity: 5.,
            radius: 100.,
            core: LightCore {
                radius: 20.0,
                ..default()
            },
            color: Color::srgb(0.8, 0.8, 1.0),
            ..default()
        },
        LightHeight(22.),
    ));
}

fn spawn_player(
    mut commands: Commands,
    idle_atlas: Res<PlayerIdleSheet>,
    asset_server: Res<AssetServer>,
) {
    let image: Handle<Image> = asset_server.load("hero/Idle/Idle.png");
    // Sprite::from_image(asset_server.load("hero.png")),
    let sprite = Sprite {
        image,
        texture_atlas: Some(TextureAtlas {
            layout: idle_atlas.clone_inner(),
            ..default()
        }),
        ..Default::default()
    };
    commands.spawn((
        Name::new("Player"),
        Player,
        sprite,
        MoveSpeed(50.0),
        Anchor(vec2(-0.03, -0.45 + 3.0 / 18.0)),
        // NormalMap::from_file("crate_normal.png", &asset_server),
        Transform::from_translation(vec3(0., -20., 20.)),
        Occluder2d::round_rectangle(5.4, 0.5, 3.),
        // component added to simulate height for the normal maps. Could be useful if the object is floating above the ground.
        // this can safely not be added, and it defaults to 0.
        // SpriteHeight(0.),
    ));
}

// setting the sprite's z in relation to their y, so that Bevy's sprite renderer and firefly sort them properly.
fn z_sorting(mut sprites: Query<&mut Transform, With<Sprite>>) {
    for mut transform in &mut sprites {
        transform.translation.z = -transform.translation.y;
    }
}

#[derive(Resource, Default)]
struct Dragged(pub Option<Entity>);

fn drag_objects(
    mut objects: Query<(Entity, &mut Transform), With<Sprite>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut dragged: ResMut<Dragged>,
    mut gizmos: Gizmos,
) {
    let Some(cursor_position) = window
        .cursor_position()
        .and_then(|cursor| camera.0.viewport_to_world_2d(&camera.1, cursor).ok())
    else {
        dragged.0 = None;
        return;
    };

    if buttons.pressed(MouseButton::Left)
        && let Some(dragged) = dragged.0
        && let Ok((_, mut transform)) = objects.get_mut(dragged)
    {
        transform.translation.x = cursor_position.x;
        transform.translation.y = cursor_position.y;
        gizmos.circle_2d(
            Isometry2d::from_translation(transform.translation.xy()),
            3.,
            RED,
        );
        return;
    }

    if let Some((hovered, transform)) = objects.iter().min_by(|(_, a), (_, b)| {
        a.translation
            .xy()
            .distance(cursor_position)
            .total_cmp(&b.translation.xy().distance(cursor_position))
    }) && transform.translation.xy().distance(cursor_position) < 4.
    {
        gizmos.circle_2d(
            Isometry2d::from_translation(transform.translation.xy()),
            3.,
            RED,
        );
        if buttons.just_pressed(MouseButton::Left) {
            dragged.0 = Some(hovered);
        }
    }

    if !buttons.pressed(MouseButton::Left) {
        dragged.0 = None;
    }
}

fn player_movement(
    time: Res<Time>,
    // mut query: Query<(&mut Transform, &MoveSpeed), With<Player>>,
    player: Single<(&mut Transform, &MoveSpeed), With<Player>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let (mut transform, MoveSpeed(speed)) = player.into_inner();

    let mut direction = Vec3::ZERO;

    if keyboard_input.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }

    if direction.length_squared() > 0.0 {
        direction = direction.normalize();
    }

    transform.translation += direction * speed * time.delta_secs();
}

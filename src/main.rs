// #![windows_subsystem = "windows"]

use bevy::{
    prelude::*,
    window::{ PresentMode },
    diagnostic::{ FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin },
};

use bevy_inspector_egui::{ WorldInspectorPlugin, Inspectable, RegisterInspectable };

static GRID_SIZE: f32 = 32.0;
static SPRITE_SCALE: f32 = 2.0;

static TILE_SIZE: f32 = 16.0;
static TILESET_SIZE: Vec2 = Vec2 { x: 49.0, y: 22.0 };

static CAMERA_BASE_SPEED: f32 = 10.0;

static PLAYER_MOVEMENT_BASE_COOLDOWN: f32 = 0.15;
static PLAYER_HENSHIN_BASE_COOLDOWN: f32 = 1.0;

static PLAYER_ZOOM_DEFAULT: f32 = 0.5;
static PLAYER_ZOOM: f32 = 0.25;
static PLAYER_MIN_ZOOM: f32 = 0.25;
static PLAYER_MAX_ZOOM: f32 = 0.75;

#[derive(Resource)]
struct SpriteSheets {
    base: Handle<TextureAtlas>,
    alpha: Handle<TextureAtlas>,
}

#[derive(Component)]
struct NPC;

#[derive(Component, Inspectable)]
struct Player {
    henshin: bool,
}

impl Default for Player {
    fn default() -> Player {
        Player {
            henshin: false,
        }
    }
}

#[derive(Component, Inspectable)]
struct RemainingCooldown(f32);

#[derive(Component, Inspectable)]
enum EquipmentSlot {
    Head = 0,
}

fn main() {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "I like your cut G.".to_owned(),
                width: 800.0,
                height: 600.0,
                present_mode: PresentMode::AutoVsync,
                resizable: true,
                ..default()
            },
            ..default()
        }).set(ImagePlugin::default_nearest())
    );

    app.add_startup_system(setup);

    app.add_system(toggle_vsync);
    app.add_system(player_controller);
    app.add_system(camera_controller);
    app.add_system(npc_spawner);
    app.add_system(update_cooldown);
    app.add_system(exit_handler);

    app.insert_resource(ClearColor(Color::NONE));

    if cfg!(debug_assertions) {
        app.add_plugin(WorldInspectorPlugin::new());
        app.register_inspectable::<Player>();
        app.register_inspectable::<EquipmentSlot>();
        app.register_inspectable::<RemainingCooldown>();

        app.add_plugin(LogDiagnosticsPlugin::default());
        app.add_plugin(FrameTimeDiagnosticsPlugin::default());
    }

    app.run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>
) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scale = PLAYER_ZOOM_DEFAULT;

    commands.spawn(camera);

    let texture_handle_0 = asset_server.load("colored.png");
    let texture_atlas_0 = TextureAtlas::from_grid(
        texture_handle_0,
        Vec2::splat(TILE_SIZE),
        TILESET_SIZE.x as usize,
        TILESET_SIZE.y as usize,
        Some(Vec2::splat(1.0)),
        None
    );
    let texture_atlas_handle_0 = texture_atlases.add(texture_atlas_0);

    let texture_handle_1 = asset_server.load("colored-transparent.png");
    let texture_atlas_1 = TextureAtlas::from_grid(
        texture_handle_1,
        Vec2::splat(TILE_SIZE),
        TILESET_SIZE.x as usize,
        TILESET_SIZE.y as usize,
        Some(Vec2::splat(1.0)),
        None
    );
    let texture_atlas_handle_1 = texture_atlases.add(texture_atlas_1);

    // FLOOR
    for x in -10..10 {
        for y in -10..10 {
            commands.spawn((
                SpriteSheetBundle {
                    transform: Transform {
                        translation: Vec3::new((x as f32) * GRID_SIZE, (y as f32) * GRID_SIZE, 0.0),
                        scale: Vec3::splat(SPRITE_SCALE),
                        ..default()
                    },
                    texture_atlas: texture_atlas_handle_0.clone(),
                    sprite: TextureAtlasSprite {
                        index: 1,
                        ..default()
                    },
                    ..default()
                },
            ));
        }
    }

    // Player
    commands
        .spawn((
            SpriteSheetBundle {
                transform: Transform {
                    translation: Vec3::new(0.0 * GRID_SIZE, 0.0 * GRID_SIZE, 20.0),
                    scale: Vec3::splat(SPRITE_SCALE),
                    ..default()
                },
                texture_atlas: texture_atlas_handle_0.clone(),
                sprite: TextureAtlasSprite {
                    index: 24,
                    ..default()
                },
                ..default()
            },
            Player { ..default() },
            RemainingCooldown(0.0),
        ))
        .with_children(|parent| {
            parent.spawn((
                SpriteSheetBundle {
                    transform: Transform {
                        translation: Vec3::new(0.0, 0.0, 21.0),
                        ..default()
                    },
                    texture_atlas: texture_atlas_handle_1.clone(),
                    sprite: TextureAtlasSprite {
                        index: 33,
                        ..default()
                    },
                    ..default()
                },
                EquipmentSlot::Head,
            ));
        });

    commands.insert_resource(SpriteSheets {
        base: texture_atlas_handle_0,
        alpha: texture_atlas_handle_1,
    });
}

fn player_controller(
    input: Res<Input<KeyCode>>,
    mut players: Query<(&mut Player, &mut Transform, &mut RemainingCooldown, &Children)>,
    mut childs: Query<&mut TextureAtlasSprite>,
    child_type: Query<Option<&EquipmentSlot>>
) {
    let (mut player, mut transform, mut remaining_cooldown, childrens) = players.single_mut();

    if input.just_pressed(KeyCode::E) && !player.henshin {
        player.henshin = true;
        remaining_cooldown.0 = PLAYER_HENSHIN_BASE_COOLDOWN;
        for &children in childrens.iter() {
            let mut current_children = childs.get_mut(children).unwrap();
            let current_children_type = child_type.get(children).unwrap();
            match current_children_type {
                Some(EquipmentSlot::Head) => {
                    current_children.index = 0;
                }
                None => unreachable!(),
            }
        }
    }

    if remaining_cooldown.0 <= 0.0 {
        if input.pressed(KeyCode::W) {
            transform.translation.y += GRID_SIZE;
            remaining_cooldown.0 = PLAYER_MOVEMENT_BASE_COOLDOWN;
        } else if input.pressed(KeyCode::S) {
            transform.translation.y -= GRID_SIZE;
            remaining_cooldown.0 = PLAYER_MOVEMENT_BASE_COOLDOWN;
        }
        if input.pressed(KeyCode::D) {
            transform.translation.x += GRID_SIZE;
            remaining_cooldown.0 = PLAYER_MOVEMENT_BASE_COOLDOWN;
        } else if input.pressed(KeyCode::A) {
            transform.translation.x -= GRID_SIZE;
            remaining_cooldown.0 = PLAYER_MOVEMENT_BASE_COOLDOWN;
        }
    }
}

fn update_cooldown(time: Res<Time>, mut cooldowns: Query<&mut RemainingCooldown>) {
    for mut cooldown in cooldowns.iter_mut() {
        cooldown.0 -= time.delta_seconds();
        if cooldown.0 <= 0.0 {
            cooldown.0 = 0.0;
        }
    }
}

fn camera_controller(
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    mut cameras: Query<
        (&mut Transform, &mut OrthographicProjection),
        (With<Camera2d>, Without<Player>)
    >,
    players: Query<&Transform, (With<Player>, Without<Camera2d>)>,
    mut tiles: Query<
        (&Transform, &mut Visibility),
        (Without<Player>, Without<Camera2d>, Without<EquipmentSlot>)
    >
) {
    let (mut transform, mut projection) = cameras.single_mut();
    let player_transform = players.single();

    if input.just_pressed(KeyCode::RBracket) {
        if matches!(f32::trunc(projection.scale * 100.0) / 100.0 <= PLAYER_MIN_ZOOM, false) {
            projection.scale -= PLAYER_ZOOM;

            #[cfg(debug_assertions)]
            info!("zoom in {}", projection.scale);
        }
    } else if input.just_pressed(KeyCode::LBracket) {
        if matches!(f32::trunc(projection.scale * 100.0) / 100.0 >= PLAYER_MAX_ZOOM, false) {
            projection.scale += PLAYER_ZOOM;

            #[cfg(debug_assertions)]
            info!("zoom out {}", projection.scale);
        }
    }

    let camera_speed = CAMERA_BASE_SPEED * time.delta_seconds();

    transform.translation = transform.translation.lerp(player_transform.translation, camera_speed);
    transform.translation = transform.translation.floor();
    transform.translation.z = 999.0;

    for (_transform, mut visibility) in tiles.iter_mut() {
        let _translation = Vec2::new(_transform.translation.x, _transform.translation.y);

        let rect = Rect::from_center_size(
            Vec2::new(transform.translation.x, transform.translation.y),
            Vec2::new(projection.right * 2.0, projection.top * 2.0)
        );

        if rect.contains(_translation) {
            visibility.is_visible = true;
        } else {
            visibility.is_visible = false;
        }
    }
}

fn npc_spawner(
    mut commands: Commands,
    input: Res<Input<KeyCode>>,
    mut players: Query<&Transform, (With<Player>, Without<NPC>)>,
    npcs: Query<Entity, (With<NPC>, Without<Player>)>,
    sprite_sheets: Res<SpriteSheets>
) {
    let player_transform = players.single_mut();

    if input.just_pressed(KeyCode::F) {
        commands
            .spawn((
                SpriteSheetBundle {
                    transform: Transform {
                        translation: Vec3::new(
                            player_transform.translation.x.floor(),
                            player_transform.translation.y.floor(),
                            10.0
                        ),
                        scale: Vec3::splat(SPRITE_SCALE),
                        ..default()
                    },
                    texture_atlas: sprite_sheets.base.clone(),
                    sprite: TextureAtlasSprite {
                        index: 26,
                        ..default()
                    },
                    ..default()
                },
                NPC,
                RemainingCooldown(0.0),
            ))
            .with_children(|parent| {
                parent.spawn((
                    SpriteSheetBundle {
                        transform: Transform {
                            translation: Vec3::new(0.0, 0.0, 11.0),
                            ..default()
                        },
                        texture_atlas: sprite_sheets.alpha.clone(),
                        sprite: TextureAtlasSprite {
                            index: 0,
                            ..default()
                        },
                        ..default()
                    },
                    EquipmentSlot::Head,
                ));
            });
    }
    if input.just_pressed(KeyCode::G) {
        for npc in npcs.iter() {
            commands.entity(npc).despawn_recursive();
        }
    }
}

fn toggle_vsync(input: Res<Input<KeyCode>>, mut windows: ResMut<Windows>) {
    if input.just_pressed(KeyCode::V) {
        let window = windows.primary_mut();

        window.set_present_mode(
            if matches!(window.present_mode(), PresentMode::AutoVsync) {
                PresentMode::AutoNoVsync
            } else {
                PresentMode::AutoVsync
            }
        );

        #[cfg(debug_assertions)]
        info!("PRESENT_MODE: {:?}", window.present_mode());
    }
}

fn exit_handler(input: Res<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
}
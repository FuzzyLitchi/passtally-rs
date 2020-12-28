use bevy::{prelude::*, render::camera::Camera};

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(GamePlugin)
        .run();
}

struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(setup.system())
            .add_system(update.system())
            .add_system(fit_camera_to_screen.system())
            .add_system(animate_sprite_system.system());
    }
}

struct Board;
const BOARD_POSITION: Vec2 = Vec2 { x: 0.0, y: 0.0 };
const SCREEN_SIZE: (f32, f32) = (128.0, 192.0); //in pixels

fn setup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_scale(Vec3::splat(1.0 / 6.0)),
        ..Default::default()
    });

    let board_texture = asset_server.load("passtally_board.png");
    commands
        .spawn(SpriteBundle {
            material: materials.add(board_texture.into()),
            ..Default::default()
        })
        .with(Transform::from_translation(BOARD_POSITION.extend(-10.0)))
        .with(Board);

    let pieces_texture = asset_server.load("passtally_pieces.png");
    let pieces_spritesheet = TextureAtlas::from_grid(pieces_texture, Vec2::new(32.0, 16.0), 3, 3);
    let pieces_spritesheet_handle = texture_atlases.add(pieces_spritesheet);
    commands.with_children(|parent| {
        parent
            .spawn(SpriteSheetBundle {
                texture_atlas: pieces_spritesheet_handle,
                ..Default::default()
            })
            .with(Transform::from_translation(Vec3::new(
                -16.0 * 2.0,
                16.0 * 2.0 + 8.0,
                1.0,
            )))
            .with(Timer::from_seconds(0.5, true));
    });
}

fn fit_camera_to_screen(windows: Res<Windows>, mut query: Query<Mut<Transform>, With<Camera>>) {
    // Only one camera thanks.
    assert_eq!(query.iter_mut().count(), 1);
    for mut pos in query.iter_mut() {
        match windows.get_primary() {
            Some(window) => {
                let scale = (window.width() / SCREEN_SIZE.0).min(window.height() / SCREEN_SIZE.1);
                pos.scale = Vec2::splat(1.0 / scale).extend(1.0);
            }
            None => warn!("Couldn't get window for camera resizing."),
        }
    }
}

fn update(time: Res<Time>, mut query: Query<Mut<Transform>, With<Board>>) {
    for mut transform in query.iter_mut() {
        println!("{}", transform.translation.x);
        transform.translation.x = (time.seconds_since_startup().sin() as f32 * 50.0).round();
    }
    // println!("{}", query.iter_mut().count());
}

fn animate_sprite_system(time: Res<Time>, mut query: Query<(&mut Timer, &mut TextureAtlasSprite)>) {
    for (mut timer, mut sprite) in query.iter_mut() {
        timer.tick(time.delta_seconds());
        if timer.finished() {
            sprite.index = ((sprite.index as usize + 1) % 7) as u32;
        }
    }
}

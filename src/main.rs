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
            .add_system(animate_sprite_system.system());
    }
}

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
        .with(Transform::from_translation(Vec3::new(0.0, 8.0, 0.0)));

    let pieces_texture = asset_server.load("passtally_pieces.png");
    let pieces_spritesheet = TextureAtlas::from_grid(pieces_texture, Vec2::new(32.0, 16.0), 3, 3);
    let pieces_spritesheet_handle = texture_atlases.add(pieces_spritesheet);
    commands
        .spawn(SpriteSheetBundle {
            texture_atlas: pieces_spritesheet_handle,
            ..Default::default()
        })
        .with(Timer::from_seconds(0.5, true));
}

fn update(time: Res<Time>, mut query: Query<Mut<Transform>, Without<Camera>>) {
    for mut transform in query.iter_mut() {
        println!("{}", transform.translation.x);
        transform.translation.x = (time.seconds_since_startup().sin() as f32 * 100.0).round();
    }
    println!("{}", query.iter_mut().count());
}

fn animate_sprite_system(time: Res<Time>, mut query: Query<(&mut Timer, &mut TextureAtlasSprite)>) {
    for (mut timer, mut sprite) in query.iter_mut() {
        timer.tick(time.delta_seconds());
        if timer.finished() {
            sprite.index = ((sprite.index as usize + 1) % 7) as u32;
        }
    }
}

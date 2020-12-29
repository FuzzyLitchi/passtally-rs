use std::f32::consts::PI;

use bevy::{log, prelude::*, render::camera::Camera};
use passtally_rs::{
    board::BoardPosition,
    game::{Action, Game as PasstallyGame},
    piece::{Piece, PositionedPiece},
};
use rand::{thread_rng, Rng};

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
            .add_event::<Action>()
            .add_resource(PasstallyGame::new(2))
            .add_system(debug_keyboard.system())
            .add_system(process_passtally_move.system())
            .add_system(fit_camera_to_screen.system())
            .add_system(count_pieces.system());
    }
}

struct Board;
const SCREEN_SIZE: Vec2 = Vec2 { x: 192.0, y: 128.0 }; //in pixels
const BOARD_POSITION: Vec2 = Vec2 {
    x: -SCREEN_SIZE.x / 2.0 + 64.0,
    y: -SCREEN_SIZE.y / 2.0 + 64.0,
};
const BOARD_TOP_LEFT: Vec2 = Vec2 {
    x: BOARD_POSITION.x - 40.0,
    y: BOARD_POSITION.y - 40.0,
};

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
    texture_atlases.set("pieces", pieces_spritesheet);
}

fn fit_camera_to_screen(windows: Res<Windows>, mut query: Query<Mut<Transform>, With<Camera>>) {
    // Only one camera thanks.
    assert_eq!(query.iter_mut().count(), 1);
    for mut pos in query.iter_mut() {
        match windows.get_primary() {
            Some(window) => {
                let scale = (window.width() / SCREEN_SIZE.x).min(window.height() / SCREEN_SIZE.y);
                pos.scale = Vec2::splat(1.0 / scale).extend(1.0);
            }
            None => debug!("Couldn't get window for camera resizing."),
        }
    }
}

struct PieceMarker;
fn count_pieces(query: Query<&PieceMarker>) {
    // trace!("{} pieces!", query.iter().count());
}

fn debug_keyboard(keyboard: Res<Input<KeyCode>>, mut events: ResMut<Events<Action>>) {
    let mut rng = thread_rng();
    if keyboard.pressed(KeyCode::A) {
        events.send(Action::PlacePiece(PositionedPiece {
            piece: match rng.gen_range(0..6) {
                0 => Piece::Red,
                1 => Piece::Green,
                2 => Piece::Yellow,
                3 => Piece::Blue,
                4 => Piece::Cyan,
                5 => Piece::Pink,
                _ => unreachable!(),
            },
            position: BoardPosition::new(rng.gen_range(0..6), rng.gen_range(0..6)),
            rotation: rng.gen_range(0..4),
        }));
    }
}

fn process_passtally_move(
    commands: &mut Commands,
    events: Res<Events<Action>>,
    mut reader: Local<EventReader<Action>>,
    mut passtally_game: ResMut<PasstallyGame>,
    texture_atlases: Res<Assets<TextureAtlas>>,
) {
    for action in reader.iter(&events) {
        trace!("Handling {:?}", action);
        match passtally_game.do_action(action.clone()) {
            Err(e) => trace!("{}", e),
            Ok(_) => {
                // Add
                match action {
                    Action::PlacePiece(piece) => {
                        let pieces_spritesheet_handle = texture_atlases.get_handle("pieces");

                        let (pos1, pos2) = piece.positions();
                        let mut transform = Transform::from_translation(
                            (BOARD_TOP_LEFT
                                + Vec2::new(
                                    16.0 * (pos1.x as f32 + pos2.x as f32) / 2.0,
                                    16.0 * (pos1.y as f32 + pos2.y as f32) / 2.0,
                                ))
                            .extend(-1.0 + 0.001 * (passtally_game.board.next_id as f32)),
                        );
                        transform.rotate(Quat::from_rotation_z(PI / 2.0 * piece.rotation as f32));

                        commands
                            .spawn(SpriteSheetBundle {
                                texture_atlas: pieces_spritesheet_handle,
                                sprite: TextureAtlasSprite::new(piece.piece.index()),
                                ..Default::default()
                            })
                            .with(transform)
                            .with(PieceMarker);
                    }
                    _ => unimplemented!(),
                }
            }
        }
    }
}

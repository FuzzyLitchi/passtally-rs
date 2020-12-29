use std::{f32::consts::PI, time::Duration};

use bevy::{prelude::*, render::camera::Camera};
use bevy_easings::{Ease, EaseFunction, EasingType, EasingsPlugin};
use passtally_rs::{
    board::BoardPosition,
    game::{Action, Game as PasstallyGame},
    piece::{Piece, PositionedPiece},
};
use rand::{thread_rng, Rng};

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(EasingsPlugin)
        .add_plugin(GamePlugin)
        .run();
}

struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(setup.system())
            .add_event::<Action>()
            .add_system(debug_keyboard.system())
            .add_system(process_passtally_move.system())
            .add_system(fit_camera_to_screen.system())
            .add_system(selection_system.system());
    }
}

struct Board;
const SCREEN_SIZE: Vec2 = Vec2 { x: 192.0, y: 128.0 }; //in pixels
const BOARD_POSITION: Vec2 = Vec2 {
    x: -SCREEN_SIZE.x / 2.0 + 64.0,
    y: -SCREEN_SIZE.y / 2.0 + 64.0,
};
const BOARD_BOTTOM_LEFT: Vec2 = Vec2 {
    x: BOARD_POSITION.x - 40.0,
    y: BOARD_POSITION.y - 40.0,
};

fn setup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let camera = commands
        .spawn(Camera2dBundle {
            transform: Transform::from_scale(Vec3::splat(1.0 / 6.0)),
            ..Default::default()
        })
        .current_entity()
        .unwrap();
    commands.insert_resource(SelectionSystemState { camera_e: camera });

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

    let markers = asset_server.load("player_markers.png");
    let pieces_spritesheet = TextureAtlas::from_grid(markers, Vec2::new(8.0, 8.0), 2, 1);
    texture_atlases.set("markers", pieces_spritesheet);

    let passtally = PasstallyGame::new(2);
    for (i, player) in passtally.player_markers() {
        info!("Player {1} has a marker at {0}", i, player);

        let player_marker = PlayerMarker {
            pos: i as u8,
            player,
        };

        commands
            .spawn(SpriteSheetBundle {
                texture_atlas: texture_atlases.get_handle("markers"),
                sprite: TextureAtlasSprite::new(player_marker.player as u32),
                transform: Transform::from_translation(player_marker.world_pos()),
                ..Default::default()
            })
            .with(player_marker)
            .with(Clickable {
                bounding_box: Size::new(8.0, 8.0),
            });
    }

    let mut rng = thread_rng();
    for i in 0..3 {
        let mut transform = Transform::from_translation(
            Vec2::new(144.0 - 96.0, (40 * i) as f32 + 24.0 - 64.0).extend(-1.0),
        );
        transform.rotate(Quat::from_rotation_z(PI / 2.0));

        commands
            .spawn(SpriteSheetBundle {
                texture_atlas: texture_atlases.get_handle("pieces"),
                sprite: TextureAtlasSprite::new(rng.gen_range(0..6)),
                transform,
                ..Default::default()
            })
            .with(Clickable {
                bounding_box: Size::new(16.0, 32.0),
            });
    }
    for i in 0..3 {
        let mut transform = Transform::from_translation(
            Vec2::new(168.0 - 96.0, (40 * i) as f32 + 24.0 - 64.0).extend(-1.0),
        );
        transform.rotate(Quat::from_rotation_z(PI / 2.0));

        commands.spawn(SpriteSheetBundle {
            texture_atlas: texture_atlases.get_handle("pieces"),
            sprite: TextureAtlasSprite::new(6),
            transform,
            ..Default::default()
        });
    }
    commands.insert_resource(passtally);
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
    if keyboard.pressed(KeyCode::B) {
        events.send(Action::MovePlayerMarker(
            rng.gen_range(0..24),
            rng.gen_range(0..24),
        ));
    }
}

struct PlayerMarker {
    pos: u8,
    player: u8,
}

impl PlayerMarker {
    fn world_pos(&self) -> Vec3 {
        let pos = match self.pos {
            0..=5 => Vec2::new(self.pos as f32, 0.0) * 16.0 + Vec2::new(0.0, -13.0),
            6..=11 => Vec2::new(5.0, (self.pos % 6) as f32) * 16.0 + Vec2::new(13.0, 0.0),
            12..=17 => Vec2::new((5 - (self.pos % 6)) as f32, 5.0) * 16.0 + Vec2::new(0.0, 13.0),
            18..=23 => Vec2::new(0.0, (5 - (self.pos % 6)) as f32) * 16.0 + Vec2::new(-13.0, 0.0),
            _ => unreachable!(),
        };
        (BOARD_BOTTOM_LEFT + pos).extend(0.0)
    }
}

fn process_passtally_move(
    commands: &mut Commands,
    events: Res<Events<Action>>,
    mut reader: Local<EventReader<Action>>,
    mut passtally_game: ResMut<PasstallyGame>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut player_marker_query: Query<(Entity, &mut PlayerMarker, &Transform)>,
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
                            (BOARD_BOTTOM_LEFT
                                + Vec2::new(
                                    16.0 * (pos1.x as f32 + pos2.x as f32) / 2.0,
                                    16.0 * (pos1.y as f32 + pos2.y as f32) / 2.0,
                                ))
                            .extend(-1.0 + 0.001 * (passtally_game.board.next_id as f32)),
                        );
                        transform.rotate(Quat::from_rotation_z(PI / 2.0 * piece.rotation as f32));

                        commands.spawn(SpriteSheetBundle {
                            texture_atlas: pieces_spritesheet_handle,
                            sprite: TextureAtlasSprite::new(piece.piece.index()),
                            transform,
                            ..Default::default()
                        });
                    }
                    Action::MovePlayerMarker(from, to) => {
                        for (entity, mut player_marker, transform) in player_marker_query.iter_mut()
                        {
                            if player_marker.pos == *from {
                                // Update position index.
                                player_marker.pos = *to;

                                // Move player marker in world.
                                let easing = transform.ease_to(
                                    Transform::from_translation(player_marker.world_pos()),
                                    EaseFunction::QuadraticOut,
                                    EasingType::Once {
                                        duration: Duration::from_millis(500),
                                    },
                                );
                                commands.insert_one(entity, easing);
                            }
                        }
                    }
                }
            }
        }
    }
}

struct SelectionSystemState {
    // need to identify the main camera
    camera_e: Entity,
    // Selected entity
}

struct Clickable {
    bounding_box: Size<f32>,
}

fn selection_system(
    state: Res<SelectionSystemState>,
    mouse: Res<Input<MouseButton>>,
    // need to get window dimensions
    windows: Res<Windows>,
    // query to get camera components
    camera_query: Query<&Transform>,
    query: Query<(&Clickable, &Transform)>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        let window = windows.get_primary().unwrap();
        if let Some(cursor) = window.cursor_position() {
            let camera_transform = camera_query.get(state.camera_e).unwrap();
            // get the size of the window that the event is for
            let size = Vec2::new(window.width() as f32, window.height() as f32);

            // the default orthographic projection is in pixels from the center;
            // just undo the translation
            let p = cursor - size / 2.0;

            // apply the camera transform
            let world_position = camera_transform.compute_matrix() * p.extend(0.0).extend(1.0);
            let world_position = world_position.truncate().truncate();
            debug!("World coords: {}/{}", world_position.x, world_position.y);

            for (clickable, transform) in query.iter() {
                let click_pos = transform.translation.truncate();
                let bounding_box = clickable.bounding_box;
                let left = click_pos.x - bounding_box.width / 2.0;
                let right = click_pos.x + bounding_box.width / 2.0;
                let bottom = click_pos.y - bounding_box.height / 2.0;
                let top = click_pos.y + bounding_box.height / 2.0;

                if world_position.x > left
                    && world_position.x < right
                    && world_position.y > bottom
                    && world_position.y < top
                {
                    info!("Clicked!!");
                }
            }
        }
    }
}

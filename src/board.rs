use array_macro::array;
use std::ops::Add;

use crate::game::PasstallyError;
use crate::piece::{Side::*, *};

#[derive(Clone)]
pub struct Board {
    top_pieces: [[RotatedPartialPiece; 6]; 6], // Used to direct lines
    tile_id: [[u32; 6]; 6], // Used to tell when you are moving from a one piece to another
    next_id: u32,           // Id of the next piece, assured to be unique
    height: [[u32; 6]; 6],  // Height of specific partial piece, used to calculate score
}

impl Board {
    pub fn default() -> Self {
        Board {
            top_pieces: array![array![RotatedPartialPiece::new(PartialPiece::TopBottom_LeftRight, 0); 6]; 6],
            tile_id: [[0; 6]; 6],
            next_id: 1,
            height: [[0; 6]; 6],
        }
    }

    pub fn place_piece(&mut self, piece: PositionedPiece) -> Result<(), PasstallyError> {
        let (pos1, pos2) = piece.positions();

        // Assert position is within board
        if !pos1.valid() {
            return Err(PasstallyError::InvalidPosition(pos1));
        } else if !pos2.valid() {
            return Err(PasstallyError::InvalidPosition(pos2));
        }

        // Assert height for the positions are the same.
        if self.height(pos1) != self.height(pos2) {
            return Err(PasstallyError::BadHeight);
        }

        // Assert the pieces we're placing the piece on are different. But only if they aren't the board.
        // i.e. we aren't placing this piece directly ontop of another one.
        if self.tile_id(pos1) != 0
            && self.tile_id(pos2) != 0
            && self.tile_id(pos1) == self.tile_id(pos2)
        {
            return Err(PasstallyError::BadPiece);
        }

        // This is a valid move, so we do it
        *self.height_mut(pos1) += 1;
        *self.height_mut(pos2) += 1;

        *self.tile_id_mut(pos1) = self.next_id;
        *self.tile_id_mut(pos2) = self.next_id;
        self.next_id += 1;

        let (piece1, piece2) = piece.rotated_partial_pieces();
        *self.top_piece_mut(pos1) = piece1;
        *self.top_piece_mut(pos2) = piece2;

        Ok(())
    }

    // TODO: calulate points
    fn enter(&self, entry: BoardPosition, mut side: Side) -> BoardPosition {
        let mut pos = entry;
        while pos == entry || !pos.on_edge() {
            // Where does this piece take us?
            let exit_side = self.top_piece(pos).pass(side);
            println!("{:?} {:?}", pos, exit_side);
            // Calculate delta_position
            let delta_position = match exit_side {
                Top => (0, -1),
                Bottom => (0, 1),
                Left => (-1, 0),
                Right => (1, 0),
            };
            pos.x += delta_position.0;
            pos.y += delta_position.1;

            // Next enter side is the opposite of exit side
            side = exit_side.opposite();
        }
        pos
    }

    fn top_piece(&self, i: BoardPosition) -> &RotatedPartialPiece {
        &self.top_pieces[i.x as usize][i.y as usize]
    }

    fn tile_id(&self, i: BoardPosition) -> u32 {
        self.tile_id[i.x as usize][i.y as usize]
    }

    fn height(&self, i: BoardPosition) -> u32 {
        self.height[i.x as usize][i.y as usize]
    }

    fn top_piece_mut(&mut self, i: BoardPosition) -> &mut RotatedPartialPiece {
        &mut self.top_pieces[i.x as usize][i.y as usize]
    }

    fn tile_id_mut(&mut self, i: BoardPosition) -> &mut u32 {
        &mut self.tile_id[i.x as usize][i.y as usize]
    }

    fn height_mut(&mut self, i: BoardPosition) -> &mut u32 {
        &mut self.height[i.x as usize][i.y as usize]
    }
}

/// Position on board. x and y value are 0..=5 when on the board
/// 0,0 is at the top left. x is horizontal and y is vertical
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct BoardPosition {
    x: i8,
    y: i8,
}

impl BoardPosition {
    pub fn new(x: i8, y: i8) -> Self {
        BoardPosition { x, y }
    }

    fn on_edge(&self) -> bool {
        self.x == 0 || self.y == 0 || self.x == 5 || self.y == 5
    }

    fn valid(&self) -> bool {
        self.x <= 5 && self.x >= 0 && self.y <= 5 && self.y >= 0
    }
}

impl Add for BoardPosition {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        BoardPosition {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_pieces_sanity() {
        use crate::piece::PartialPiece::*;

        for partial_piece in &[
            TopBottom_LeftRight,
            TopLeft_BottomRight,
            TopRight_BottomLeft,
        ] {
            for side in &[Top, Bottom, Left, Right] {
                assert_eq!(partial_piece.pass(partial_piece.pass(*side)), *side);
            }
        }
    }

    #[test]
    fn simple_board() {
        let board = Board::default();

        let a = board.enter(BoardPosition::new(2, 0), Side::Top);
        assert_eq!(a, BoardPosition::new(2, 5));

        let b = board.enter(BoardPosition::new(0, 2), Side::Left);
        assert_eq!(b, BoardPosition::new(5, 2));
    }

    #[test]
    fn rotated_partial_piece_sanity() {
        use PartialPiece::*;

        for partial_piece in &[
            TopBottom_LeftRight,
            TopLeft_BottomRight,
            TopRight_BottomLeft,
        ] {
            for rotation in 0..4 {
                let rotated_partial_piece = RotatedPartialPiece::new(*partial_piece, rotation);

                for side in &[Top, Bottom, Left, Right] {
                    println!("Rotation {}", rotation);
                    assert_eq!(
                        rotated_partial_piece.pass(rotated_partial_piece.pass(*side)),
                        *side
                    );
                }
            }
        }
    }

    #[test]
    fn place_pieces() {
        let mut board = Board::default();
        let piece = PositionedPiece {
            piece: Piece::Pink,
            position: BoardPosition::new(0, 0),
            rotation: 0,
        };
        board.place_piece(piece).unwrap();

        // Placing it again will fail.
        let piece = PositionedPiece {
            piece: Piece::Pink,
            position: BoardPosition::new(0, 0),
            rotation: 0,
        };
        assert!(matches!(
            board.place_piece(piece).unwrap_err(),
            PasstallyError::BadPiece,
        ));

        // Placing a piece halfway ontop of it will also fail
        let piece = PositionedPiece {
            piece: Piece::Pink,
            position: BoardPosition::new(0, 0),
            rotation: 1, // Rotated
        };
        assert!(matches!(
            board.place_piece(piece).unwrap_err(),
            PasstallyError::BadHeight,
        ));

        // Placing a piece below is fine
        // ^^
        // vv
        let piece = PositionedPiece {
            piece: Piece::Pink,
            position: BoardPosition::new(1, 1),
            rotation: 2, // Rotated
        };
        board.place_piece(piece).unwrap();

        // Placing a piece on top of these now works.
        // >^
        // >v
        let piece = PositionedPiece {
            piece: Piece::Pink,
            position: BoardPosition::new(0, 0),
            rotation: 1, // Rotated
        };
        board.place_piece(piece).unwrap();

        assert_eq!(
            board.height,
            [
                [2, 2, 0, 0, 0, 0], // The x and y axes are swapped.
                [1, 1, 0, 0, 0, 0], // So just transpose this in your mind
                [0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0],
            ]
        )
    }
}

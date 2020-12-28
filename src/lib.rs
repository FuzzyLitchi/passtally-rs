use array_macro::array;
use num_enum::TryFromPrimitive;
use std::{convert::TryFrom, ops::Add};
use thiserror::Error;

#[derive(Copy, Clone, PartialEq, Debug, TryFromPrimitive)]
#[repr(u8)]
enum Side {
    Top = 0,
    Right = 1,
    Bottom = 2,
    Left = 3,
}
use Side::*;

impl Side {
    fn opposite(self) -> Self {
        match self {
            Top => Bottom,
            Bottom => Top,
            Left => Right,
            Right => Left,
        }
    }

    /// Rotation is clockwise and 0..=3
    fn rotate(self, n: u8) -> Self {
        Self::try_from((self as u8 + n) % 4).unwrap()
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[allow(non_camel_case_types, clippy::enum_variant_names)]
enum PartialPiece {
    TopBottom_LeftRight, // Pipes top to bottom and left to right
    TopLeft_BottomRight, // Pipes top to left and bottom to right
    TopRight_BottomLeft, // Pipes top to right and bottom to left
}

impl PartialPiece {
    // returns which side we are leaving from when we pass through this partial piece
    fn pass(&self, side: Side) -> Side {
        use PartialPiece::*;

        match self {
            TopBottom_LeftRight => side.opposite(),
            TopLeft_BottomRight => match side {
                Top => Left,
                Left => Top,
                Bottom => Right,
                Right => Bottom,
            },
            TopRight_BottomLeft => match side {
                Top => Right,
                Right => Top,
                Bottom => Left,
                Left => Bottom,
            },
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
struct RotatedPartialPiece {
    partial_piece: PartialPiece,
    rotation: u8,
}

impl RotatedPartialPiece {
    fn new(partial_piece: PartialPiece, rotation: u8) -> Self {
        RotatedPartialPiece {
            partial_piece,
            rotation,
        }
    }

    fn pass(&self, side: Side) -> Side {
        // Rotate into local side
        let local_side = side.rotate(4 - self.rotation);
        // Pass through piece
        let exit_side = self.partial_piece.pass(local_side);
        // Rotate back to global
        exit_side.rotate(self.rotation)
    }
}

enum Piece {
    // A is TopBottom_LeftRight,
    // B is TopLeft_BottomRight,
    // C is TopRight_BottomLeft
    Red,    // A A
    Green,  // B B
    Yellow, // C C
    Blue,   // A B
    Cyan,   // A C
    Pink,   // C B
}

struct PositionedPiece {
    piece: Piece,
    rotation: u8,
    position: BoardPosition,
}

impl PositionedPiece {
    fn positions(&self) -> (BoardPosition, BoardPosition) {
        let second_position = match self.rotation {
            0 => self.position + BoardPosition::new(1, 0), // Unrotated pieces are horizontal, and the second part is to the right
            1 => self.position + BoardPosition::new(0, 1),
            2 => self.position + BoardPosition::new(-1, 0),
            3 => self.position + BoardPosition::new(0, -1),
            _ => unreachable!("Rotation should only be 0-3"),
        };
        (self.position, second_position)
    }

    fn rotated_partial_pieces(&self) -> (RotatedPartialPiece, RotatedPartialPiece) {
        let (first_piece, second_piece) = self.partial_pieces();
        (
            RotatedPartialPiece::new(first_piece, self.rotation),
            RotatedPartialPiece::new(second_piece, self.rotation),
        )
    }

    fn partial_pieces(&self) -> (PartialPiece, PartialPiece) {
        use Piece::*;
        match self.piece {
            Red => (
                PartialPiece::TopBottom_LeftRight,
                PartialPiece::TopBottom_LeftRight,
            ),
            Green => (
                PartialPiece::TopLeft_BottomRight,
                PartialPiece::TopLeft_BottomRight,
            ),
            Yellow => (
                PartialPiece::TopRight_BottomLeft,
                PartialPiece::TopRight_BottomLeft,
            ),
            Blue => (
                PartialPiece::TopBottom_LeftRight,
                PartialPiece::TopLeft_BottomRight,
            ),
            Cyan => (
                PartialPiece::TopBottom_LeftRight,
                PartialPiece::TopRight_BottomLeft,
            ),
            Pink => (
                PartialPiece::TopRight_BottomLeft,
                PartialPiece::TopLeft_BottomRight,
            ),
        }
    }
}

#[derive(Error, Debug)]
pub enum PasstallyError {
    #[error("The piece is outside of the board.")]
    InvalidPosition,
    #[error("The height for the two positions aren't the same.")]
    BadHeight,
    #[error("You cannot place a piece directly ontop of another piece.")]
    BadPiece,
}

struct Board {
    top_pieces: [[RotatedPartialPiece; 6]; 6], // Used to direct lines
    tile_id: [[u32; 6]; 6], // Used to tell when you are moving from a one piece to another
    next_id: u32,           // Id of the next piece, assured to be unique
    height: [[u32; 6]; 6],  // Height of specific partial piece, used to calculate score
}

impl Board {
    fn new() -> Self {
        Board {
            top_pieces: array![array![RotatedPartialPiece::new(PartialPiece::TopBottom_LeftRight, 0); 6]; 6],
            tile_id: [[0; 6]; 6],
            next_id: 1,
            height: [[0; 6]; 6],
        }
    }

    fn place_piece(&mut self, piece: PositionedPiece) -> Result<(), PasstallyError> {
        let (pos1, pos2) = piece.positions();

        // Assert position is within board
        if !(pos1.valid() && pos2.valid()) {
            return Err(PasstallyError::InvalidPosition);
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
struct BoardPosition {
    x: i8,
    y: i8,
}

impl BoardPosition {
    fn new(x: i8, y: i8) -> Self {
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
        use PartialPiece::*;

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
        let board = Board::new();

        let a = board.enter(BoardPosition::new(2, 0), Side::Top);
        assert_eq!(a, BoardPosition::new(2, 5));

        let b = board.enter(BoardPosition::new(0, 2), Side::Left);
        assert_eq!(b, BoardPosition::new(5, 2));
    }

    #[test]
    fn rotated_partial_piece() {
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
    fn place_piece() {
        let mut board = Board::new();
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
        // nn
        // uu
        let piece = PositionedPiece {
            piece: Piece::Pink,
            position: BoardPosition::new(1, 1),
            rotation: 2, // Rotated
        };
        board.place_piece(piece).unwrap();

        // Placing a piece on top of these now works.
        let piece = PositionedPiece {
            piece: Piece::Pink,
            position: BoardPosition::new(0, 0),
            rotation: 1, // Rotated
        };
        board.place_piece(piece).unwrap();
    }
}

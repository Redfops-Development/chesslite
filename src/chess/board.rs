use std::ops::Not;

use bevy::prelude::*;

#[derive(Clone,Copy,PartialEq,Debug)]
pub enum GameOverState{
    AgreedDraw,
    Stalemate,
    Checkmate(PieceColor),
    Resignation(PieceColor),
    Ongoing
}

#[derive(Clone,Copy,PartialEq)]
pub enum PieceType{
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn
}

#[derive(Clone,Copy,PartialEq,Debug)]
pub enum PieceColor{
    Black,
    White
}

impl Not for PieceColor{
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            PieceColor::Black => PieceColor::White,
            PieceColor::White => PieceColor::Black,
        }
    }
}

#[derive(Component,Clone,Copy)]
pub struct Piece {
    pub piece: PieceType,
    pub color: PieceColor
}

pub struct CastleCheck {
    pub cancastle: bool,
    pub rooksource: Option<(usize,usize)>,
    pub rookdestination: Option<(usize,usize)>,
}

impl CastleCheck {
    pub fn new(cancastle:bool) -> Self {
        CastleCheck {
            cancastle,
            rooksource: None,
            rookdestination: None,
        }
    }
}

#[derive(Clone,Copy)]
pub struct Move {
    pub from: (usize, usize),
    pub to: (usize, usize),
    pub color: PieceColor,
    pub piece: Piece,
}

#[derive(Resource,Clone)]
pub struct Board {
    pub tiles: [[Option<Piece>;8];8],
    pub lastmove: Option<Move>,
    pub movelist: Vec<Move>,
}

impl Board {
    pub fn new() -> Self {
        Board {
            tiles: [
            [Some(Piece{piece: PieceType::Rook, color: PieceColor::White}),Some(Piece{piece: PieceType::Knight, color: PieceColor::White}),Some(Piece{piece: PieceType::Bishop, color: PieceColor::White}), Some(Piece{piece: PieceType::Queen, color: PieceColor::White}), Some(Piece{piece: PieceType::King, color: PieceColor::White}), Some(Piece{piece: PieceType::Bishop, color: PieceColor::White}), Some(Piece{piece: PieceType::Knight, color: PieceColor::White}), Some(Piece{piece: PieceType::Rook, color: PieceColor::White})],
            [Some(Piece{piece: PieceType::Pawn, color: PieceColor::White});8],
            [None;8],
            [None;8],
            [None;8],
            [None;8],
            [Some(Piece{piece: PieceType::Pawn, color: PieceColor::Black});8],
            [Some(Piece{piece: PieceType::Rook, color: PieceColor::Black}),Some(Piece{piece: PieceType::Knight, color: PieceColor::Black}),Some(Piece{piece: PieceType::Bishop, color: PieceColor::Black}), Some(Piece{piece: PieceType::Queen, color: PieceColor::Black}), Some(Piece{piece: PieceType::King, color: PieceColor::Black}), Some(Piece{piece: PieceType::Bishop, color: PieceColor::Black}), Some(Piece{piece: PieceType::Knight, color: PieceColor::Black}), Some(Piece{piece: PieceType::Rook, color: PieceColor::Black})],
            ],
            lastmove: None,
            movelist: Vec::new(),
        }
    }

    pub fn is_gameover(&self) -> GameOverState {
        if self.any_legal_moves() {return GameOverState::Ongoing;}
        
        let moving_color = self.to_move();

        if self.is_check(moving_color) {return GameOverState::Checkmate(!moving_color);}
        return GameOverState::Stalemate;
    }

    pub fn any_legal_moves(&self) -> bool {
        let moving_color = self.to_move();
        for x in 0..8 {
            for y in 0..8 {
                if let Some(piece) = self.tiles[x][y] {
                    if moving_color != piece.color {continue;}
                    for x_dest in 0..8 {
                        for y_dest in 0..8 {
                            if self.is_legal_move((x,y), (x_dest,y_dest)) {return true;}
                        }
                    }
                }
            }
        }
        return false;
    }
    
    pub fn to_move(&self) -> PieceColor {
        if let Some(prevmove) = self.lastmove {
            return !prevmove.color;
        }
        return PieceColor::White;
    }

    pub fn can_promote(&self, from: (usize, usize), to: (usize, usize)) -> bool {
        if let Some(piece) = self.tiles[from.0][from.1] {
            if piece.piece != PieceType::Pawn {return false;}
            let promotion_rank = match piece.color {
                PieceColor::White => 7usize,
                PieceColor::Black => 0usize,
            };
            if to.0 != promotion_rank {return false;}
            return true;
        }
        return false;
    }

    pub fn make_move(&mut self, from: (usize, usize), to: (usize, usize), promote: Option<Piece>) {
        if self.is_legal_move(from, to) {
            if let Some(piece) = self.tiles[from.0][from.1] {
                if self.is_en_passant(from, to) {
                    let vert_dif: isize = match piece.color {
                        PieceColor::White => -1,
                        PieceColor::Black => 1,
                    };
                    self.tiles[(to.0 as isize + vert_dif) as usize][to.1] = None;
                }
                //if self.is_check_after(piece.color, from, to) {return;}

                let ccheck = self.can_castle(from, to);
                if ccheck.cancastle {
                    let dest = ccheck.rookdestination.unwrap();
                    let orig = ccheck.rooksource.unwrap();
                    self.tiles[dest.0][dest.1] = self.tiles[orig.0][orig.1];
                    self.tiles[orig.0][orig.1] = None;
                }

                //Handle promotion
                let mut dest_piece = self.tiles[from.0][from.1].unwrap();
                if let Some(promote_to) = promote {
                    if self.can_promote(from, to) {dest_piece = promote_to;}
                }
                
                self.tiles[to.0][to.1] = Some(dest_piece);
                self.tiles[from.0][from.1] = None;

                let thismove = Move{from,to,color:piece.color,piece};
                self.lastmove = Some(thismove);
                self.movelist.push(thismove);
            }
        }
    }

    pub fn can_castle(&self, from: (usize, usize), to: (usize, usize)) -> CastleCheck {
        if let Some(piece) = self.tiles[from.0][from.1] {
            if piece.piece != PieceType::King {
                return CastleCheck::new(false);
            }
            let (possible_castle, required_starting_square) = match piece.color {
                PieceColor::White => ([(0usize,2usize),(0usize,6usize)],(0usize,4usize)),
                PieceColor::Black => ([(7usize,2usize),(7usize,6usize)],(7usize,4usize))
            };

            //If is not attempting to castle, then just use our normal attacking logic.
            if (required_starting_square != from) || !(possible_castle.contains(&to)) {
                return CastleCheck::new(false)
            }
            //Find relevant rook to castle with, where the rook should end up, and which direction the king is traveling.
            let (rook, dest, dir) = match to {
                (0usize,2usize) => ((0usize,0usize), (0usize,3usize), -1i32),
                (0usize,6usize) => ((0usize,7usize), (0usize,5usize), 1),
                (7usize,2usize) => ((7usize,0usize), (7usize,3usize), -1),
                (7usize,6usize) => ((7usize,7usize), (7usize,5usize), 1),
                _ => return CastleCheck::new(false),
            };

            let kingrook = [from,rook];
            
            //If the king or rook has moved at all, castling is illegal.
            for pastmove in &self.movelist {
                if kingrook.contains(&pastmove.from) {
                    return CastleCheck::new(false);
                }
            }
            
            //Check the squares between us and our destination, inclusive of the destination.
            for i in 1..3 {
                let file = (4 + i * dir) as usize;
                
                //If there is a piece in the way, we can't castle.
                if let Some(_) = self.tiles[to.0][file] {return CastleCheck::new(false);}

                //We also may not castle through or into check.
                for x in 0..8 {
                    for y in 0..8 {
                        if self.is_attacking((x,y), (to.0,file), Some(piece.color)) {return CastleCheck::new(false);}
                    }
                }
            }

            //Finally, we may not castle out of check.
            if self.is_check(piece.color) {return CastleCheck::new(false);}

            //If all of the checks passed, then castling is legal.
            return CastleCheck{cancastle: true, rooksource: Some(rook), rookdestination: Some(dest)};
        }
        return CastleCheck::new(false);
    }

    pub fn is_en_passant(&self, from: (usize, usize), to: (usize, usize)) -> bool {
        if let Some(lastmove) = self.lastmove {
            if let Some(piece) = self.tiles[from.0][from.1] {
                //Pawn must have been moved last turn
                if lastmove.piece.piece != PieceType::Pawn {return false;}

                let lastmove_vert_dif = (lastmove.to.0 as isize) - (lastmove.from.0 as isize);
                //Other pawn must have moved two spaces last turn.
                if lastmove_vert_dif.abs() != 2 {return false;}

                let from_hori_dif = (lastmove.to.1 as isize) - (from.1 as isize);
                let to_hori_dif = (lastmove.to.1 as isize) - (to.1 as isize);
                //We should end up on the same file as the last moved pawn, but should start adjacent to it.
                if (from_hori_dif.abs() != 1) || (to_hori_dif.abs() != 0) {return false;}
                
                let to_vert_dif = (to.0 as isize) - (lastmove.to.0 as isize);
                let required_vert_dif: isize = match piece.color {
                    PieceColor::White => 1,
                    PieceColor::Black => -1,
                };
                //Our pawn should end up one past the other.
                if to_vert_dif != required_vert_dif {return false;}
                return true;
            }
        }
        return false;
    }

    pub fn is_legal_move(&self, from: (usize, usize), to: (usize, usize)) -> bool {
        
        if let Some(piece) = self.tiles[from.0][from.1] {
            if let Some(lastmove) = self.lastmove {
                if lastmove.color == piece.color {
                    return false;
                }
            } else if piece.color == PieceColor::Black {
                return false;
            }
            match piece.piece {
                //Pawns need special logic because they attack and move differently
                PieceType::Pawn => {
                    if let Some(_) = self.tiles[to.0][to.1] {
                        return if self.is_attacking(from, to, None) {self.final_move_validation(from,to)} else {false};
                    }
                    let vert_dif = (to.0 as isize) - (from.0 as isize);
                    let hori_dif = (to.1 as isize) - (from.1 as isize);
                    match piece.color {
                        PieceColor::White => {
                            if(vert_dif == 1) && (hori_dif == 0) {return self.final_move_validation(from,to);}
                            if(from.0 == 1) && (vert_dif == 2) && (hori_dif == 0) {
                                if let None = self.tiles[2][from.1] {return self.final_move_validation(from,to);}
                            }
                        },
                        PieceColor::Black => {
                            if(vert_dif == -1) && (hori_dif == 0) {return self.final_move_validation(from,to);}
                            if(from.0 == 6) && (vert_dif == -2) && (hori_dif == 0) {
                                if let None = self.tiles[5][from.1] {return self.final_move_validation(from,to);}
                            }
                        }
                    }
                    //Only other legal move would be en passant
                    return if self.is_en_passant(from,to) {self.final_move_validation(from,to)} else {false};
                },
                //King needs castling logic
                PieceType::King => {
                    let ccheck = self.can_castle(from,to);
                    if ccheck.cancastle {return self.final_move_validation(from,to);}
                    //If not castling, return true if we are attacking that square.
                    else {return if self.is_attacking(from,to, None) {self.final_move_validation(from,to)} else {false};}
                }
                //Everything else can only move to where they are attacking.
                _ => return if self.is_attacking(from,to, None) {self.final_move_validation(from,to)} else {false},
            }
        } else {
            return false;
        }
    }

    pub fn king_coords(&self, color: PieceColor) -> (usize,usize) {
        let mut king_coords: (usize,usize) = (0,0);
        'outerloop: for x in 0..8 {
            for y in 0..8 {
                if let Some(piece) = self.tiles[x][y] {
                    if (color == piece.color) && (piece.piece == PieceType::King) {
                        king_coords = (x,y);
                        break 'outerloop;
                    }
                }
            }
        }
        return king_coords;
    }

    pub fn final_move_validation(&self, from: (usize, usize), to: (usize, usize)) -> bool {
        let orig = self.tiles[from.0][from.1].unwrap();
        if self.is_check_after(orig.color, from, to) {return false;}
        return true;
    }

    pub fn is_check_after(&self, color: PieceColor, from: (usize, usize), to: (usize, usize)) -> bool {
        let orig = self.tiles[from.0][from.1].unwrap();
        let mut copy = self.clone();
        copy.tiles[to.0][to.1] = Some(orig);
        copy.tiles[from.0][from.1] = None;
        return copy.is_check(color);
    }

    pub fn is_check(&self, color: PieceColor) -> bool{
        let king_coords = self.king_coords(color);
        for x in 0..8 {
            for y in 0..8 {
                if self.is_attacking((x,y), king_coords, None) {
                    return true;
                }
            }
        }
        return false;
    }

    pub fn is_attacking(&self,source: (usize,usize), target: (usize,usize), override_source_color: Option<PieceColor>) -> bool {
        if source == target {
            return false;
        }
        if let Some(piece) = self.tiles[source.0][source.1] {
            if let Some(target_piece) = self.tiles[target.0][target.1] {
                if target_piece.color == piece.color {return false;}
            }
            if let Some(color) = override_source_color {
                if piece.color == color {return false;}
            }
            match piece.piece {
                PieceType::King => {
                    //Target square must simply be within 1 tile distance.
                    if (((target.0 as isize) - (source.0 as isize)).abs() <= 1) && (((target.1 as isize) - (source.1 as isize)).abs() <= 1) {
                        return true;
                    } else {
                        return false;
                    }
                },
                PieceType::Pawn => {
                    //Target piece must be 1 rank above/below the pawn depending on color. Not considering en passant because this is for the purposes of check/checkmate.
                    match piece.color {
                        PieceColor::White => {
                            if (target.0 as isize) - (source.0 as isize) != 1 {
                                return false;
                            }
                        },
                        PieceColor::Black => {
                            if (source.0 as isize) - (target.0 as isize) != 1 {
                                return false;
                            }
                        }
                    }
                    //Horizontal difference must be exactly one in addition to above condition, meaning that the target square is in front (relative) of and diagonally adjacent to the target square.
                    let hori_dif = ((target.1 as isize) - (source.1 as isize)).abs();
                    if hori_dif == 1 {
                        return true
                    } else {
                        return false
                    }
                },
                PieceType::Rook => {
                    //Target square must be on the same rank or file as the attacking rook.
                    if (target.0 != source.0) && (target.1 != source.1) {
                        return false;
                    }
                    if target.0 == source.0 {
                        //Check which direction
                        let hori_dif = (target.1 as isize) - (source.1 as isize);
                        //If directly adjacent then is being attacked.
                        if hori_dif.abs() == 1 {
                            return true;
                        }
                        //Otherwise check if any other pieces are in the way.
                        let range = if hori_dif > 0 {(source.1+1)..target.1} else {(target.1+1)..source.1};
                        for i in range {
                            //If we find a piece, return false.
                            if let Some(_) = self.tiles[source.0][i] {
                                return false;
                            }
                        }
                        //If we made it through the loop, then nothing is between us.
                        return true;
                    } else {
                        //Check which direction
                        let vert_dif = (target.0 as isize) - (source.0 as isize);
                        //If directly adjacent then is being attacked.
                        if vert_dif.abs() == 1 {
                            return true;
                        }
                        //Otherwise check if any other pieces are in the way.
                        let range = if vert_dif > 0 {(source.0+1)..target.0} else {(target.0+1)..source.0};
                        for i in range {
                            //If we find a piece, return false.
                            if let Some(_) = self.tiles[i][source.1] {
                                return false;
                            }
                        }
                        //If we made it through the loop, then nothing is between us.
                        return true;
                    }
                },
                PieceType::Knight => {
                    let hori_dif = ((target.1 as isize) - (source.1 as isize)).abs();
                    let vert_dif = ((target.0 as isize) - (source.0 as isize)).abs();
                    //Makes an L shape.
                    if (hori_dif == 2 && vert_dif == 1) || (hori_dif == 1 && vert_dif==2) {
                        return true;
                    }
                    return false;
                }
                PieceType::Bishop => {
                    let hori_dif = (target.1 as isize) - (source.1 as isize);
                    let vert_dif = (target.0 as isize) - (source.0 as isize);
                    //The absolutes of the vertical and horizontal differences must be equal if the pieces share a diagonal.
                    if !(hori_dif.abs() == vert_dif.abs()) {
                        return false;
                    }
                    //If diagonally adjacent, then it is being attacked
                    if hori_dif.abs() == 1 {
                        return true;
                    }
                    //Get the signs for direction to travel in
                    let hori_dir = hori_dif.signum();
                    let vert_dir = vert_dif.signum();
                    //Check to see if any pieces are in the way
                    for i in 1..hori_dif.abs(){
                        //If a piece is in the way, then return false.
                        if let Some(_) = self.tiles[(target.0 as isize - (i * vert_dir)) as usize][(target.1 as isize - (i * hori_dir)) as usize] {
                            return false;
                        }
                    }
                    //Otherwise if we manage to make it through the loop, return true.
                    return true;
                }
                PieceType::Queen => {
                    let hori_dif = (target.1 as isize) - (source.1 as isize);
                    let vert_dif = (target.0 as isize) - (source.0 as isize);
                    //If they are diagonal, use bishop logic.
                    if hori_dif.abs() == vert_dif.abs() {
                        //If diagonally adjacent, then it is being attacked
                        if hori_dif.abs() == 1 {
                            return true;
                        }
                        //Get the signs for direction to travel in
                        let hori_dir = hori_dif.signum();
                        let vert_dir = vert_dif.signum();
                        //Check to see if any pieces are in the way
                        for i in 1..hori_dif.abs(){
                            //If a piece is in the way, then return false.
                            if let Some(_) = self.tiles[(target.0 as isize - (i * vert_dir)) as usize][(target.1 as isize - (i * hori_dir)) as usize] {
                                return false;
                            }
                        }
                        //Otherwise if we manage to make it through the loop, return true.
                        return true;
                    }
                    //If they are orthogonal, then use rook logic.
                    if (hori_dif == 0) || (vert_dif == 0) {
                        if target.0 == source.0 {
                            //Check which direction
                            let hori_dif = (target.1 as isize) - (source.1 as isize);
                            //If directly adjacent then is being attacked.
                            if hori_dif.abs() == 1 {
                                return true;
                            }
                            //Otherwise check if any other pieces are in the way.
                            let range = if hori_dif > 0 {(source.1+1)..target.1} else {(target.1+1)..source.1};
                            for i in range {
                                //If we find a piece, return false.
                                if let Some(_) = self.tiles[source.0][i] {
                                    return false;
                                }
                            }
                            //If we made it through the loop, then nothing is between us.
                            return true;
                        } else {
                            //Check which direction
                            let vert_dif = (target.0 as isize) - (source.0 as isize);
                            //If directly adjacent then is being attacked.
                            if vert_dif.abs() == 1 {
                                return true;
                            }
                            //Otherwise check if any other pieces are in the way.
                            let range = if vert_dif > 0 {(source.0+1)..target.0} else {(target.0+1)..source.0};
                            for i in range {
                                //If we find a piece, return false.
                                if let Some(_) = self.tiles[i][source.1] {
                                    return false;
                                }
                            }
                            //If we made it through the loop, then nothing is between us.
                            return true;
                        }
                    }
                    //If they are neither, then return false.
                    return false;
                }
            }
        } else {
            return false;
        }
        
    }
}

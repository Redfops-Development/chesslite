use std::{ops::Not, io::{Write,Read}, process::{Stdio, ChildStdout}};

use bevy::prelude::*;

use std::process::Command;

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

impl PieceType{
    pub fn to_char(&self) -> char {
        match self {
            PieceType::King => 'k',
            PieceType::Queen => 'q',
            PieceType::Rook => 'r',
            PieceType::Bishop => 'b',
            PieceType::Knight => 'n',
            PieceType::Pawn => 'p'
        }
    }
}

#[derive(Clone,Copy,PartialEq,Debug)]
pub enum PieceColor{
    Black,
    White
}

impl PieceColor{
    fn to_char(&self) -> char {
        if let PieceColor::White = self {'w'} else {'b'}
    }
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

impl Piece{
    fn to_char(&self) -> char {
        let c = match self.piece {
            PieceType::King => 'k',
            PieceType::Queen => 'q',
            PieceType::Rook => 'r',
            PieceType::Bishop => 'b',
            PieceType::Knight => 'n',
            PieceType::Pawn => 'p'
        };
        if let PieceColor::White = self.color {
            return c.to_uppercase().next().unwrap();
        }
        return c;
    }
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

pub struct SimpleMove {
    pub from: (usize, usize),
    pub to: (usize, usize),
    pub promotion: Option<PieceType>,
}

impl SimpleMove {
    pub fn from_algebraic(algebraic: &str) -> Self{
        let mut chars = algebraic.chars();
        let a = chars.next().unwrap() as u32 - 97;
        let b = chars.next().unwrap().to_digit(10).unwrap() - 1;
        let c = chars.next().unwrap() as u32 - 97;
        let d = chars.next().unwrap().to_digit(10).unwrap() - 1;
        
        if let Some(e) = chars.next() {
            let ptype = match e {
                'q' => PieceType::Queen,
                'n' => PieceType::Knight,
                'k' => PieceType::Knight,
                'b' => PieceType::Bishop,
                'r' => PieceType::Rook,
                _ => PieceType::Queen,
            };
            return Self {from: (b as usize, a as usize), to: (d as usize, c as usize), promotion: Some(ptype)};
        }
        return Self {from: (b as usize, a as usize), to: (d as usize, c as usize), promotion: None};
    }
    pub fn to_algebraic(&self) -> String {
        let mut out = String::new();
        out.push(((self.from.1 + 97) as u8) as char);
        out += &(self.from.0 + 1).to_string();
        out.push(((self.to.1 + 97) as u8) as char);
        out += &(self.to.0 + 1).to_string();
        if let Some(piece) = self.promotion {
            out.push(piece.to_char())
        }
        return out;
    }
}

#[derive(Clone,Copy)]
pub struct Move {
    pub from: (usize, usize),
    pub to: (usize, usize),
    pub color: PieceColor,
    pub piece: Piece,
    pub capture: bool,
}

/* 
impl Move {
    pub fn to_algebraic(&self) -> String {
        let out1 = toalgebraicsquare(self.from);
        let out2 = toalgebraic
    }
}*/

pub fn toalgebraicsquare(square: (usize, usize)) -> String {
    let mut out = String::new();
    out.push(((square.1 + 97) as u8) as char);
    out += &(square.0 + 1).to_string();
    return out;
}

fn read_lines(stdout: Option<&mut ChildStdout>) -> String{
    let mut strings = String::new();
    let stdout = stdout.unwrap();
    loop {
        let mut s = String::new();
        let mut buf: Vec<u8> = vec![0];
        
        loop {
            stdout.read(&mut buf).unwrap();
            s.push(buf[0] as char);
            if buf[0] == '\n' as u8 {
                break
            }
        }
        if s.starts_with("bestmove") {
            return s;
        }
        strings += &s;
    }
}

#[derive(Clone,Copy)]
pub struct CastleAvailability{
    pub white_kingside: bool,
    pub white_queenside: bool,
    pub black_kingside: bool,
    pub black_queenside: bool,
}

impl CastleAvailability {
    pub fn new() -> Self {
        CastleAvailability {
            white_kingside: true,
            white_queenside: true,
            black_kingside: true,
            black_queenside: true,
        }
    }

    pub fn set_availability(&mut self, color: PieceColor, kingside: bool, availability: bool){
        match color {
            PieceColor::Black => {
                if kingside {self.black_kingside = availability} else {self.black_queenside = availability}
            },
            PieceColor::White => {
                if kingside {self.white_kingside = availability} else {self.white_queenside = availability}
            }
        }
    }

    pub fn any_castle(&self, color: PieceColor) -> bool {
        match color {
            PieceColor::Black => {
                self.black_kingside || self.black_queenside
            },
            PieceColor::White => {
                self.white_kingside || self.white_queenside
            }
        }
    }

    pub fn check_availability(&self, color: PieceColor, kingside: bool) -> bool {
        match color {
            PieceColor::Black => {if kingside {self.black_kingside} else {self.black_queenside}},
            PieceColor::White => {if kingside {self.white_kingside} else {self.white_queenside}}
        }
    }
}

#[derive(Resource,Clone)]
pub struct Board {
    pub tiles: [[Option<Piece>;8];8],
    pub lastmove: Option<Move>,
    pub movelist: Vec<Move>,
    pub halfmoves: usize,
    pub fullmoves: usize,
    pub castles: CastleAvailability,
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
            halfmoves: 0usize,
            fullmoves: 1usize,
            castles: CastleAvailability::new(),
        }
    }

    pub fn fen(&self) -> String {
        let mut out = String::new();
        for i in (0..8).rev() {
            let rank = self.tiles[i];
            let mut num = 0;
            let mut empty = 0;
            for square in rank {
                if let Some(piece) = square {
                    if empty > 0 {
                        out += &empty.to_string();
                        empty = 0;
                    }
                    out.push(piece.to_char());
                } else {
                    empty += 1;
                    if num == 7 {
                        out += &empty.to_string();
                    }
                }
                num += 1;
            }
            if i != 0 {
                out.push('/');
            }
        }
        out += " ";
        out.push(self.to_move().to_char());
        out += " ";

        let chars = ['K','Q','k','q'];

        let mut anycastles = false;

        for char in chars {
            match char {
                'K' => {if self.castles.white_kingside {anycastles = true; out.push(char)}},
                'Q' => {if self.castles.white_queenside {anycastles = true; out.push(char)}},
                'k' => {if self.castles.black_kingside {anycastles = true; out.push(char)}},
                'q' => {if self.castles.black_queenside {anycastles = true; out.push(char)}},
                _ => continue
            }
        }

        if !anycastles {
            out.push('-');
        }

        out += " ";

        let mut en_passant = "-".to_string();

        if let Some(lastmove) = self.lastmove {
            if [1usize,6usize].contains(&lastmove.from.0) && [3usize,4usize].contains(&lastmove.to.0) {
                if let Some(piece) = self.tiles[lastmove.to.0][lastmove.to.1] {
                    if let PieceType::Pawn = piece.piece {
                        let mut en_passant_square = lastmove.to;
                        if lastmove.to.0 == 3usize {
                            en_passant_square.0 = 2;
                            en_passant = toalgebraicsquare(en_passant_square);
                        } else {
                            en_passant_square.0 = 5;
                            en_passant = toalgebraicsquare(en_passant_square);
                        }
                    }
                }
            }
        }

        out += &en_passant;

        out += " ";

        out += &self.halfmoves.to_string();

        out += " ";

        out += &self.fullmoves.to_string();

        return out;
    }

    pub fn engine_move(&mut self) {
        /*let mut command = Command::new("stockfish/stockfish.exe").spawn().unwrap();
        let mut stdin = command.stdin.take().unwrap();
        let mut stdout = command.stdout.take().unwrap();
        stdin.write_all("isready".as_bytes()).unwrap();
        let mut ready = String::new();
        stdout.read_to_string(&mut ready).unwrap();
        println!("{}",ready);

        stdin.write_all(("position fen ".to_owned() + &self.fen()).as_bytes()).unwrap();
        let mut ready2 = String::new();
        stdout.read_to_string(&mut ready2).unwrap();
        println!("{}",ready2);

        stdin.write_all("go movetime 1000".as_bytes()).unwrap();
        let mut out = String::new();
        stdout.read_to_string(&mut out).unwrap();*/

        let mut cmd = Command::new("stockfish/stockfish.exe")
                          .stdin(Stdio::piped())
                          .stdout(Stdio::piped())
                          .spawn()
                          .expect("Unable to run engine");
        //let s = read_line(cmd.stdout.as_mut());
        //println!("{}",s);
        let fen = self.fen();
        cmd.stdin.as_mut().unwrap().write_fmt(format_args!("isready\n")).unwrap();
        cmd.stdin.as_mut().unwrap().write_fmt(format_args!("position fen {fen}\n")).unwrap();
        cmd.stdin.as_mut().unwrap().write_fmt(format_args!("go movetime 1000\n")).unwrap();
        let s = read_lines(cmd.stdout.as_mut());
    
        let out = s.split(' ').nth(1).unwrap();

        //let out = String::from_utf8_lossy(&output);
        println!("output {}", out);
        let bmove = SimpleMove::from_algebraic(&out);
        let promote = if let Some(piece) = bmove.promotion {Some(Piece{piece, color: self.to_move()})} else {None};
        println!("{:?},{:?}",bmove.from,bmove.to);
        self.make_move(bmove.from,bmove.to,promote);
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
            return self.is_legal_move(from, to);
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

                //Is capture?
                let captured = if let Some(_) = self.tiles[to.0][to.1] {true} else {false};

                //Update halfmove count
                if captured || piece.piece == PieceType::Pawn {
                    self.halfmoves = 0;
                } else {
                    self.halfmoves += 1;
                }

                //Update fullmove count
                if let PieceColor::Black = piece.color {
                    self.fullmoves += 1;
                }

                //Update castle availability
                if self.castles.any_castle(piece.color) && [PieceType::King,PieceType::Rook].contains(&piece.piece) {
                    //Get rook starting squares for the specific side.
                    let kingside_rook = if let PieceColor::Black = piece.color {(7usize,7usize)} else {(0usize, 7usize)};
                    let queenside_rook = if let PieceColor::Black = piece.color {(7usize,0usize)} else {(0usize, 0usize)};

                    //If king moves at all, no more castling on either side.
                    if let PieceType::King = piece.piece {
                        self.castles.set_availability(piece.color, true, false);
                        self.castles.set_availability(piece.color, false, false);
                    } else { //Otherwise if a rook moves from its starting position, no more castling on that side.
                        if from == kingside_rook {
                            self.castles.set_availability(piece.color, true, false);
                        }
                        if from == queenside_rook {
                            self.castles.set_availability(piece.color, false, false);
                        }
                    }
                }
                
                self.tiles[to.0][to.1] = Some(dest_piece);
                self.tiles[from.0][from.1] = None;

                let thismove = Move{from,to,color:piece.color,piece, capture:captured};
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
            
            //If the king or rook has moved at all, castling is illegal.
            if to.1 == 2usize && !self.castles.check_availability(piece.color, false) {
                return CastleCheck::new(false)
            }

            if to.1 == 6usize && !self.castles.check_availability(piece.color, true) {
                return CastleCheck::new(false)
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

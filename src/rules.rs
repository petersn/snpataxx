include!(concat!(env!("OUT_DIR"), "/tables.rs"));

#[derive(Clone, Copy)]
pub enum Color {
  Black,
  White,
}

impl Color {
  pub fn other_player(self) -> Color {
    match self {
      Color::Black => Color::White,
      Color::White => Color::Black,
    }
  }
}

#[derive(Clone)]
pub struct State {
  pub black_stones: u64,
  pub white_stones: u64,
  pub gaps: u64,
  pub to_move: Color,
}

#[derive(Clone, Copy)]
pub struct Spot(u8);

fn iter_bits(bitboard: &mut u64) -> Option<Spot> {
  let pos = bitboard.trailing_zeros();
  if pos == 64 {
    return None;
  }
  *bitboard &= *bitboard - 1;
  Some(Spot(pos as u8))
}

#[derive(Clone, Copy)]
pub struct Move {
  pub from: Spot,
  pub to: Spot,
}

impl State {
  pub fn new() -> State {
    State {
      black_stones: 0,
      white_stones: 0,
      gaps: 0,
      to_move: Color::Black,
    }
  }

  pub fn render(&self) {
    for y in 0..7 {
      for x in 0..7 {
        let pos = y * 8 + x;
        let mask = 1 << pos;
        if self.black_stones & mask != 0 {
          print!("\x1b[91mX\x1b[0m");
        } else if self.white_stones & mask != 0 {
          print!("\x1b[94mO\x1b[0m");
        } else if self.gaps & mask != 0 {
          print!("#");
        } else {
          print!(".");
        }
        if x != 6 {
          print!(" ");
        }
      }
      println!("");
    }
  }

  pub fn from_fen(fen: &str) -> Result<State, String> {
    let mut state = State::new();
    let mut chars = fen.chars();
    let mut i = 0;
    while let Some(c) = chars.next() {
      match c {
        '1'..='7' => i += c as u8 - '1' as u8,
        'x' => state.black_stones |= 1 << i,
        'o' => state.white_stones |= 1 << i,
        '-' => state.gaps |= 1 << i,
        '/' => {
          if i % 8 != 7 {
            return Err(format!("Misplaced slash i={}", i));
          }
        }
        ' ' => break,
        _ => return Err(format!("Invalid character in FEN: {}", c)),
      }
      i += 1;
    }
    if i != 6 * 8 + 7 {
      return Err("FEN too short".to_string());
    }
    match chars.next() {
      Some('x') => state.to_move = Color::Black,
      Some('o') => state.to_move = Color::White,
      Some(c) => return Err(format!("Invalid player to move: {}", c)),
      None => return Err("Missing player to move".to_string()),
    }
    // TODO: Parse half-move clock and full-move number.
    Ok(state)
  }

  pub fn move_gen(&self, moves: &mut Vec<Move>) {
    let unoccupied = !(self.black_stones | self.white_stones | self.gaps);
    let mut our_stones = match self.to_move {
      Color::Black => self.black_stones,
      Color::White => self.white_stones,
    };
    let mut single_moves = 0;
    while let Some(pos) = iter_bits(&mut our_stones) {
      single_moves |= MOORE_MASK[pos.0 as usize] & unoccupied;
      let mut double_moves = DOUBLE_MOVES_MASK[pos.0 as usize] & unoccupied;
      while let Some(to) = iter_bits(&mut double_moves) {
        moves.push(Move { from: pos, to });
      }
    }
    while let Some(to) = iter_bits(&mut single_moves) {
      moves.push(Move { from: to, to });
    }
  }

  pub fn sanity_check(&self) {
    if self.black_stones & self.white_stones != 0 {
      panic!("Black and white stones overlap");
    }
    if self.black_stones & self.gaps != 0 {
      panic!("Black stones and gaps overlap");
    }
    if self.white_stones & self.gaps != 0 {
      panic!("White stones and gaps overlap");
    }
  }

  pub fn make_move(&mut self, m: Move) -> Result<(), &'static str> {
    // Place the target stone.
    match self.to_move {
      Color::Black => self.black_stones |= 1 << m.to.0,
      Color::White => self.white_stones |= 1 << m.to.0,
    };
    // Remove the source stone if it's a double move.
    if m.from.0 != m.to.0 {
      match self.to_move {
        Color::Black => self.black_stones &= !(1 << m.from.0),
        Color::White => self.white_stones &= !(1 << m.from.0),
      }
    }
    // Capture neighbors.
    let captures = MOORE_MASK[m.to.0 as usize] & (self.black_stones | self.white_stones);
    self.black_stones &= !captures;
    self.white_stones &= !captures;
    match self.to_move {
      Color::Black => self.black_stones |= captures,
      Color::White => self.white_stones |= captures,
    }
    self.to_move = self.to_move.other_player();
    Ok(())
  }
}

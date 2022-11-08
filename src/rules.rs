use crate::rng::RNG_MULT;

include!(concat!(env!("OUT_DIR"), "/tables.rs"));

const ALL_CELLS_MASK: u64 = 0x7f7f7f7f7f7f7f;

#[derive(Clone, Copy, PartialEq, Eq)]
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
  pub gaps:         u64,
  pub to_move:      Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spot(u8);

impl Spot {
  pub fn from_uai(s: &str) -> Spot {
    let mut chars = s.chars();
    let letter = chars.next().unwrap();
    let number = chars.next().unwrap();
    if chars.next().is_some() {
      panic!("Invalid spot: {}", s);
    }
    let letter = letter as u8 - b'a';
    let number = number as u8 - b'1';
    if letter > 6 || number > 6 {
      panic!("Invalid spot: {}", s);
    }
    Spot(letter + 8 * number)
  }

  pub fn to_uai(self) -> String {
    let (x, y) = (self.0 % 8, self.0 / 8);
    let letter = (b'a' + x) as char;
    let number = (b'1' + (6 - y)) as char;
    format!("{}{}", letter, number)
  }
}

fn iter_bits(bitboard: &mut u64) -> Option<Spot> {
  let pos = bitboard.trailing_zeros();
  if pos == 64 {
    return None;
  }
  *bitboard &= *bitboard - 1;
  Some(Spot(pos as u8))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
  pub from: Spot,
  pub to:   Spot,
}

impl Move {
  pub const PASS: Self = Move {
    from: Spot(255),
    to:   Spot(255),
  };

  pub fn from_uai(uai: &str) -> Move {
    if uai == "0000" {
      return Move::PASS;
    }
    match uai.len() {
      4 => Move { from: Spot::from_uai(&uai[..2]), to: Spot::from_uai(&uai[2..]) },
      2 => {
        let spot = Spot::from_uai(uai);
        Move { from: spot, to: spot }
      }
      _ => panic!("Invalid move: {}", uai),
    }
  }

  pub fn to_uai(self) -> String {
    if self == Move::PASS {
      return "0000".to_string();
    }
    match self.from == self.to {
      true => self.from.to_uai(),
      false => format!("{}{}", self.from.to_uai(), self.to.to_uai()),
    }
  }
}

impl State {
  pub fn new() -> State {
    State {
      black_stones: 0,
      white_stones: 0,
      gaps:         0,
      to_move:      Color::Black,
    }
  }

  pub fn game_is_over(&self) -> bool {
    (self.black_stones | self.white_stones | self.gaps) == ALL_CELLS_MASK
  }

  pub fn get_winner(&self) -> Option<Color> {
    match self.game_is_over() {
      false => None,
      true => {
        let black_score = self.black_stones.count_ones();
        let white_score = self.white_stones.count_ones();
        match black_score.cmp(&white_score) {
          std::cmp::Ordering::Less => Some(Color::White),
          std::cmp::Ordering::Equal => panic!("Odd number of gaps lead to draw"),
          std::cmp::Ordering::Greater => Some(Color::Black),
        }
      }
    }
  }

  pub fn render(&self) -> String {
    let mut s = String::new();
    for y in 0..7 {
      for x in 0..7 {
        let pos = y * 8 + x;
        let mask = 1 << pos;
        if self.black_stones & mask != 0 {
          s.push_str("\x1b[91mX\x1b[0m");
        } else if self.white_stones & mask != 0 {
          s.push_str("\x1b[94mO\x1b[0m");
        } else if self.gaps & mask != 0 {
          s.push('#');
        } else {
          s.push('.');
        }
        if x != 6 {
          s.push(' ');
        }
      }
      s.push_str("\n");
    }
    match self.to_move {
      Color::Black => {
        s.push_str("\x1b[91mX\x1b[0m -- Black to move\n");
      }
      Color::White => {
        s.push_str("\x1b[94mO\x1b[0m -- White to move\n");
      }
    }
    s
  }

  pub fn get_hash(&self) -> u64 {
    let mut hash = 0;
    macro_rules! hash_in(
      ($x:expr) => {
        hash ^= $x;
        hash = hash.wrapping_mul(RNG_MULT);
        hash ^= hash >> 37;
      }
    );
    // We don't need to include the gaps, because they're fixed.
    hash_in!(self.black_stones);
    hash_in!(self.white_stones);
    hash_in!(self.to_move as u64);
    hash
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
    if moves.is_empty() {
      moves.push(Move::PASS);
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
    if m == Move::PASS {
      self.to_move = self.to_move.other_player();
      return Ok(());
    }
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

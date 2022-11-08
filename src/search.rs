use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crate::rng::Rng;
use crate::rules::{Color, Move, State};

struct FixedHashTable<const SIZE: usize, T> {
  table: Vec<(u64, T)>,
}

impl<const SIZE: usize, T> FixedHashTable<SIZE, T> {
  pub fn new() -> FixedHashTable<SIZE, T> {
    let mut table = Vec::with_capacity(SIZE);
    for _ in 0..SIZE {
      table.push((0, unsafe { std::mem::zeroed() }));
    }
    FixedHashTable {
      table,
    }
  }

  fn get(&self, key: u64) -> Option<&T> {
    let index = (key % SIZE as u64) as usize;
    match self.table[index].0 == key {
      true => Some(&self.table[index].1),
      false => None,
    }
  }

  fn set(&mut self, key: u64, value: T) {
    let index = (key % SIZE as u64) as usize;
    self.table[index] = (key, value);
  }
}

type Evaluation = i32;

const VERY_NEGATIVE_EVAL: Evaluation = -1_000_000_000;
const VERY_POSITIVE_EVAL: Evaluation = 1_000_000_000;

fn make_terminal_score_slightly_less_extreme(score: Evaluation) -> Evaluation {
  if score > 100_000 {
    score - 1
  } else if score < -100_000 {
    score + 1
  } else {
    score
  }
}

/// Returns an evaluation for the current player.
pub fn evaluate(state: &State) -> Evaluation {
  let mut score =
    100 * (state.black_stones.count_ones() as i32 - state.white_stones.count_ones() as i32);
  if state.game_is_over() {
    if score > 0 {
      score += 1_000_000;
    } else if score < 0 {
      score -= 1_000_000;
    }
  }
  match state.to_move {
    Color::Black => score,
    Color::White => -score,
  }
}

pub struct Engine {
  rng:              Rng,
  pub state:        State,
  move_order_table: FixedHashTable<{ 1 << 20 }, Move>,
  killer_moves:     [Option<Move>; 64],
  do_stop:          Arc<AtomicBool>,
  nodes:            u64,
}

impl Engine {
  pub fn new(seed: u64) -> Engine {
    Engine {
      nodes:            0,
      rng:              Rng::new(seed),
      state:            State::new(),
      move_order_table: FixedHashTable::new(),
      killer_moves:     [None; 64],
      do_stop:          Arc::new(AtomicBool::new(false)),
    }
  }

  pub fn make_move(&mut self, m: Move) -> Result<(), &'static str> {
    self.state.make_move(m)
  }

  pub fn set_position(&mut self, state: State) {
    self.state = state;
  }

  pub fn run_depth(&mut self, max_depth: u16) -> (Evaluation, Option<Move>) {
    self.nodes = 0;
    let mut p = (0, None);
    let state = self.state.clone();
    // Iterative deepening.
    for d in 1..=max_depth {
      p = self.pvs(d, &state, VERY_NEGATIVE_EVAL, VERY_POSITIVE_EVAL);
    }
    p
  }

  pub fn run_time(&mut self, movetime_ms: i32) -> (Evaluation, Option<Move>) {
    self.nodes = 0;
    let mut p = (0, None);
    let state = self.state.clone();
    self.do_stop.store(false, std::sync::atomic::Ordering::Relaxed);
    // Use a thread to stop the search after the given time.
    let do_stop = self.do_stop.clone();
    std::thread::spawn(move || {
      std::thread::sleep(std::time::Duration::from_millis(movetime_ms as u64));
      do_stop.store(true, std::sync::atomic::Ordering::Relaxed);
    });
    let start = std::time::Instant::now();
    // Iterative deepening.
    for d in 1.. {
      p = self.pvs(d, &state, VERY_NEGATIVE_EVAL, VERY_POSITIVE_EVAL);
      if self.do_stop.load(std::sync::atomic::Ordering::Relaxed) {
        break;
      }
    }
    println!("info time {} nodes {}", start.elapsed().as_millis(), self.nodes);
    p
  }

  pub fn run_time_managed(&mut self, ms_on_clock: i32, ms_increment: i32) -> (Evaluation, Option<Move>) {
    // Estimate the number of moves left in the game.
    let filled = (self.state.black_stones | self.state.white_stones | self.state.gaps).count_ones();
    let moves_left = 1 + (7*7 - filled) * 2;
    assert!(moves_left > 0);
    // Estimate the time we have left.
    let time_left = ms_on_clock + ms_increment * moves_left as i32;
    // Estimate the time we should spend on this move.
    let time_per_move = (time_left as f32 / moves_left as f32) as i32;
    // Make sure we don't spend more than half of our time on this move.
    let time_to_spend = time_per_move.min(ms_on_clock / 2);
    println!("moves_left: {moves_left} time_left: {}, time_per_move: {}, time_to_spend: {}", time_left, time_per_move, time_to_spend);
    // Run the search.
    self.run_time(time_to_spend)
  }

  pub fn pvs(
    &mut self,
    depth: u16,
    state: &State,
    mut alpha: Evaluation,
    beta: Evaluation,
  ) -> (Evaluation, Option<Move>) {
    self.nodes += 1;
    let random_bonus = || self.rng.generate_range(15) as i32;
    if state.game_is_over() || depth == 0 {
      return (evaluate(state) + random_bonus(), None);
    }

    let mut moves = Vec::new();
    state.move_gen(&mut moves);
    if moves.is_empty() {
      return (evaluate(state) + random_bonus(), None);
    }

    // Sort moves by score.
    let state_hash = state.get_hash();
    let mot_move = self.move_order_table.get(state_hash).copied();
    let killer_move = self.killer_moves[depth as usize];
    moves.sort_by_key(|m| match (mot_move, killer_move) {
      (Some(mot_move), _) if mot_move == *m => 2,
      (_, Some(killer_move)) if killer_move == *m => 1,
      _ => 0,
    });

    let mut first = true;
    let mut best_score = VERY_NEGATIVE_EVAL;
    let mut best_move = None;
    for m in moves {
      let mut new_state = state.clone();
      new_state.make_move(m).unwrap();
      // Recurse on subtrees.
      let mut score;
      if first {
        score = -self.pvs(depth - 1, &new_state, -beta, -alpha).0;
      } else {
        score = -self.pvs(depth - 1, &new_state, -alpha - 1, -alpha).0;
        if alpha < score && score < beta {
          score = -self.pvs(depth - 1, &new_state, -beta, -score).0;
        }
      }
      // Evaluate cut-offs, etc.
      if score > best_score {
        best_score = score;
        best_move = Some(m);
      }
      if score > alpha {
        alpha = score;
        self.move_order_table.set(state_hash, m);
      }
      if alpha >= beta {
        self.killer_moves[depth as usize] = Some(m);
        break;
      }
      // If we're out of time then stop early.
      if self.do_stop.load(std::sync::atomic::Ordering::Relaxed) {
        break;
      }
      first = false;
    }

    // We slightly decrease terminal scores to make sure we pick mate-in-2 over mate-in-3.
    (make_terminal_score_slightly_less_extreme(alpha), best_move)
  }
}

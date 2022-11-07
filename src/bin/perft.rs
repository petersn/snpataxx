use snpataxx::rules::State;

fn perft(depth: usize, state: State) -> usize {
  if depth == 0 {
    return 1;
  }
  let mut moves = Vec::new();
  state.move_gen(&mut moves);
  if depth == 1 {
    return moves.len();
  }
  let mut total = 0;
  for m in moves {
    let mut new_state = state.clone();
    new_state.make_move(m).unwrap();
    new_state.sanity_check();
    total += perft(depth - 1, new_state);
  }
  total
}

fn main() {
  let state = snpataxx::rules::State::from_fen("x5o/7/7/7/7/7/o5x x 0 1").unwrap();
  println!("{}", perft(6, state));
  //for i in 0..6 {
  //  println!("{} {}", i, perft(i, state.clone()));
  //}
}

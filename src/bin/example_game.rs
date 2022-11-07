use snpataxx::rules::{State, Move};
use snpataxx::search::Engine;

fn main() {
  let mut engine = Engine::new(rand::random());
  engine.set_position(State::from_fen("x5o/7/7/7/7/7/o5x x 0 1").unwrap());
  loop {
    println!("{}", engine.state.render());
    let (eval, m) = engine.run(3);
    let m = match m {
      Some(m) => m,
      None => break,
    };
    println!("{} [eval: {}]", m.to_uai(), eval);
    engine.make_move(m).unwrap();
    // Wait for the user to hit enter.
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
  }
}

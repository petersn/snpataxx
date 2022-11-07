use std::{collections::HashMap, io::BufRead};

fn main() {
  let stdin = std::io::stdin();
  let mut options = HashMap::new();
  let mut engine = snpataxx::search::Engine::new(rand::random());

  for line in stdin.lock().lines().map(|r| r.unwrap()) {
    let tokens = line.split_whitespace().collect::<Vec<_>>();
    if tokens.is_empty() {
      continue;
    }
    match tokens[0] {
      "uai" => {
        println!("id name snpataxx");
        println!("id author Peter Schmidt-Nielsen");
        println!("uaiok");
      }
      "uaiok" => {}
      "isready" => println!("readyok"),
      "quit" => break,
      "setoption" => {
        assert_eq!(tokens[1], "name");
        assert_eq!(tokens[3], "value");
        let name = tokens[2];
        let value = tokens[4];
        options.insert(name.to_string(), value.to_string());
      }
      "dbg" => {
        engine.state.render();
      }
      "position" => match tokens[1] {
        "startpos" => {
          engine = snpataxx::search::Engine::new(rand::random());
          if tokens.len() > 2 {
            assert_eq!(tokens[2], "moves");
            let moves = &tokens[3..];
            for m in moves {
              let m = snpataxx::rules::Move::from_uai(m);
              engine.make_move(m).unwrap();
            }
          }
        }
        "fen" => {
          let fen_string = tokens[2..tokens.len() - 1].join(" ");
          let state = snpataxx::rules::State::from_fen(&fen_string).unwrap();
          engine.set_position(state);
        }
        _ => panic!("Unknown position command: {}", line),
      }
      "go" => {
        let mut depth = 3;
        let mut infinite = false;
        for i in 1..tokens.len() {
          match tokens[i] {
            "depth" => depth = tokens[i + 1].parse().unwrap(),
            "infinite" => infinite = true,
            _ => (),
          }
        }
        let (score, m) = engine.run(depth);
        match m {
          Some(m) => {
            println!("bestmove {}", m.to_uai());
            println!("info score cp {} pv {}", score, m.to_uai());
            engine.make_move(m).unwrap();
          }
          None => println!("bestmove 0000"),
        }
      }
      _ => panic!("Unknown command: {}", line),
    }
  }
}

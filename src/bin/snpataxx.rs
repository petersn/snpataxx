use std::{collections::HashMap, io::BufRead};

use snpataxx::{
  rules::{Color, State},
  search::Engine,
};

fn main() {
  let stdin = std::io::stdin();
  let mut options = HashMap::new();
  let mut engine = Engine::new(rand::random());
  engine.set_position(State::from_fen("x5o/7/7/7/7/7/o5x x 0 1").unwrap());

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
      "uainewgame" => engine = Engine::new(rand::random()),
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
          engine = Engine::new(rand::random());
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
      },
      "go" => {
        let mut depth = None;
        let mut btime = 10;
        let mut wtime = 10;
        let mut binc = 1000;
        let mut winc = 1000;
        let mut movetime = None;
        for i in 1..tokens.len() {
          match tokens[i] {
            "depth" => depth = Some(tokens[i + 1].parse().unwrap()),
            "btime" => btime = tokens[i + 1].parse().unwrap(),
            "wtime" => wtime = tokens[i + 1].parse().unwrap(),
            "binc" => binc = tokens[i + 1].parse().unwrap(),
            "winc" => winc = tokens[i + 1].parse().unwrap(),
            "movetime" => movetime = Some(tokens[i + 1].parse().unwrap()),
            _ => (),
          }
        }

        let (score, m) = match (depth, movetime) {
          (Some(_), Some(_)) => panic!("Cannot specify both depth and movetime"),
          // If we have a specified depth, use that.
          (Some(depth), _) => engine.run_depth(depth),
          // If we have a specified time, use that.
          (_, Some(movetime)) => engine.run_time(movetime),
          // Otherwise, use time controls.
          (None, None) => {
            let (time, inc) = match engine.state.to_move {
              Color::White => (wtime, winc),
              Color::Black => (btime, binc),
            };
            engine.run_time_managed(time, inc)
          }
        };

        if m == Some(snpataxx::rules::Move::PASS) {
          // Make sure we have no moves!
          let mut moves = vec![];
          engine.state.move_gen(&mut moves);
          if moves.len() > 0 {
            panic!(
              "PASS move when we have other moves: {:?}\n{}",
              moves,
              engine.state.render()
            );
          }
        }
        match m {
          Some(m) => {
            println!("bestmove {}", m.to_uai());
            println!("info score cp {} pv {}", score, m.to_uai());
            engine.make_move(m).unwrap();
          }
          None => println!("bestmove xyzw"),
        }
      }
      "stop" => {}
      _ => panic!("Unknown command: {}", line),
    }
  }
}

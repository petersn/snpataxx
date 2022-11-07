// build.rs

use std::env;
use std::fs;
use std::path::Path;

fn main() {
  let mut moore = vec![0u64; 64];
  let mut double_moves = vec![0u64; 64];
  let all_cells: Vec<(i32, i32)> = (0..7).flat_map(|y| (0..7).map(move |x| (x, y))).collect();
  for (from_x, from_y) in &all_cells {
    for (to_x, to_y) in &all_cells {
      if from_x == to_x && from_y == to_y {
        continue;
      }
      let (dx, dy) = ((to_x - from_x).abs(), (to_y - from_y).abs());
      if dx <= 1 && dy <= 1 {
        moore[(from_x + from_y * 8) as usize] |= 1 << (to_x + to_y * 8);
      } else if dx <= 2 && dy <= 2 {
        double_moves[(from_x + from_y * 8) as usize] |= 1 << (to_x + to_y * 8);
      }
    }
  }

  let format = |numbers: &[u64]| {
    numbers.iter().map(|x| format!("0x{:016x}", x)).collect::<Vec<_>>().join(", ")
  };

  let formatted_moore = format(&moore);
  let formatted_double_moves = format(&double_moves);

  let code = format!(
    r#"
      pub const MOORE_MASK: [u64; 64] = [
        {formatted_moore}
      ];
      pub const DOUBLE_MOVES_MASK: [u64; 64] = [
        {formatted_double_moves}
      ];
    "#,
  );

  let out_dir = env::var_os("OUT_DIR").unwrap();
  let dest_path = Path::new(&out_dir).join("tables.rs");
  fs::write(&dest_path, code).unwrap();
  println!("cargo:rerun-if-changed=build.rs");
}

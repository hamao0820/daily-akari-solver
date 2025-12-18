use std::io::Read;

use akari::{Field, Solver, solver};

fn main() -> Result<(), &'static str> {
    let (h, w, s) = get_input();

    let solver = solver::CFS::new(Some(5));
    let field = Field::from_str(h, w, &s)?;

    // 求解
    if let Some(sol) = solver.solve(&field) {
        println!("> found answer\n{}", field.display_with_solution(&sol));
    } else {
        println!("> answer not found");
    }

    Ok(())
}

fn get_input() -> (usize, usize, String) {
    // 入力全部読む
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();

    // 空白区切りでイテレータにする
    let mut it = input.split_whitespace();

    let h: usize = it.next().unwrap().parse().unwrap();
    let w: usize = it.next().unwrap().parse().unwrap();

    // 残りは S_1, ..., S_h
    // split_whitespace すると改行は消えるので，ただつなげれば OK
    let mut s = String::new();
    for _ in 0..h {
        let row = it.next().unwrap();
        s += row;
        s += "\n";
    }

    (h, w, s)
}

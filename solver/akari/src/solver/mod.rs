//! ソルバの実装

mod naive;
pub use naive::Naive;
mod cfs;
pub use cfs::{CFS, Cell, TempFill};
mod cfs2;
pub use cfs2::CFSwithPB;

use crate::{
    field::{Field, Solution, State},
    utility::{ADJ, GridUtility},
};

const MISMATCH_AKARI: &str = "The number of lights does not match.";
const OVERLAP_AKARI: &str = "The light is already in place.";
const UNLIT_CELL: &str = "There are cells that are not lighted.";

/// ソルバを表すトレイト
pub trait Solver {
    fn solve(&self, field: &Field) -> Option<Solution>;
    /// 解 sol が条件を満たすか判定
    fn _check(field: &Field, sol: &Solution) -> Result<(), &'static str> {
        let (h, w) = (field.h, field.w);

        // あかりの配置が条件を満たすか判定
        for r in 0..field.h {
            for c in 0..field.w {
                let akari_count = (r, c).adj(h, w).filter(|&(r, c)| sol.field[r][c]).count();
                if match field.field[r][c] {
                    // あかりが置かれていれば x
                    State::Nil => sol.field[r][c],
                    State::Empty => false,
                    // あかりが置かれているか，周囲の数と一致しなければ x
                    State::Adj0 => sol.field[r][c] || akari_count != 0,
                    State::Adj1 => sol.field[r][c] || akari_count != 1,
                    State::Adj2 => sol.field[r][c] || akari_count != 2,
                    State::Adj3 => sol.field[r][c] || akari_count != 3,
                    State::Adj4 => sol.field[r][c] || akari_count != 4,
                } {
                    return Err(MISMATCH_AKARI);
                }
            }
        }

        // None: セルなし | ブロック
        // Some(false): あかりで照らされていない
        // Some(true): あかりで照らされている
        let mut fill: Vec<_> = field
            .field
            .iter()
            .map(|row| {
                row.iter()
                    .map(|c| (c == &State::Empty).then_some(false))
                    .collect::<Vec<_>>()
            })
            .collect();

        // あかりが重複していないか判定
        for r in 0..h {
            for c in 0..w {
                if !sol.field[r][c] {
                    continue;
                }
                // その場を塗る
                fill[r][c].replace(true);
                for dir in ADJ {
                    // 特定方向に塗れるだけ塗る
                    for (nr, nc) in (r, c).while_dir(h, w, dir) {
                        // あかりが置かれていたら失敗
                        if sol.field[nr][nc] {
                            return Err(OVERLAP_AKARI);
                        }
                        // ブロックに当たったら終了
                        if fill[nr][nc].is_none() {
                            break;
                        }
                        fill[nr][nc].replace(true);
                    }
                }
            }
        }

        // すべてのセルが照らされているか
        if fill.into_iter().flatten().all(|c| c.unwrap_or(true)) {
            Ok(())
        } else {
            Err(UNLIT_CELL)
        }
    }
}

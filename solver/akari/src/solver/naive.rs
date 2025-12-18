//! 愚直な全探索

use crate::{
    field::{Field, Solution},
    solver::Solver,
    utility::{ADJ, GridUtility},
};

/// バックトラックによる愚直な求解
pub struct Naive;

impl Naive {
    fn rec(
        field: &Field,
        pos: usize,
        sol: Solution,
        fill: Vec<Vec<Option<bool>>>,
        found: &mut Option<Solution>,
    ) {
        let (h, w) = (field.h, field.w);

        if found.is_some() {
            return;
        }

        // 最後のセルに来た場合，終了
        if pos == h * w {
            if Self::_check(field, &sol).is_ok() {
                *found = Some(sol);
            }
            return;
        }

        let (r, c) = (pos / w, pos % w);

        // あかりが設置できる場合
        if fill[r][c].is_some_and(|c| !c) {
            // あかりを設置
            let mut new_sol = sol.clone();
            new_sol.field[r][c] = true;

            // あかりが重複していないか判定
            let mut new_fill = fill.clone();
            // その場を塗る
            new_fill[r][c].replace(true);
            for dir in ADJ {
                // 特定方向に塗れるだけ塗る
                for (nr, nc) in (r, c).while_dir(h, w, dir) {
                    // あかりが置かれていたら失敗
                    if new_sol.field[nr][nc] {
                        return;
                    }
                    // ブロックに当たったら終了
                    if new_fill[nr][nc].is_none() {
                        break;
                    }
                    new_fill[nr][nc].replace(true);
                }
            }

            // 再帰呼び出し
            Self::rec(field, pos + 1, new_sol, new_fill, found);
        }

        // あかりを設置しない
        Self::rec(field, pos + 1, sol, fill, found);
    }
}

impl Solver for Naive {
    fn solve(&self, field: &Field) -> Option<Solution> {
        let h = field.field.len();
        let w = field.field.first().as_ref().map(|r| r.len()).unwrap_or(0);
        let sol = Solution {
            field: vec![vec![false; w]; h],
        };
        let fill: Vec<_> = field
            .field
            .iter()
            .map(|row| {
                row.iter()
                    .map(|c| c.is_empty().then_some(false))
                    .collect::<Vec<_>>()
            })
            .collect();
        let mut found = None;

        Self::rec(field, 0, sol, fill, &mut found);

        found
    }
}

#[cfg(test)]
mod test_naive {
    use crate::{
        field::{Field, Solution, State},
        solver::{MISMATCH_AKARI, Naive, OVERLAP_AKARI, Solver, UNLIT_CELL},
    };

    #[test]
    fn test_check() {
        // 成功
        let field = Field::from_str(3, 3, "2.1 ... ..0").unwrap();
        let sol = Solution {
            field: vec![
                vec![false, true, false],
                vec![true, false, false],
                vec![false, false, false],
            ],
        };
        assert_eq!(Naive::_check(&field, &sol), Ok(()));

        // あかりの数の不一致
        let field = Field::from_str(3, 3, "2.2 ... ..0").unwrap();
        let sol = Solution {
            field: vec![
                vec![false, true, false],
                vec![true, false, false],
                vec![false, false, false],
            ],
        };
        assert_eq!(Naive::_check(&field, &sol), Err(MISMATCH_AKARI));

        // あかりの重複
        let field = Field {
            h: 3,
            w: 3,
            field: vec![
                vec![State::Adj2, State::Empty, State::Adj2],
                vec![State::Empty, State::Empty, State::Empty],
                vec![State::Empty, State::Empty, State::Empty],
            ],
        };
        let sol = Solution {
            field: vec![
                vec![false, true, false],
                vec![true, false, true],
                vec![false, false, false],
            ],
        };
        assert_eq!(Naive::_check(&field, &sol), Err(OVERLAP_AKARI));

        // 照らされていないマスが存在
        let field = Field::from_str(3, 3, "2.1 ... ...").unwrap();
        let sol = Solution {
            field: vec![
                vec![false, true, false],
                vec![true, false, false],
                vec![false, false, false],
            ],
        };
        assert_eq!(Naive::_check(&field, &sol), Err(UNLIT_CELL));
    }

    #[test]
    fn test_solve() {
        let field = Field::from_str(1, 3, ".2.").unwrap();
        let answer = Solution {
            field: vec![vec![true, false, true]],
        };
        assert_eq!(Naive.solve(&field), Some(answer));

        let field = Field::from_str(3, 3, "2.1 ... ..0").unwrap();
        let answer = Solution {
            field: vec![
                vec![false, true, false],
                vec![true, false, false],
                vec![false, false, false],
            ],
        };
        assert_eq!(Naive.solve(&field), Some(answer));
    }
}

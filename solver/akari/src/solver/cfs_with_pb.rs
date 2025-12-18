//! 制約を優先的に探索（プログレスバー付き）

use itertools::Itertools;

use crate::{
    Field, ProgressBar, Solution, Solver, State,
    solver::{Cell, OVERLAP_AKARI, TempFill},
    utility::{ADJ, GridUtility},
};

/// constraint first search with progress bar
///
/// 影響範囲が狭く強い制約を持つセル（数字セル）が「最も情報量の大きい変数」として優先される変数選択ヒューリスティック．
pub struct CFSwithPB;

impl CFSwithPB {
    fn rec(
        field: &Field,
        cell_pos: usize,
        sol: Solution,
        fill: TempFill,
        found: &mut Option<Solution>,
    ) {
        // println!("{}", field.display_with_solution_and_state(&sol, &fill));

        let (h, w) = (field.h, field.w);

        if found.is_some() {
            return;
        }

        // 最後のセルに来た場合，終了
        if cell_pos == h * w {
            if Self::_check(field, &sol).is_ok() {
                *found = Some(sol);
            }
            return;
        }

        // 充足不可能なセルがあればスキップ
        if Self::has_unfeasible_cell(field, &fill) {
            return;
        }

        // 制約が残っていない場合，愚直に埋めていく
        let (r, c) = (cell_pos / w, cell_pos % w);

        // あかりが設置できる場合
        if fill[r][c].can_put_akari() {
            // あかりを設置
            if let Ok((sol, fill)) = Self::put_akari(field, r, c, sol.clone(), fill.clone()) {
                Self::rec(field, cell_pos + 1, sol, fill, found);
            }
        }

        // あかりを設置しない
        let mut fill = fill;
        fill[r][c].disable();
        Self::rec(field, cell_pos + 1, sol, fill, found);
    }

    /// 制約を充足する配置を列挙する．
    fn enum_constraints(
        field: &Field,
        constraints: &[(usize, usize)],
        cons_pos: usize,
        sol: Solution,
        fill: TempFill,
    ) -> Vec<(Solution, TempFill)> {
        if cons_pos == constraints.len() {
            return vec![(sol, fill)];
        }

        // 充足不可能なセルがあればスキップ
        if Self::has_unfeasible_cell(field, &fill) {
            return vec![];
        }

        let mut results = Vec::new();
        let (r, c) = constraints[cons_pos];
        let (h, w) = (field.h, field.w);

        match field.field[r][c] {
            State::Adj0 => {
                if let Some((sol, fill)) = Some((sol, fill))
                    .map(|(sol, mut fill)| {
                        if let Some((ar, ac)) = (r, c).right(h, w) {
                            fill[ar][ac].disable();
                        }
                        (sol, fill)
                    })
                    .map(|(sol, mut fill)| {
                        if let Some((ar, ac)) = (r, c).up(h, w) {
                            fill[ar][ac].disable();
                        }
                        (sol, fill)
                    })
                    .map(|(sol, mut fill)| {
                        if let Some((ar, ac)) = (r, c).left(h, w) {
                            fill[ar][ac].disable();
                        }
                        (sol, fill)
                    })
                    .map(|(sol, mut fill)| {
                        if let Some((ar, ac)) = (r, c).down(h, w) {
                            fill[ar][ac].disable();
                        }
                        (sol, fill)
                    })
                {
                    results.extend(Self::enum_constraints(
                        field,
                        constraints,
                        cons_pos + 1,
                        sol,
                        fill,
                    ));
                }
            }
            State::Adj1 => {
                for d in ADJ {
                    let nd: Vec<_> = ADJ.into_iter().filter(|&x| x != d).collect();

                    if let Some((sol, fill)) = (r, c)
                        .dir(h, w, d)
                        .and_then(|(ar, ac)| {
                            Self::put_akari(field, ar, ac, sol.clone(), fill.clone()).ok()
                        })
                        .map(|(sol, mut fill)| {
                            if let Some((ar, ac)) = (r, c).dir(h, w, nd[0]) {
                                fill[ar][ac].disable();
                            }
                            (sol, fill)
                        })
                        .map(|(sol, mut fill)| {
                            if let Some((ar, ac)) = (r, c).dir(h, w, nd[1]) {
                                fill[ar][ac].disable();
                            }
                            (sol, fill)
                        })
                        .map(|(sol, mut fill)| {
                            if let Some((ar, ac)) = (r, c).dir(h, w, nd[2]) {
                                fill[ar][ac].disable();
                            }
                            (sol, fill)
                        })
                    {
                        results.extend(Self::enum_constraints(
                            field,
                            constraints,
                            cons_pos + 1,
                            sol,
                            fill,
                        ));
                    }
                }
            }
            State::Adj2 => {
                for d in ADJ.iter().combinations(2) {
                    let nd: Vec<_> = ADJ.into_iter().filter(|x| !d.contains(&x)).collect();

                    if let Some((sol, fill)) = Some((sol.clone(), fill.clone()))
                        .and_then(|(sol, fill)| {
                            (r, c)
                                .dir(h, w, *d[0])
                                .and_then(|(ar, ac)| Self::put_akari(field, ar, ac, sol, fill).ok())
                        })
                        .and_then(|(sol, fill)| {
                            (r, c)
                                .dir(h, w, *d[1])
                                .and_then(|(ar, ac)| Self::put_akari(field, ar, ac, sol, fill).ok())
                        })
                        .map(|(sol, mut fill)| {
                            if let Some((ar, ac)) = (r, c).dir(h, w, nd[0]) {
                                fill[ar][ac].disable();
                            }
                            (sol, fill)
                        })
                        .map(|(sol, mut fill)| {
                            if let Some((ar, ac)) = (r, c).dir(h, w, nd[1]) {
                                fill[ar][ac].disable();
                            }
                            (sol, fill)
                        })
                    {
                        results.extend(Self::enum_constraints(
                            field,
                            constraints,
                            cons_pos + 1,
                            sol,
                            fill,
                        ));
                    }
                }
            }
            State::Adj3 => {
                for d in ADJ.iter().combinations(3) {
                    let nd: Vec<_> = ADJ.into_iter().filter(|x| !d.contains(&x)).collect();

                    if let Some((sol, fill)) = Some((sol.clone(), fill.clone()))
                        .and_then(|(sol, fill)| {
                            (r, c)
                                .dir(h, w, *d[0])
                                .and_then(|(ar, ac)| Self::put_akari(field, ar, ac, sol, fill).ok())
                        })
                        .and_then(|(sol, fill)| {
                            (r, c)
                                .dir(h, w, *d[1])
                                .and_then(|(ar, ac)| Self::put_akari(field, ar, ac, sol, fill).ok())
                        })
                        .and_then(|(sol, fill)| {
                            (r, c)
                                .dir(h, w, *d[2])
                                .and_then(|(ar, ac)| Self::put_akari(field, ar, ac, sol, fill).ok())
                        })
                        .map(|(sol, mut fill)| {
                            if let Some((ar, ac)) = (r, c).dir(h, w, nd[0]) {
                                fill[ar][ac].disable();
                            }
                            (sol, fill)
                        })
                    {
                        results.extend(Self::enum_constraints(
                            field,
                            constraints,
                            cons_pos + 1,
                            sol,
                            fill,
                        ));
                    }
                }
            }
            State::Adj4 => {
                if let Some((sol, fill)) = Some((sol, fill))
                    .and_then(|(sol, fill)| {
                        (r, c)
                            .right(h, w)
                            .and_then(|(ar, ac)| Self::put_akari(field, ar, ac, sol, fill).ok())
                    })
                    .and_then(|(sol, fill)| {
                        (r, c)
                            .up(h, w)
                            .and_then(|(ar, ac)| Self::put_akari(field, ar, ac, sol, fill).ok())
                    })
                    .and_then(|(sol, fill)| {
                        (r, c)
                            .left(h, w)
                            .and_then(|(ar, ac)| Self::put_akari(field, ar, ac, sol, fill).ok())
                    })
                    .and_then(|(sol, fill)| {
                        (r, c)
                            .down(h, w)
                            .and_then(|(ar, ac)| Self::put_akari(field, ar, ac, sol, fill).ok())
                    })
                {
                    results.extend(Self::enum_constraints(
                        field,
                        constraints,
                        cons_pos + 1,
                        sol,
                        fill,
                    ));
                }
            }
            _ => unreachable!(),
        }
        results
    }

    /// field の (r, c) にあかりを配置する
    fn put_akari(
        field: &Field,
        r: usize,
        c: usize,
        mut sol: Solution,
        mut fill: TempFill,
    ) -> Result<(Solution, TempFill), &'static str> {
        // その場を塗れるか確認
        if let Cell::Fillable = fill[r][c] {
            fill[r][c] = Cell::Unfillable(true);
        } else if sol.field[r][c] {
            return Ok((sol, fill));
        } else {
            return Err("Given cell is not fillable.");
        }
        // あかりを設置
        sol.field[r][c] = true;

        // 重複確認
        for dir in ADJ {
            // 特定方向に塗れるだけ塗る
            for (nr, nc) in (r, c).while_dir(field.h, field.w, dir) {
                // あかりが置かれていたら失敗
                if sol.field[nr][nc] {
                    return Err(OVERLAP_AKARI);
                }
                // ブロックに当たったら終了
                match fill[nr][nc] {
                    Cell::Nil => break,
                    _ => fill[nr][nc] = Cell::Unfillable(true),
                }
            }
        }

        Ok((sol, fill))
    }

    /// 充足不可能なセルが存在するか判定する
    ///
    /// TODO: 差分更新の実装
    ///
    /// - 時間計算量: O(h*w*(h+w))
    fn has_unfeasible_cell(field: &Field, fill: &TempFill) -> bool {
        for r in 0..field.h {
            for c in 0..field.w {
                if !matches!(fill[r][c], Cell::Unfillable(false)) {
                    continue;
                }
                // (r,c) を照らせるセルが存在するか
                let mut is_ok = false;
                'outer: for dir in ADJ {
                    // 特定方向に塗れるだけ塗る
                    for (nr, nc) in (r, c).while_dir(field.h, field.w, dir) {
                        if fill[nr][nc].can_put_akari() {
                            is_ok = true;
                            break 'outer;
                        }
                    }
                }
                if !is_ok {
                    return true;
                }
            }
        }
        false
    }
}

impl Solver for CFSwithPB {
    fn solve(&self, field: &Field) -> Option<Solution> {
        let h = field.field.len();
        let w = field.field.first().as_ref().map(|r| r.len()).unwrap_or(0);
        let sol = Solution {
            field: vec![vec![false; w]; h],
        };
        let constraints: Vec<_> = (0..h * w)
            .map(|i| (i / w, i % w))
            .filter(|&(r, c)| field.field[r][c].is_adj().is_some())
            .collect();
        let fill: Vec<_> = field
            .field
            .iter()
            .map(|row| {
                row.iter()
                    .map(|c| {
                        if c.is_empty() {
                            Cell::Fillable
                        } else {
                            Cell::Nil
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        let mut found = None;
        let enumerated = Self::enum_constraints(field, &constraints, 0, sol, fill);
        let mut progress = ProgressBar::new(enumerated.len());
        for (sol, fill) in enumerated {
            Self::rec(field, 0, sol, fill, &mut found);
            progress.tick();
            if found.is_some() {
                break;
            }
        }
        progress.finish();

        found
    }
}

#[cfg(test)]
mod test_cfs2 {
    use crate::{
        field::{Field, Solution},
        solver::{Solver, cfs_with_pb::CFSwithPB},
    };

    #[test]
    fn test_solve() {
        let field = Field::from_str(1, 3, ".2.").unwrap();
        let answer = Solution {
            field: vec![vec![true, false, true]],
        };
        assert_eq!(CFSwithPB.solve(&field), Some(answer));

        let field = Field::from_str(3, 3, "2.1 ... ..0").unwrap();
        let answer = Solution {
            field: vec![
                vec![false, true, false],
                vec![true, false, false],
                vec![false, false, false],
            ],
        };
        assert_eq!(CFSwithPB.solve(&field), Some(answer));
    }
}

//! alg-fast.md に基づく高速ソルバ

use std::collections::VecDeque;

use crate::{utility::GridUtility, Field, Solution, Solver, State};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CellState {
    Unknown,
    Light,
    Blocked,
}

#[derive(Clone, Debug)]
struct Segment {
    cells: Vec<usize>,
    light: Option<usize>,
    free_cnt: i32, // Blocked でないセル数
}

#[derive(Clone, Debug)]
struct NumCell {
    value: i32,
    adj: Vec<usize>,
    on: i32,
    unk: i32,
}

#[derive(Clone, Debug)]
enum Action {
    CellState { idx: usize, prev: CellState },
    RowLight { seg: usize, prev: Option<usize> },
    ColLight { seg: usize, prev: Option<usize> },
    RowFree { seg: usize, prev: i32 },
    ColFree { seg: usize, prev: i32 },
    LitCount { idx: usize, prev: i32 },
    NumOn { idx: usize, prev: i32 },
    NumUnk { idx: usize, prev: i32 },
}

#[derive(Clone, Debug)]
struct Core {
    n_empty: usize,
    empty_pos: Vec<(usize, usize)>,
    row_seg_id: Vec<usize>,
    col_seg_id: Vec<usize>,
    row_segs: Vec<Segment>,
    col_segs: Vec<Segment>,
    num_cells: Vec<NumCell>,
    num_adj_of_empty: Vec<Vec<usize>>,
    lit_list: Vec<Vec<usize>>,
    lit_count: Vec<i32>,
    cell_state: Vec<CellState>,
    trail: Vec<Action>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Fast;

impl Fast {
    pub fn new() -> Self {
        Self
    }
}

impl Solver for Fast {
    fn solve(&self, field: &Field) -> Option<Solution> {
        let mut core = Core::new(field);
        if core.dfs() {
            Some(core.to_solution(field))
        } else {
            None
        }
    }
}

impl Core {
    fn new(field: &Field) -> Self {
        let h = field.h;
        let w = field.w;

        let mut empty_pos = Vec::new();
        let mut empty_id = vec![vec![None; w]; h];
        for r in 0..h {
            for c in 0..w {
                if field.field[r][c] == State::Empty {
                    let id = empty_pos.len();
                    empty_pos.push((r, c));
                    empty_id[r][c] = Some(id);
                }
            }
        }

        let n_empty = empty_pos.len();
        let mut row_seg_id = vec![0; n_empty];
        let mut col_seg_id = vec![0; n_empty];

        let mut row_segs = Vec::new();
        for r in 0..h {
            let mut c = 0;
            while c < w {
                if field.field[r][c] == State::Empty {
                    let mut cells = Vec::new();
                    while c < w && field.field[r][c] == State::Empty {
                        let id = empty_id[r][c].unwrap();
                        row_seg_id[id] = row_segs.len();
                        cells.push(id);
                        c += 1;
                    }
                    row_segs.push(Segment {
                        light: None,
                        free_cnt: cells.len() as i32,
                        cells,
                    });
                } else {
                    c += 1;
                }
            }
        }

        let mut col_segs = Vec::new();
        for c in 0..w {
            let mut r = 0;
            while r < h {
                if field.field[r][c] == State::Empty {
                    let mut cells = Vec::new();
                    while r < h && field.field[r][c] == State::Empty {
                        let id = empty_id[r][c].unwrap();
                        col_seg_id[id] = col_segs.len();
                        cells.push(id);
                        r += 1;
                    }
                    col_segs.push(Segment {
                        light: None,
                        free_cnt: cells.len() as i32,
                        cells,
                    });
                } else {
                    r += 1;
                }
            }
        }

        let mut num_cells = Vec::new();
        let mut num_adj_of_empty = vec![Vec::new(); n_empty];
        for r in 0..h {
            for c in 0..w {
                if let Some(value) = field.field[r][c].is_adj() {
                    let mut adj = Vec::new();
                    for (nr, nc) in (r, c).adj(h, w) {
                        if field.field[nr][nc] == State::Empty {
                            let id = empty_id[nr][nc].unwrap();
                            adj.push(id);
                            num_adj_of_empty[id].push(num_cells.len());
                        }
                    }
                    let unk = adj.len() as i32;
                    num_cells.push(NumCell {
                        value: value as i32,
                        adj,
                        on: 0,
                        unk,
                    });
                }
            }
        }

        let mut lit_list = Vec::with_capacity(n_empty);
        for id in 0..n_empty {
            let mut list = row_segs[row_seg_id[id]].cells.clone();
            for &x in &col_segs[col_seg_id[id]].cells {
                if !list.contains(&x) {
                    list.push(x);
                }
            }
            lit_list.push(list);
        }

        Core {
            n_empty,
            empty_pos,
            row_seg_id,
            col_seg_id,
            row_segs,
            col_segs,
            num_cells,
            num_adj_of_empty,
            lit_list,
            lit_count: vec![0; n_empty],
            cell_state: vec![CellState::Unknown; n_empty],
            trail: Vec::new(),
        }
    }

    fn to_solution(&self, field: &Field) -> Solution {
        let mut grid = vec![vec![false; field.w]; field.h];
        for (id, &(r, c)) in self.empty_pos.iter().enumerate() {
            if self.cell_state[id] == CellState::Light {
                grid[r][c] = true;
            }
        }
        Solution { field: grid }
    }

    fn checkpoint(&self) -> usize {
        self.trail.len()
    }

    fn undo(&mut self, cp: usize) {
        while self.trail.len() > cp {
            match self.trail.pop().unwrap() {
                Action::CellState { idx, prev } => self.cell_state[idx] = prev,
                Action::RowLight { seg, prev } => self.row_segs[seg].light = prev,
                Action::ColLight { seg, prev } => self.col_segs[seg].light = prev,
                Action::RowFree { seg, prev } => self.row_segs[seg].free_cnt = prev,
                Action::ColFree { seg, prev } => self.col_segs[seg].free_cnt = prev,
                Action::LitCount { idx, prev } => self.lit_count[idx] = prev,
                Action::NumOn { idx, prev } => self.num_cells[idx].on = prev,
                Action::NumUnk { idx, prev } => self.num_cells[idx].unk = prev,
            }
        }
    }

    fn set_blocked(&mut self, cell: usize, q_num: &mut VecDeque<usize>) -> bool {
        match self.cell_state[cell] {
            CellState::Blocked => return true,
            CellState::Light => return false,
            CellState::Unknown => {}
        }
        self.trail.push(Action::CellState {
            idx: cell,
            prev: CellState::Unknown,
        });
        self.cell_state[cell] = CellState::Blocked;

        let rseg = self.row_seg_id[cell];
        let prev_row = self.row_segs[rseg].free_cnt;
        self.trail.push(Action::RowFree {
            seg: rseg,
            prev: prev_row,
        });
        self.row_segs[rseg].free_cnt -= 1;

        let cseg = self.col_seg_id[cell];
        let prev_col = self.col_segs[cseg].free_cnt;
        self.trail.push(Action::ColFree {
            seg: cseg,
            prev: prev_col,
        });
        self.col_segs[cseg].free_cnt -= 1;

        for &idx in &self.num_adj_of_empty[cell] {
            let prev_unk = self.num_cells[idx].unk;
            self.trail.push(Action::NumUnk {
                idx,
                prev: prev_unk,
            });
            self.num_cells[idx].unk -= 1;
            q_num.push_back(idx);
        }

        true
    }

    fn set_light(&mut self, cell: usize, q_num: &mut VecDeque<usize>) -> bool {
        match self.cell_state[cell] {
            CellState::Light => return true,
            CellState::Blocked => return false,
            CellState::Unknown => {}
        }
        self.trail.push(Action::CellState {
            idx: cell,
            prev: CellState::Unknown,
        });
        self.cell_state[cell] = CellState::Light;

        for &lit in &self.lit_list[cell] {
            let prev = self.lit_count[lit];
            self.trail.push(Action::LitCount { idx: lit, prev });
            self.lit_count[lit] = prev + 1;
        }

        for &idx in &self.num_adj_of_empty[cell] {
            let prev_on = self.num_cells[idx].on;
            self.trail.push(Action::NumOn {
                idx,
                prev: prev_on,
            });
            self.num_cells[idx].on = prev_on + 1;

            let prev_unk = self.num_cells[idx].unk;
            self.trail.push(Action::NumUnk {
                idx,
                prev: prev_unk,
            });
            self.num_cells[idx].unk = prev_unk - 1;
            q_num.push_back(idx);
        }

        let rseg = self.row_seg_id[cell];
        match self.row_segs[rseg].light {
            Some(prev) if prev != cell => return false,
            Some(_) => {}
            None => {
                self.trail.push(Action::RowLight { seg: rseg, prev: None });
                self.row_segs[rseg].light = Some(cell);
                let cells = self.row_segs[rseg].cells.clone();
                for other in cells {
                    if other != cell && !self.set_blocked(other, q_num) {
                        return false;
                    }
                }
            }
        }

        let cseg = self.col_seg_id[cell];
        match self.col_segs[cseg].light {
            Some(prev) if prev != cell => return false,
            Some(_) => {}
            None => {
                self.trail.push(Action::ColLight { seg: cseg, prev: None });
                self.col_segs[cseg].light = Some(cell);
                let cells = self.col_segs[cseg].cells.clone();
                for other in cells {
                    if other != cell && !self.set_blocked(other, q_num) {
                        return false;
                    }
                }
            }
        }

        true
    }

    fn propagate(&mut self) -> bool {
        let mut q_num: VecDeque<usize> = (0..self.num_cells.len()).collect();

        loop {
            let mut changed = false;

            while let Some(idx) = q_num.pop_front() {
                let (on, unk, value) = {
                    let n = &self.num_cells[idx];
                    (n.on, n.unk, n.value)
                };

                if on > value || on + unk < value {
                    return false;
                }

                if on == value {
                    let adj = self.num_cells[idx].adj.clone();
                    for cell in adj {
                        if self.cell_state[cell] == CellState::Unknown {
                            if !self.set_blocked(cell, &mut q_num) {
                                return false;
                            }
                            changed = true;
                        }
                    }
                } else if on + unk == value {
                    let adj = self.num_cells[idx].adj.clone();
                    for cell in adj {
                        if self.cell_state[cell] == CellState::Unknown {
                            if !self.set_light(cell, &mut q_num) {
                                return false;
                            }
                            changed = true;
                        }
                    }
                }
            }

            for cell in 0..self.n_empty {
                if self.lit_count[cell] > 0 {
                    continue;
                }
                let rseg = self.row_seg_id[cell];
                let cseg = self.col_seg_id[cell];

                if self.row_segs[rseg].light.is_some() || self.col_segs[cseg].light.is_some() {
                    continue;
                }

                let row_free = self.row_segs[rseg].free_cnt;
                let col_free = self.col_segs[cseg].free_cnt;
                let self_free = if self.cell_state[cell] != CellState::Blocked {
                    1
                } else {
                    0
                };
                let cand = row_free + col_free - self_free;

                if cand == 0 {
                    return false;
                }
                if cand == 1 {
                    if let Some(pos) = self.find_single_candidate(rseg, cseg) {
                        if !self.set_light(pos, &mut q_num) {
                            return false;
                        }
                        changed = true;
                    } else {
                        return false;
                    }
                }
            }

            if !changed && q_num.is_empty() {
                break;
            }
        }

        true
    }

    fn find_single_candidate(&self, rseg: usize, cseg: usize) -> Option<usize> {
        let mut only = None;
        for &cell in &self.row_segs[rseg].cells {
            if self.cell_state[cell] != CellState::Blocked {
                if let Some(prev) = only {
                    if prev != cell {
                        return None;
                    }
                } else {
                    only = Some(cell);
                }
            }
        }
        for &cell in &self.col_segs[cseg].cells {
            if self.cell_state[cell] != CellState::Blocked {
                if let Some(prev) = only {
                    if prev != cell {
                        return None;
                    }
                } else {
                    only = Some(cell);
                }
            }
        }
        only
    }

    fn is_solved(&self) -> bool {
        if self.lit_count.iter().any(|&v| v == 0) {
            return false;
        }
        self.num_cells.iter().all(|n| n.on == n.value)
    }

    fn choose_branch_cell(&self) -> Option<Vec<usize>> {
        let mut best: Option<(i32, Vec<usize>)> = None;

        for cell in 0..self.n_empty {
            if self.lit_count[cell] > 0 {
                continue;
            }
            let rseg = self.row_seg_id[cell];
            let cseg = self.col_seg_id[cell];
            if self.row_segs[rseg].light.is_some() || self.col_segs[cseg].light.is_some() {
                continue;
            }

            let row_free = self.row_segs[rseg].free_cnt;
            let col_free = self.col_segs[cseg].free_cnt;
            let self_free = if self.cell_state[cell] != CellState::Blocked {
                1
            } else {
                0
            };
            let cand_count = row_free + col_free - self_free;
            if cand_count <= 1 {
                continue;
            }

            let mut candidates = Vec::new();
            for &x in &self.row_segs[rseg].cells {
                if self.cell_state[x] != CellState::Blocked {
                    candidates.push(x);
                }
            }
            for &x in &self.col_segs[cseg].cells {
                if self.cell_state[x] != CellState::Blocked && !candidates.contains(&x) {
                    candidates.push(x);
                }
            }

            match &best {
                None => best = Some((cand_count, candidates)),
                Some((cnt, _)) if cand_count < *cnt => best = Some((cand_count, candidates)),
                _ => {}
            }
        }

        best.map(|(_, cand)| cand)
    }

    fn dfs(&mut self) -> bool {
        if !self.propagate() {
            return false;
        }
        if self.is_solved() {
            return true;
        }

        let Some(candidates) = self.choose_branch_cell() else {
            return false;
        };

        for pos in candidates {
            let cp = self.checkpoint();
            let mut q = VecDeque::new();
            if self.set_light(pos, &mut q) && self.dfs() {
                return true;
            }
            self.undo(cp);
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::Fast;
    use crate::{Field, Solver};

    #[test]
    fn solve_single_cell() {
        let field = Field::from_str(1, 1, ".\n").unwrap();
        let solver = Fast::new();
        let sol = solver.solve(&field).expect("solution");
        assert!(sol.field[0][0]);
    }
}

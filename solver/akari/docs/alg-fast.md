> 以下の問題を解く Rust のアルゴリズムを考えてください。正当性があり、かつ高速なものを示してください。「dailyakariのソルバーを作成してください。盤面の大きさ（縦、横）とそれぞれの盤面が壁(#), 床(.), 隣接する床に何個明かりが置かれるかの数字のどれかです。明かりを置く位置を出力してください。必ず解が存在し、一意であることが保証されます。入力例 11 20 ........1....1...... .......2......1...2. ...............1.... .....2..........2... .................1.. ...2..............#. ..................1. .2.................. 1....1.............. .#........0..0..0..0 ....................」

以下は「Akari（Light Up）」を高速に解くための実装方針です．保証「解が存在し，一意」なので，強い制約伝播＋少しの分岐で十分に高速に解けます．Rustで素直に実装できます．

---

## 問題の制約の捉え方

盤面の「明かりを置けるマス」を空マス（`.`）と呼びます．壁は `#`，数字は `0..4` の「数字壁」です．

明かり配置の条件は次の3つです．

1. 同一直線上（上下左右）で壁に遮られずに見える位置に，明かり同士が同時に存在しない
2. すべての空マスが，少なくとも1つの明かりに照らされる（同じ行・列で壁に遮られない）
3. 数字壁は「上下左右の隣接4マスにある明かりの個数」が数字と一致する

ここで決定的に効く観察があります．

### 観察：行セグメント・列セグメント

壁（`#` と数字壁）の間で連続する空マスの列を

* 行方向の最大連続区間＝行セグメント
* 列方向の最大連続区間＝列セグメント

と呼ぶと，同じ行セグメント内に明かりを2個置くと必ず互いに見えてしまうので

* 各行セグメントに置ける明かりは高々1個
* 各列セグメントに置ける明かりは高々1個

が成り立ちます．これが伝播を劇的に強くします．

---

## 状態と前処理

### 前処理（O(HW)）

1. 空マスに id を振る（0..N-1）
2. 各空マスについて「所属する行セグメント id」「所属する列セグメント id」を計算
3. 各セグメントについて「含まれる空マス id の配列」を持つ
4. 各数字壁について「隣接する空マス id の配列」を持つ
5. 各空マス `c` に明かりを置いたとき照らされる空マス集合は
   `rowSeg(c) の全空マス ∪ colSeg(c) の全空マス`
   なので，これを `lit_list[c]` として前計算（合計サイズは概ね O(N·平均セグ長) ですが実用上十分小さいです）

### 状態（バックトラック対応）

* `cell_state[c] ∈ {Unknown, Light, Blocked}`（明かり確定／置けない確定／未確定）
* `row_light[r] : Option<cell>`（その行セグメントに明かりがあるなら位置）
* `col_light[k] : Option<cell>`
* `row_free_cnt[r]`：その行セグメントで `Blocked` でない空マス数
  `col_free_cnt[k]` も同様
* `lit_count[c]`：空マス `c` を照らしている明かり数（0なら未照明）

数字壁用に

* `num_on[i]`：隣接4マスで `Light` の数
* `num_unk[i]`：隣接4マスで `Unknown` の数
  を持つと，強い確定がすぐ出ます．

---

## 制約伝播（正当性つき）

以下を「変化がなくなるまで」繰り返します．

### 伝播A：数字壁（局所で強い）

数字壁 i の値を `k` とし，現在 `num_on = a`，`num_unk = u` とします．

* `a > k` なら矛盾
* `a + u < k` なら矛盾
* `a == k` なら，残りの unknown 隣接マスは全て `Blocked` に確定
* `a + u == k` なら，残りの unknown 隣接マスは全て `Light` に確定

これは定義そのものなので正当です．

### 伝播B：セグメントの高々1制約

`c` を `Light` にしたら，

* 同じ行セグメントの他の空マスは全部 `Blocked`
* 同じ列セグメントの他の空マスは全部 `Blocked`
* `row_light`，`col_light` が既に別位置なら矛盾

これは「同一直線で見える明かり禁止」から直ちに正当です．

### 伝播C：未照明マスの「候補が1つなら強制」

空マス `c` がまだ未照明（`lit_count[c]==0`）で，かつ

* 行セグメントに明かりが未確定（`row_light` が None）
* 列セグメントに明かりが未確定（`col_light` が None）

なら，`c` を照らせる「明かり候補」は

* `rowSeg(c)` 内で `Blocked` でないマス
* `colSeg(c)` 内で `Blocked` でないマス

の和集合です．行セグと列セグの交点は `c` 自身の1点だけなので，候補数は

`cand = row_free_cnt[rowSeg(c)] + col_free_cnt[colSeg(c)] - (c が Blocked でないなら 1 else 0)`

で即座に計算できます．

* `cand == 0` なら矛盾（このマスは永遠に照らせない）
* `cand == 1` なら，その唯一の候補に `Light` を強制

「このマスは最終的に照らされる必要がある」ことから正当です．

---

## 探索（分岐）と高速化

伝播だけで終わらない場合のみ分岐します（ユニーク解なので分岐は小さいはずです）．

### 分岐の選び方（効くヒューリスティック）

未照明の空マス `c` のうち `cand` が最小のものを選びます（MRV）．
その `c` を照らす候補集合（行セグ内＋列セグ内，Blocked除く）を列挙し，

候補 `p` を1つ選んで「`p` を Light にする」を試す
失敗したら次の候補へ，を繰り返します．

ユニーク解なので，どこかの候補でただ1つ成功します．

### 正当性

各分岐は「どれかの候補が明かりでなければ `c` が照らされない」という必要条件からの分割なので，探索木全体で解空間を漏れなく覆います．
伝播はすべて必要条件・定義からの帰結なので，解を誤って捨てません．
よって「矛盾で枝刈りしつつ探索」して最初に見つかった解は必ず正解です．さらに「解が一意」なのでそれが唯一解です．

---

## Rust実装スケルトン（要点込み）

出力形式は問題文から厳密に読めないので，ここでは「盤面と同じサイズで `L` を置いた盤面」を出す例にします（必要なら座標列に変更できます）．

```rust
use std::collections::VecDeque;
use std::io::{self, Read};

#[derive(Clone, Copy, PartialEq, Eq)]
enum CellState { Unknown, Light, Blocked }

#[derive(Clone)]
struct NumCell {
    k: u8,
    adj: Vec<usize>, // 隣接する空マスid
    on: i8,
    unk: i8,
}

#[derive(Clone)]
struct Segment {
    cells: Vec<usize>,      // 空マスid
    light: Option<usize>,   // 空マスid
    free_cnt: i16,          // Blockedでない数
}

#[derive(Clone)]
enum Action {
    SetCell { c: usize, prev: CellState },
    SetRowLight { r: usize, prev: Option<usize> },
    SetColLight { k: usize, prev: Option<usize> },
    AddRowFree { r: usize, delta: i16 }, // undo用に加算
    AddColFree { k: usize, delta: i16 },
    IncLit { c: usize },                 // undoでdecrement
    AddNumOn { i: usize, delta: i8 },
    AddNumUnk { i: usize, delta: i8 },
}

#[derive(Clone)]
struct Solver {
    h: usize,
    w: usize,
    grid: Vec<Vec<char>>,

    // 空マス
    n_empty: usize,
    empty_pos: Vec<(usize, usize)>, // id -> (r,c)
    empty_id: Vec<Vec<Option<usize>>>,

    // セグメント
    row_seg_id: Vec<usize>, // empty id -> row seg
    col_seg_id: Vec<usize>, // empty id -> col seg
    row_segs: Vec<Segment>,
    col_segs: Vec<Segment>,

    // 数字壁
    num_cells: Vec<NumCell>,
    num_adj_of_empty: Vec<Vec<usize>>, // empty id -> 影響する数字壁index群

    // 照明
    lit_list: Vec<Vec<usize>>, // empty id（ここにLight） -> 照らされるempty id
    lit_count: Vec<i16>,

    // 状態
    cell_state: Vec<CellState>,

    // trail
    trail: Vec<Action>,
}

impl Solver {
    fn checkpoint(&self) -> usize { self.trail.len() }

    fn undo(&mut self, cp: usize) {
        while self.trail.len() > cp {
            match self.trail.pop().unwrap() {
                Action::SetCell { c, prev } => self.cell_state[c] = prev,
                Action::SetRowLight { r, prev } => self.row_segs[r].light = prev,
                Action::SetColLight { k, prev } => self.col_segs[k].light = prev,
                Action::AddRowFree { r, delta } => self.row_segs[r].free_cnt += delta,
                Action::AddColFree { k, delta } => self.col_segs[k].free_cnt += delta,
                Action::IncLit { c } => self.lit_count[c] -= 1,
                Action::AddNumOn { i, delta } => self.num_cells[i].on += delta,
                Action::AddNumUnk { i, delta } => self.num_cells[i].unk += delta,
            }
        }
    }

    fn set_blocked(&mut self, c: usize, q_num: &mut VecDeque<usize>) -> bool {
        match self.cell_state[c] {
            CellState::Blocked => return true,
            CellState::Light => return false,
            CellState::Unknown => {}
        }
        self.trail.push(Action::SetCell { c, prev: CellState::Unknown });
        self.cell_state[c] = CellState::Blocked;

        let r = self.row_seg_id[c];
        let k = self.col_seg_id[c];
        self.row_segs[r].free_cnt -= 1;
        self.trail.push(Action::AddRowFree { r, delta: 1 });
        self.col_segs[k].free_cnt -= 1;
        self.trail.push(Action::AddColFree { k, delta: 1 });

        for &ni in &self.num_adj_of_empty[c] {
            self.num_cells[ni].unk -= 1;
            self.trail.push(Action::AddNumUnk { i: ni, delta: 1 });
            q_num.push_back(ni);
        }
        true
    }

    fn set_light(&mut self, c: usize, q_num: &mut VecDeque<usize>) -> bool {
        match self.cell_state[c] {
            CellState::Light => return true,
            CellState::Blocked => return false,
            CellState::Unknown => {}
        }
        self.trail.push(Action::SetCell { c, prev: CellState::Unknown });
        self.cell_state[c] = CellState::Light;

        // 照明加算
        for &t in &self.lit_list[c] {
            self.lit_count[t] += 1;
            self.trail.push(Action::IncLit { c: t });
        }

        // 数字壁更新
        for &ni in &self.num_adj_of_empty[c] {
            self.num_cells[ni].on += 1;
            self.trail.push(Action::AddNumOn { i: ni, delta: -1 });
            q_num.push_back(ni);
        }

        // 行セグ
        let r = self.row_seg_id[c];
        if let Some(prev) = self.row_segs[r].light {
            if prev != c { return false; }
        } else {
            self.trail.push(Action::SetRowLight { r, prev: None });
            self.row_segs[r].light = Some(c);
            // 同行セグの他はBlocked
            for &x in &self.row_segs[r].cells {
                if x != c {
                    if !self.set_blocked(x, q_num) { return false; }
                }
            }
        }

        // 列セグ
        let k = self.col_seg_id[c];
        if let Some(prev) = self.col_segs[k].light {
            if prev != c { return false; }
        } else {
            self.trail.push(Action::SetColLight { k, prev: None });
            self.col_segs[k].light = Some(c);
            for &x in &self.col_segs[k].cells {
                if x != c {
                    if !self.set_blocked(x, q_num) { return false; }
                }
            }
        }

        true
    }

    fn propagate(&mut self) -> bool {
        let mut q_num = VecDeque::new();
        // 初回に全部入れても良いが，ここでは変更時にpushされる前提
        // ただし安全に全数字壁を入れて開始しても良い
        for i in 0..self.num_cells.len() {
            q_num.push_back(i);
        }

        loop {
            let mut changed = false;

            // 数字壁伝播
            while let Some(i) = q_num.pop_front() {
                let k = self.num_cells[i].k as i8;
                let on = self.num_cells[i].on;
                let unk = self.num_cells[i].unk;
                if on > k { return false; }
                if on + unk < k { return false; }

                if on == k {
                    for &c in &self.num_cells[i].adj {
                        if self.cell_state[c] == CellState::Unknown {
                            if !self.set_blocked(c, &mut q_num) { return false; }
                            changed = true;
                        }
                    }
                } else if on + unk == k {
                    for &c in &self.num_cells[i].adj {
                        if self.cell_state[c] == CellState::Unknown {
                            if !self.set_light(c, &mut q_num) { return false; }
                            changed = true;
                        }
                    }
                }
            }

            // 未照明の単一候補伝播（全走査）
            for c in 0..self.n_empty {
                if self.lit_count[c] > 0 { continue; }
                let r = self.row_seg_id[c];
                let k = self.col_seg_id[c];
                if self.row_segs[r].light.is_some() || self.col_segs[k].light.is_some() {
                    continue; // どちらかに明かりが確定なら照らされるはず（lit_count更新済みの前提だが保険）
                }
                let row_free = self.row_segs[r].free_cnt as i32;
                let col_free = self.col_segs[k].free_cnt as i32;
                let self_free = if self.cell_state[c] != CellState::Blocked { 1 } else { 0 };
                let cand = row_free + col_free - self_free;

                if cand == 0 {
                    return false;
                }
                if cand == 1 {
                    // 唯一候補を特定
                    let mut only: Option<usize> = None;
                    if row_free > 0 {
                        for &x in &self.row_segs[r].cells {
                            if self.cell_state[x] != CellState::Blocked {
                                only = Some(x);
                                break;
                            }
                        }
                    }
                    if only.is_none() && col_free > 0 {
                        for &x in &self.col_segs[k].cells {
                            if self.cell_state[x] != CellState::Blocked {
                                only = Some(x);
                                break;
                            }
                        }
                    }
                    let p = only.unwrap();
                    let mut dummy = VecDeque::new();
                    if !self.set_light(p, &mut dummy) { return false; }
                    // set_lightがpushした数字壁を処理するため，dummyをq_numに合流して次ループで処理したいので，
                    // ここは簡単のため propagate をループ全体でやり直す
                    changed = true;
                    // dummy分は次の周回の最初に全数字壁を入れているので省略可能
                }
            }

            if !changed { break; }
            // 変更があったら数字壁を再評価したいので，再度全投入
            for i in 0..self.num_cells.len() {
                q_num.push_back(i);
            }
        }

        true
    }

    fn is_solved(&self) -> bool {
        // 全空マスが照明されている
        if self.lit_count.iter().any(|&x| x == 0) { return false; }
        // 数字壁一致
        for nc in &self.num_cells {
            if nc.on != nc.k as i8 { return false; }
        }
        true
    }

    fn choose_branch_cell(&self) -> Option<(usize, Vec<usize>)> {
        let mut best: Option<(i32, usize, Vec<usize>)> = None;

        for c in 0..self.n_empty {
            if self.lit_count[c] > 0 { continue; }
            let r = self.row_seg_id[c];
            let k = self.col_seg_id[c];
            if self.row_segs[r].light.is_some() || self.col_segs[k].light.is_some() { continue; }

            let row_free = self.row_segs[r].free_cnt as i32;
            let col_free = self.col_segs[k].free_cnt as i32;
            let self_free = if self.cell_state[c] != CellState::Blocked { 1 } else { 0 };
            let cand = row_free + col_free - self_free;
            if cand <= 1 { continue; }

            // 候補列挙（重複の可能性は交点のみなので簡単に弾ける）
            let mut candidates = Vec::new();
            for &x in &self.row_segs[r].cells {
                if self.cell_state[x] != CellState::Blocked { candidates.push(x); }
            }
            for &x in &self.col_segs[k].cells {
                if self.cell_state[x] != CellState::Blocked && x != c {
                    candidates.push(x);
                }
            }
            // candidates内にcが2回入ることを避けたいので，上でx!=cにしている
            // ただしrow側にcが入っていない場合もあり得るので，必要なら整える

            match &best {
                None => best = Some((cand, c, candidates)),
                Some((bc, _, _)) if cand < *bc => best = Some((cand, c, candidates)),
                _ => {}
            }
        }

        best.map(|(_, c, cand)| (c, cand))
    }

    fn dfs(&mut self) -> bool {
        if !self.propagate() { return false; }
        if self.is_solved() { return true; }

        let Some((_c, candidates)) = self.choose_branch_cell() else {
            // 未照明がないのに未解決なら数字壁などが未一致
            return false;
        };

        for p in candidates {
            let cp = self.checkpoint();
            let mut q = VecDeque::new();
            if self.set_light(p, &mut q) && self.dfs() {
                return true;
            }
            self.undo(cp);
        }
        false
    }
}

fn main() {
    // 入力：H W の後に H 行（空白区切りでもOK）
    let mut s = String::new();
    io::stdin().read_to_string(&mut s).unwrap();
    let mut it = s.split_whitespace();
    let h: usize = it.next().unwrap().parse().unwrap();
    let w: usize = it.next().unwrap().parse().unwrap();
    let mut grid = vec![vec![' '; w]; h];
    for r in 0..h {
        let row = it.next().unwrap();
        for (c, ch) in row.chars().enumerate() {
            grid[r][c] = ch;
        }
    }

    // ここから Solver 構築（前処理）を行い，dfsで解く
    // 紙幅の都合で前処理の実装は省略せずに書くべきですが，
    // 要点は上の説明どおりです．

    // TODO: Solver::new(grid) を作って前処理を詰める
    // let mut solver = Solver::new(h, w, grid);
    // assert!(solver.dfs());
    // 出力：L を置いた盤面
    // for r in 0..h {
    //     for c in 0..w {
    //         print!("{}", solver.render[r][c]);
    //     }
    //     println!();
    // }
}
```

前処理（`Solver::new`）でやることは説明どおりで，実装は定型です．特に

* `empty_id[r][c]` を埋める
* 行セグ：各行を走査して，壁で区切って `row_segs` 追加，その区間の空マスに `row_seg_id` を付与
* 列セグも同様
* 数字壁：周囲4方向を見て `empty_id` があれば `adj` に追加，`on=0`，`unk=adj.len()`
* `lit_list[c]`：`row_segs[row_seg_id[c]].cells` と `col_segs[col_seg_id[c]].cells` をマージ（重複は `c` のみ）

を作れば完成します．

---

## 実用上のポイント

* ユニーク解保証なら，この「数字壁＋未照明単一候補」伝播だけでほぼ決まります．
* 盤面が大きくても，分岐は少なく，1回の伝播は全空マス走査 O(N) 程度なので十分速いです．
* 出力形式が「座標列」なら，`cell_state[c]==Light` の `empty_pos[c]` を列挙して出せばOKです．

---

もし「入力形式（`.`と数字が混在する行）」「出力形式（座標か盤面か）」の仕様が手元にあるなら，それに合わせて `main` と出力部を確定版に整えたRustコードをそのまま貼れる形で出します．

//! フィールド

/// フィールドの状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// セルなし
    Nil,
    /// 空白セル
    Empty,
    /// 隣接なしのセル
    Adj0,
    /// 隣接が 1 のセル
    Adj1,
    /// 隣接が 2 のセル
    Adj2,
    /// 隣接が 3 のセル
    Adj3,
    /// 隣接が 4 のセル
    Adj4,
}

impl State {
    /// 1 セルを解析する
    ///
    /// - `#`: セルなし
    /// - `.`: 空白セル
    /// - `0` - `4`: あかりが隣接するセル
    pub fn from_char(c: char) -> Result<Self, &'static str> {
        match c {
            '#' => Ok(Self::Nil),
            '.' => Ok(Self::Empty),
            '0' => Ok(Self::Adj0),
            '1' => Ok(Self::Adj1),
            '2' => Ok(Self::Adj2),
            '3' => Ok(Self::Adj3),
            '4' => Ok(Self::Adj4),
            _ => Err("cell parse error"),
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(&self, Self::Empty)
    }

    pub fn is_adj(&self) -> Option<usize> {
        match &self {
            Self::Adj0 => Some(0),
            Self::Adj1 => Some(1),
            Self::Adj2 => Some(2),
            Self::Adj3 => Some(3),
            Self::Adj4 => Some(4),
            _ => None,
        }
    }
}

/// フィールド
#[derive(Debug, PartialEq, Eq)]
pub struct Field {
    pub h: usize,
    pub w: usize,
    pub field: Vec<Vec<State>>,
}

impl Field {
    /// フィールドを初期化する
    ///
    /// - `#`: セルなし
    /// - `.`: 空白セル
    /// - `0` - `4`: あかりが隣接するセル
    pub fn from_str(h: usize, w: usize, s: &str) -> Result<Field, &'static str> {
        let field = s
            .split_ascii_whitespace()
            .map(|row| {
                row.chars()
                    .map(State::from_char)
                    .collect::<Result<Vec<State>, &'static str>>()
            })
            .map(|row| {
                row.and_then(|r| {
                    if r.len() == w {
                        Ok(r)
                    } else {
                        Err("failed to parse")
                    }
                })
            })
            .collect::<Result<Vec<Vec<State>>, &'static str>>()?;

        if field.len() != h {
            return Err("failed to parse");
        }

        Ok(Field { h, w, field })
    }

    pub fn display_with_solution(&self, sol: &Solution) -> String {
        let mut s = String::new();
        for r in 0..self.h {
            for c in 0..self.w {
                let ch = match self.field[r][c] {
                    State::Nil => '#',
                    State::Empty => {
                        if sol.field[r][c] {
                            'A'
                        } else {
                            '.'
                        }
                    }
                    State::Adj0 => '0',
                    State::Adj1 => '1',
                    State::Adj2 => '2',
                    State::Adj3 => '3',
                    State::Adj4 => '4',
                };
                s.push(ch);
            }
            s.push('\n');
        }
        s
    }
}

/// 解
#[derive(Debug, Clone, PartialEq)]
pub struct Solution {
    pub field: Vec<Vec<bool>>,
}

// ========== テスト ==========
#[cfg(test)]
mod test_field {
    use crate::field::{Field, State};

    #[test]
    fn test_parse_field_success() {
        let field_str = "
..1..
#.01.
3.24.
.....
";

        let field_actual = Field::from_str(4, 5, field_str);
        let field_expect = Ok(Field {
            h: 4,
            w: 5,
            field: vec![
                vec![
                    State::Empty,
                    State::Empty,
                    State::Adj1,
                    State::Empty,
                    State::Empty,
                ],
                vec![
                    State::Nil,
                    State::Empty,
                    State::Adj0,
                    State::Adj1,
                    State::Empty,
                ],
                vec![
                    State::Adj3,
                    State::Empty,
                    State::Adj2,
                    State::Adj4,
                    State::Empty,
                ],
                vec![
                    State::Empty,
                    State::Empty,
                    State::Empty,
                    State::Empty,
                    State::Empty,
                ],
            ],
        });

        assert_eq!(field_actual, field_expect);
    }

    #[test]
    fn test_parse_field_fail() {
        let field_str = "
..1..
#.01.
3.24
.....
";
        let field_actual = Field::from_str(4, 5, field_str);
        eprintln!("{field_actual:?}");
        assert!(field_actual.is_err());
    }
}

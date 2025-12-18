use akari::Field;
use serde::{Deserialize, Serialize};

/// Expected payload for solving a level.
#[derive(Debug, Deserialize)]
pub struct SolveRequest {
    pub problem: Vec<Vec<char>>,
}

/// Response body returned by the solver endpoint.
#[derive(Debug, Serialize)]
pub struct SolveResponse {
    pub solution: Option<Vec<(usize, usize)>>,
    pub error: Option<String>,
}

impl SolveResponse {
    pub fn solved(solution: Vec<(usize, usize)>) -> Self {
        Self {
            solution: Some(solution),
            error: None,
        }
    }

    pub fn failed(_req: &SolveRequest, message: impl Into<String>) -> Self {
        Self {
            solution: None,
            error: Some(message.into()),
        }
    }
}

impl SolveRequest {
    /// Convert the request into a parsed `Field`.
    pub fn to_field(&self) -> Result<Field, &'static str> {
        let (h, w, normalized) = parse_level_data(&self.problem)?;
        Field::from_str(h, w, &normalized)
    }
}

#[cfg(test)]
mod solve_request_tests {
    use super::SolveRequest;

    #[test]
    fn deserialize_problem_payload() {
        let json =
            "{\"problem\": [[\".\",\".\",\".\"],[\".\",\"#\",\".\"],[\".\",\".\",\".\"]]}";

        let req: SolveRequest = serde_json::from_str(json).unwrap();
        assert_eq!(
            req.problem,
            vec![
                vec!['.', '.', '.'],
                vec!['.', '#', '.'],
                vec!['.', '.', '.']
            ]
        );
    }
}

#[cfg(test)]
mod parse_level_data_tests {
    use super::parse_level_data;

    #[test]
    fn accepts_char_matrix() {
        let raw = vec![
            vec!['.', '.', '.'],
            vec!['1', '.', '.'],
            vec!['.', '#', '.'],
        ];
        let (h, w, normalized) = parse_level_data(&raw).unwrap();
        assert_eq!((h, w), (3, 3));
        assert_eq!(normalized, "...\n1..\n.#.\n");
    }
}

/// Normalize a char matrix into the format required by the solver.
pub fn parse_level_data(level_data: &[Vec<char>]) -> Result<(usize, usize, String), &'static str> {
    if level_data.is_empty() {
        return Err("level data is empty");
    }

    let width = level_data[0].len();
    if width == 0 {
        return Err("level data has zero width");
    }
    if level_data.iter().any(|row| row.len() != width) {
        return Err("row widths are inconsistent");
    }

    let height = level_data.len();
    let mut normalized = String::new();
    for row in level_data {
        for ch in row {
            normalized.push(*ch);
        }
        normalized.push('\n');
    }

    Ok((height, width, normalized))
}

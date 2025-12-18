use akari::Field;
use serde::{Deserialize, Serialize};

/// Expected payload for solving a level.
#[derive(Debug, Deserialize)]
pub struct SolveRequest {
    pub problem: String,
    pub timeout: Option<u64>,
}

/// Response body returned by the solver endpoint.
#[derive(Debug, Serialize)]
pub struct SolveResponse {
    pub solution: Option<String>,
    pub error: Option<String>,
}

impl SolveResponse {
    pub fn solved(solution: String) -> Self {
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
    /// Returns the puzzle string the solver consumes.
    pub fn level_data(&self) -> &str {
        &self.problem
    }

    /// Convert the request into a parsed `Field`.
    pub fn to_field(&self) -> Result<Field, &'static str> {
        let (h, w, normalized) = parse_level_data(self.level_data())?;
        Field::from_str(h, w, &normalized)
    }
}

#[cfg(test)]
mod solve_request_tests {
    use super::SolveRequest;

    #[test]
    fn deserialize_problem_payload() {
        let json = r#"{
            "problem": ". . .\n. # .\n. . .",
            "timeout": 3
        }"#;

        let req: SolveRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.problem, ". . .\n. # .\n. . .");
        assert_eq!(req.timeout, Some(3));
    }

    #[test]
    fn timeout_is_optional() {
        let json = r#"{
            "problem": ". . .\n. # .\n. . ."
        }"#;

        let req: SolveRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.problem, ". . .\n. # .\n. . .");
        assert_eq!(req.timeout, None);
    }
}

#[cfg(test)]
mod parse_level_data_tests {
    use super::parse_level_data;

    #[test]
    fn trims_metadata_and_whitespace() {
        let raw = r#". . .
. # .
. . .

Author: Foo
Difficulty: 3"#;

        let (h, w, normalized) = parse_level_data(raw).unwrap();
        assert_eq!((h, w), (3, 3));
        assert_eq!(normalized, "...\n.#.\n...\n");
    }

    #[test]
    fn accepts_space_delimited_rows() {
        let raw = ". . .\n1 . .\n. # .";
        let (h, w, normalized) = parse_level_data(raw).unwrap();
        assert_eq!((h, w), (3, 3));
        assert_eq!(normalized, "...\n1..\n.#.\n");
    }
}

/// Normalize the level layout into the format required by the solver.
pub fn parse_level_data(level_data: &str) -> Result<(usize, usize, String), &'static str> {
    let mut rows: Vec<String> = Vec::new();

    for line in level_data.lines() {
        let trimmed = line.trim();

        // Stop if we've hit metadata after at least one row.
        if trimmed.is_empty() {
            if !rows.is_empty() {
                break;
            }
            continue;
        }
        if trimmed.starts_with("Author:") || trimmed.starts_with("Difficulty:") {
            break;
        }

        let tokens: Vec<&str> = trimmed.split_whitespace().collect();
        if tokens.is_empty() {
            continue;
        }

        let row = if tokens.len() == 1 {
            tokens[0].to_string()
        } else {
            tokens.join("")
        };
        rows.push(row);
    }

    if rows.is_empty() {
        return Err("level data is empty");
    }

    let width = rows[0].len();
    if width == 0 {
        return Err("level data has zero width");
    }
    if rows.iter().any(|row| row.len() != width) {
        return Err("row widths are inconsistent");
    }

    let height = rows.len();
    let mut normalized = String::new();
    for row in rows {
        normalized.push_str(&row);
        normalized.push('\n');
    }

    Ok((height, width, normalized))
}

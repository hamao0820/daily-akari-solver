/// Solve a simple path variant of TSP over the given points using 2-opt local search.
///
/// The route starts from the cell closest to the top-right corner `(0, width - 1)`
/// and does **not** return to the start (i.e., a path instead of a cycle).
/// The function returns the visiting order as a list of indices into `points`.
/// Distance is measured with the Euclidean metric.
pub fn optimize_route_with_2opt(points: &[(usize, usize)], width: usize) -> Vec<usize> {
    let n = points.len();
    let mut route: Vec<usize> = (0..n).collect();

    if n < 2 {
        return route;
    }

    let top_right = (0, width.saturating_sub(1));
    let start_idx = points
        .iter()
        .enumerate()
        .min_by_key(|(_, &p)| euclidean_sq(p, top_right))
        .map(|(idx, _)| idx)
        .unwrap_or(0);
    route.rotate_left(start_idx);

    // Nothing to optimize for very short tours.
    if n < 4 {
        return route;
    }

    let mut improved = true;
    while improved {
        improved = false;

        for i in 0..n - 2 {
            for k in i + 2..n - 1 {
                let a = route[i];
                let b = route[i + 1];
                let c = route[k];
                let d = route[k + 1];

                let current = euclidean_sq(points[a], points[b]) + euclidean_sq(points[c], points[d]);
                let candidate = euclidean_sq(points[a], points[c]) + euclidean_sq(points[b], points[d]);

                if candidate < current {
                    route[i + 1..=k].reverse();
                    improved = true;
                }
            }
        }
    }

    route
}

fn euclidean_sq(a: (usize, usize), b: (usize, usize)) -> u64 {
    let dx = a.0 as i64 - b.0 as i64;
    let dy = a.1 as i64 - b.1 as i64;
    (dx * dx + dy * dy) as u64
}

#[cfg(test)]
fn path_length(order: &[usize], points: &[(usize, usize)]) -> u64 {
    order
        .windows(2)
        .map(|pair| euclidean_sq(points[pair[0]], points[pair[1]]))
        .sum::<u64>()
}

#[cfg(test)]
mod tests {
    use super::{optimize_route_with_2opt, path_length};

    #[test]
    fn rotates_small_input_to_nearest_start() {
        let points = vec![(0, 0), (1, 2), (3, 4)];
        let order = optimize_route_with_2opt(&points, 5);
        assert_eq!(order, vec![1, 2, 0]);
    }

    #[test]
    fn starts_at_top_right() {
        let points = vec![(2, 0), (0, 3), (1, 5), (3, 4)];
        let order = optimize_route_with_2opt(&points, 6);
        assert_eq!(order[0], 2);
    }

    #[test]
    fn shortens_crossing_route() {
        let points = vec![(0, 0), (10, 10), (0, 10), (10, 0)];
        let mut baseline = vec![0, 1, 2, 3];
        baseline.rotate_left(2);
        let baseline_len = path_length(&baseline, &points);

        let optimized = optimize_route_with_2opt(&points, 11);
        let optimized_len = path_length(&optimized, &points);

        // The tour should be a permutation of all points.
        let mut sorted = optimized.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, vec![0, 1, 2, 3]);

        assert!(optimized_len < baseline_len);
    }
}

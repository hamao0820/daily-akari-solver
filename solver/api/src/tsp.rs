/// Solve a simple TSP over the given points using 2-opt local search.
///
/// The function returns the visiting order as a list of indices into `points`.
/// Distance is measured with the Euclidean metric.
pub fn optimize_route_with_2opt(points: &[(usize, usize)]) -> Vec<usize> {
    let n = points.len();
    let mut route: Vec<usize> = (0..n).collect();

    // Nothing to optimize for very short tours.
    if n < 4 {
        return route;
    }

    let mut improved = true;
    while improved {
        improved = false;

        for i in 0..n - 2 {
            for k in i + 2..n {
                // Do not break the start-end edge of the tour.
                if i == 0 && k == n - 1 {
                    continue;
                }

                let a = route[i];
                let b = route[(i + 1) % n];
                let c = route[k];
                let d = route[(k + 1) % n];

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
fn tour_length(order: &[usize], points: &[(usize, usize)]) -> u64 {
    order
        .windows(2)
        .map(|pair| euclidean_sq(points[pair[0]], points[pair[1]]))
        .sum::<u64>()
        + euclidean_sq(points[*order.last().unwrap()], points[order[0]])
}

#[cfg(test)]
mod tests {
    use super::{optimize_route_with_2opt, tour_length};

    #[test]
    fn returns_identity_for_small_input() {
        let points = vec![(0, 0), (1, 2), (3, 4)];
        let order = optimize_route_with_2opt(&points);
        assert_eq!(order, vec![0, 1, 2]);
    }

    #[test]
    fn shortens_crossing_route() {
        let points = vec![(0, 0), (10, 10), (0, 10), (10, 0)];
        let baseline = vec![0, 1, 2, 3];
        let baseline_len = tour_length(&baseline, &points);

        let optimized = optimize_route_with_2opt(&points);
        let optimized_len = tour_length(&optimized, &points);

        // The tour should be a permutation of all points.
        let mut sorted = optimized.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, vec![0, 1, 2, 3]);

        assert!(optimized_len < baseline_len);
    }
}

use bevy::prelude::*;
use std::collections::{BinaryHeap, HashMap};

pub fn walkable(p: Vec2) -> bool {
    if p.x.abs() > 7.1 || p.y < -5.4 || p.y > 5.7 {
        return false;
    }
    for (x0, x1, z0, z1) in [
        (-6.9, -3.1, -5.6, -2.4),
        (1.2, 5.95, -5.8, -2.5),
        (-5.8, -3.15, 2.6, 3.3),
        (5.22, 6.37, -0.13, 0.94),
    ] {
        if p.x > x0 && p.x < x1 && p.y > z0 && p.y < z1 {
            return false;
        }
    }
    p.distance(Vec2::new(-0.2, 1.0)) > 1.13
}

const STEP: f32 = 0.28;
fn point(k: (i32, i32)) -> Vec2 {
    Vec2::new(k.0 as f32 * STEP, k.1 as f32 * STEP)
}
fn key(p: Vec2) -> (i32, i32) {
    ((p.x / STEP).round() as i32, (p.y / STEP).round() as i32)
}
fn heuristic(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs() + (a.1 - b.1).abs()
}
pub fn route(start: Vec2, target: Vec2) -> Vec<Vec2> {
    let s = key(start);
    let mut goal = key(target);
    if !walkable(point(goal)) {
        let mut best = f32::MAX;
        for x in -26..27 {
            for z in -20..22 {
                let k = (x, z);
                let p = point(k);
                let d = p.distance_squared(target);
                if walkable(p) && d < best {
                    best = d;
                    goal = k;
                }
            }
        }
    }
    let mut queue = BinaryHeap::new();
    queue.push((0, s));
    let mut costs = HashMap::from([(s, 0)]);
    let mut parent = HashMap::new();
    while let Some((_, current)) = queue.pop() {
        if current == goal {
            let mut path = vec![point(current)];
            let mut cursor = current;
            while let Some(prev) = parent.get(&cursor) {
                cursor = *prev;
                if cursor == s {
                    break;
                }
                path.push(point(cursor));
            }
            path.reverse();
            return path;
        }
        let cost = costs[&current];
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let next = (current.0 + dx, current.1 + dz);
            if !walkable(point(next)) {
                continue;
            }
            let nc = cost + 1;
            if costs.get(&next).is_none_or(|old| nc < *old) {
                costs.insert(next, nc);
                parent.insert(next, current);
                queue.push((-(nc + heuristic(next, goal)), next));
            }
        }
    }
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_clues_are_reachable_around_obstacles() {
        for c in findcat::game::CLUES {
            let target = Vec2::from_array(c.position);
            let path = route(Vec2::new(-3.0, -0.7), target);
            assert!(!path.is_empty(), "{}", c.title);
            assert!(path.iter().all(|p| walkable(*p)));
            assert!(path.last().unwrap().distance(target) < 1.5);
        }
    }
}

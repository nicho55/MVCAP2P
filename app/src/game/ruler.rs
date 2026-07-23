#![allow(dead_code)]

use std::collections::HashMap;

use crate::protocol::*;

use super::grid::cell_center;
use super::terrain::Terrain;

/// Distance between two cells in grid steps (ignoring elevation).
pub fn cell_distance(g: &GridCfg, a: Cell, b: Cell) -> u32 {
    match g.kind {
        GridKind::Square => {
            let dx = (a.0 - b.0).unsigned_abs();
            let dy = (a.1 - b.1).unsigned_abs();
            dx + dy
        }
        GridKind::HexFlat => {
            let (aq, ar) = a;
            let (bq, br) = b;
            let as_ = -aq - ar;
            let bs = -bq - br;
            let dq = (aq - bq).unsigned_abs();
            let dr = (ar - br).unsigned_abs();
            let ds = (as_ - bs).unsigned_abs();
            dq.max(dr).max(ds)
        }
    }
}

/// 3D distance between two cells factoring in elevation difference.
/// Returns planar grid distance + vertical delta (in cell-size units).
pub fn cell_distance_3d(g: &GridCfg, terrain: &Terrain, a: Cell, b: Cell) -> f32 {
    let planar = cell_distance(g, a, b) as f32;
    let elev_a = terrain.cells.get(&a).map(|c| c.elev as f32).unwrap_or(0.0);
    let elev_b = terrain.cells.get(&b).map(|c| c.elev as f32).unwrap_or(0.0);
    let dz = (elev_a - elev_b).abs();
    (planar * planar + dz * dz).sqrt()
}

/// All neighbors of a cell in the given grid type.
pub fn neighbors(g: &GridCfg, cell: Cell) -> Vec<Cell> {
    match g.kind {
        GridKind::Square => {
            vec![
                (cell.0 - 1, cell.1),
                (cell.0 + 1, cell.1),
                (cell.0, cell.1 - 1),
                (cell.0, cell.1 + 1),
            ]
        }
        GridKind::HexFlat => {
            vec![
                (cell.0 + 1, cell.1),
                (cell.0 - 1, cell.1),
                (cell.0, cell.1 + 1),
                (cell.0, cell.1 - 1),
                (cell.0 + 1, cell.1 - 1),
                (cell.0 - 1, cell.1 + 1),
            ]
        }
    }
}

/// Movement cost to enter a cell (1 = normal, u32::MAX = impassable).
pub fn move_cost(_g: &GridCfg, terrain: &Terrain, cell: Cell) -> u32 {
    match terrain.cells.get(&cell) {
        Some(_) => 1,
        None => 1,
    }
}

/// All cells within `radius` grid-steps of `origin`.
pub fn cells_in_radius(g: &GridCfg, origin: Cell, radius: u32) -> Vec<Cell> {
    let r = radius as i32;
    let mut result = Vec::new();
    match g.kind {
        GridKind::Square => {
            for dx in -r..=r {
                let dy_max = r - dx.abs();
                for dy in -dy_max..=dy_max {
                    result.push((origin.0 + dx, origin.1 + dy));
                }
            }
        }
        GridKind::HexFlat => {
            for dq in -r..=r {
                let r1 = (-r).max(-dq - r);
                let r2 = r.min(-dq + r);
                for dr in r1..=r2 {
                    result.push((origin.0 + dq, origin.1 + dr));
                }
            }
        }
    }
    result
}

/// Cells along a line from `from` to `to` (Bresenham-like, in grid space).
pub fn cells_in_line(g: &GridCfg, from: Cell, to: Cell) -> Vec<Cell> {
    let c_from = cell_center(g, from);
    let c_to = cell_center(g, to);
    let dist = cell_distance(g, from, to);
    if dist == 0 {
        return vec![from];
    }
    let n = dist.max(1) as usize;
    let mut result = Vec::with_capacity(n + 1);
    let mut prev = None;
    for i in 0..=n {
        let t = i as f32 / n as f32;
        let p = c_from.lerp(c_to, t);
        let cell = super::grid::world_to_cell(g, p);
        if prev != Some(cell) {
            result.push(cell);
            prev = Some(cell);
        }
    }
    if result.last() != Some(&to) {
        result.push(to);
    }
    result
}

/// Cells inside a cone: origin, direction angle (radians), half-angle spread, max range in cells.
pub fn cells_in_cone(
    g: &GridCfg,
    origin: Cell,
    direction_rad: f32,
    half_angle_rad: f32,
    range: u32,
) -> Vec<Cell> {
    let candidates = cells_in_radius(g, origin, range);
    let origin_pos = cell_center(g, origin);
    candidates
        .into_iter()
        .filter(|&c| {
            if c == origin {
                return false;
            }
            let pos = cell_center(g, c);
            let delta = pos - origin_pos;
            let angle = delta.y.atan2(delta.x);
            let mut diff = (angle - direction_rad).abs();
            if diff > std::f32::consts::PI {
                diff = 2.0 * std::f32::consts::PI - diff;
            }
            diff <= half_angle_rad
        })
        .collect()
}

/// Line of sight check: can `from` see `to`? Blocked if any intermediate cell
/// has elevation strictly higher than both endpoints.
pub fn line_of_sight(g: &GridCfg, terrain: &Terrain, from: Cell, to: Cell) -> bool {
    let line = cells_in_line(g, from, to);
    if line.len() <= 2 {
        return true;
    }
    let elev_from = terrain.cells.get(&from).map(|c| c.elev).unwrap_or(0);
    let elev_to = terrain.cells.get(&to).map(|c| c.elev).unwrap_or(0);
    let max_endpoint = elev_from.max(elev_to);
    for &cell in &line[1..line.len() - 1] {
        let elev = terrain.cells.get(&cell).map(|c| c.elev).unwrap_or(0);
        if elev > max_endpoint {
            return false;
        }
    }
    true
}

/// Check if a point is inside an area (list of cells).
pub fn contains(area: &[Cell], cell: Cell) -> bool {
    area.contains(&cell)
}

/// Navigable graph edge: neighbor + cost. For future A* pathfinding.
pub struct Edge {
    pub to: Cell,
    pub cost: u32,
}

/// Get all navigable edges from a cell (neighbors with movement cost).
pub fn edges(g: &GridCfg, terrain: &Terrain, cell: Cell) -> Vec<Edge> {
    neighbors(g, cell)
        .into_iter()
        .map(|n| {
            let cost = move_cost(g, terrain, n);
            Edge { to: n, cost }
        })
        .filter(|e| e.cost < u32::MAX)
        .collect()
}

/// Find cells that provide cover from a target (LoS blocked from target to these cells).
pub fn cover_cells(g: &GridCfg, terrain: &Terrain, target: Cell, search_radius: u32) -> Vec<Cell> {
    cells_in_radius(g, target, search_radius)
        .into_iter()
        .filter(|&c| c != target && !line_of_sight(g, terrain, target, c))
        .collect()
}

/// Simple A* pathfinding on the grid. Returns path (including start and end) or None.
pub fn pathfind(
    g: &GridCfg,
    terrain: &Terrain,
    start: Cell,
    goal: Cell,
    max_steps: u32,
) -> Option<Vec<Cell>> {
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;

    let mut open = BinaryHeap::new();
    let mut g_score: HashMap<Cell, u32> = HashMap::new();
    let mut came_from: HashMap<Cell, Cell> = HashMap::new();

    g_score.insert(start, 0);
    open.push(Reverse((cell_distance(g, start, goal), 0u32, start)));

    while let Some(Reverse((_, cost, current))) = open.pop() {
        if current == goal {
            let mut path = vec![current];
            let mut c = current;
            while let Some(&prev) = came_from.get(&c) {
                path.push(prev);
                c = prev;
            }
            path.reverse();
            return Some(path);
        }
        if cost > g_score.get(&current).copied().unwrap_or(u32::MAX) {
            continue;
        }
        if cost >= max_steps {
            continue;
        }
        for edge in edges(g, terrain, current) {
            let new_cost = cost + edge.cost;
            if new_cost < g_score.get(&edge.to).copied().unwrap_or(u32::MAX) {
                g_score.insert(edge.to, new_cost);
                came_from.insert(edge.to, current);
                let f = new_cost + cell_distance(g, edge.to, goal);
                open.push(Reverse((f, new_cost, edge.to)));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sq() -> GridCfg {
        GridCfg {
            kind: GridKind::Square,
            cell: 64.0,
        }
    }

    fn hex() -> GridCfg {
        GridCfg {
            kind: GridKind::HexFlat,
            cell: 64.0,
        }
    }

    fn empty_terrain() -> Terrain {
        Terrain {
            cells: HashMap::new(),
        }
    }

    #[test]
    fn square_distance_manhattan() {
        let g = sq();
        assert_eq!(cell_distance(&g, (0, 0), (3, 4)), 7);
        assert_eq!(cell_distance(&g, (0, 0), (0, 0)), 0);
        assert_eq!(cell_distance(&g, (-2, 3), (1, -1)), 7);
    }

    #[test]
    fn hex_distance_known_values() {
        let g = hex();
        assert_eq!(cell_distance(&g, (0, 0), (0, 0)), 0);
        assert_eq!(cell_distance(&g, (0, 0), (1, 0)), 1);
        assert_eq!(cell_distance(&g, (0, 0), (1, -1)), 1);
        assert_eq!(cell_distance(&g, (0, 0), (2, -1)), 2);
        assert_eq!(cell_distance(&g, (0, 0), (3, -3)), 3);
    }

    #[test]
    fn distance_3d_with_elevation() {
        let g = sq();
        let mut t = empty_terrain();
        t.cells.insert((0, 0), TerrainCell { tex: 0, elev: 0 });
        t.cells.insert((3, 0), TerrainCell { tex: 0, elev: 4 });
        let d = cell_distance_3d(&g, &t, (0, 0), (3, 0));
        assert!((d - 5.0).abs() < 0.01); // 3² + 4² = 25, √25 = 5
    }

    #[test]
    fn square_neighbors_count() {
        let g = sq();
        assert_eq!(neighbors(&g, (0, 0)).len(), 4);
    }

    #[test]
    fn hex_neighbors_count() {
        let g = hex();
        assert_eq!(neighbors(&g, (0, 0)).len(), 6);
    }

    #[test]
    fn radius_includes_origin() {
        let g = sq();
        let cells = cells_in_radius(&g, (0, 0), 1);
        assert!(cells.contains(&(0, 0)));
        assert_eq!(cells.len(), 5); // center + 4 adjacent
    }

    #[test]
    fn hex_radius_count() {
        let g = hex();
        let cells = cells_in_radius(&g, (0, 0), 1);
        assert_eq!(cells.len(), 7); // center + 6 neighbors
    }

    #[test]
    fn line_includes_endpoints() {
        let g = sq();
        let line = cells_in_line(&g, (0, 0), (5, 0));
        assert_eq!(line.first(), Some(&(0, 0)));
        assert_eq!(line.last(), Some(&(5, 0)));
    }

    #[test]
    fn line_single_cell() {
        let g = sq();
        let line = cells_in_line(&g, (2, 3), (2, 3));
        assert_eq!(line, vec![(2, 3)]);
    }

    #[test]
    fn los_clear_on_flat() {
        let g = sq();
        let t = empty_terrain();
        assert!(line_of_sight(&g, &t, (0, 0), (5, 0)));
    }

    #[test]
    fn los_blocked_by_wall() {
        let g = sq();
        let mut t = empty_terrain();
        t.cells.insert((0, 0), TerrainCell { tex: 0, elev: 0 });
        t.cells.insert((2, 0), TerrainCell { tex: 0, elev: 5 });
        t.cells.insert((4, 0), TerrainCell { tex: 0, elev: 0 });
        assert!(!line_of_sight(&g, &t, (0, 0), (4, 0)));
    }

    #[test]
    fn los_not_blocked_when_observer_high() {
        let g = sq();
        let mut t = empty_terrain();
        t.cells.insert((0, 0), TerrainCell { tex: 0, elev: 10 });
        t.cells.insert((2, 0), TerrainCell { tex: 0, elev: 5 });
        t.cells.insert((4, 0), TerrainCell { tex: 0, elev: 0 });
        assert!(line_of_sight(&g, &t, (0, 0), (4, 0)));
    }

    #[test]
    fn cone_excludes_origin() {
        let g = sq();
        let cells = cells_in_cone(&g, (0, 0), 0.0, std::f32::consts::FRAC_PI_4, 3);
        assert!(!cells.contains(&(0, 0)));
    }

    #[test]
    fn contains_works() {
        let area = vec![(0, 0), (1, 0), (0, 1)];
        assert!(contains(&area, (1, 0)));
        assert!(!contains(&area, (2, 2)));
    }

    #[test]
    fn pathfind_straight_line() {
        let g = sq();
        let t = empty_terrain();
        let path = pathfind(&g, &t, (0, 0), (3, 0), 100).unwrap();
        assert_eq!(path.first(), Some(&(0, 0)));
        assert_eq!(path.last(), Some(&(3, 0)));
        assert_eq!(path.len(), 4);
    }

    #[test]
    fn pathfind_respects_max_steps() {
        let g = sq();
        let t = empty_terrain();
        assert!(pathfind(&g, &t, (0, 0), (100, 0), 5).is_none());
    }

    #[test]
    fn cover_cells_behind_wall() {
        let g = sq();
        let mut t = empty_terrain();
        t.cells.insert((2, 0), TerrainCell { tex: 0, elev: 10 });
        let covers = cover_cells(&g, &t, (0, 0), 5);
        assert!(covers.contains(&(4, 0)));
    }

    #[test]
    fn edges_returns_neighbors() {
        let g = sq();
        let t = empty_terrain();
        let e = edges(&g, &t, (0, 0));
        assert_eq!(e.len(), 4);
        assert!(e.iter().all(|edge| edge.cost == 1));
    }
}

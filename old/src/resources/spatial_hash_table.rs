use std::collections::HashSet;
use std::hash::Hash;
use itertools::Itertools;
use super::{Viewport, ViewportTransform};
use crate::utilities::{Vector2, AABB, Intersect};
use crate::components::{Point, Line, Circle};

static TILE_SIZE : f64 = 40.0;

#[derive(Debug)]
pub struct SpatialHashTable<T: Clone + Eq + Hash> {
  x_tiles: usize,
  y_tiles: usize,
  table: Vec<HashSet<T>>,
}

pub type Tile = usize;

impl<T: Clone + Eq + Hash> Default for SpatialHashTable<T> {
  fn default() -> Self {
    Self { x_tiles: 0, y_tiles: 0, table: vec![] }
  }
}

impl<T: Clone + Eq + Hash> SpatialHashTable<T> {
  pub fn init_viewport(&mut self, vp: &Viewport) {
    self.x_tiles = (vp.actual_width() / TILE_SIZE).ceil() as usize;
    self.y_tiles = (vp.actual_height() / TILE_SIZE).ceil() as usize;
    self.table = vec![HashSet::new(); self.x_tiles * self.y_tiles];
  }

  // p: point in virtual space
  pub fn insert_point(&mut self, ent: T, p: Point, vp: &Viewport) {
    if let Some(id) = self.get_cell(p.to_actual(vp)) {
      self.table[id].insert(ent);
    }
  }

  /// l: line in virtual space
  pub fn insert_line(&mut self, ent: T, l: Line, vp: &Viewport) {
    let aabb = vp.actual_aabb();
    let actual = l.to_actual(vp);
    if let Some((p1, p2)) = actual.intersect(aabb) {

      // Making sure p1 to p2 is from left to right
      let (p1, p2) = if p1.x > p2.x { (p2, p1) } else { (p1, p2) };
      let dir = (p2 - p1).normalized() * 0.000001;
      let p1 = p1 + dir;
      let (init_x_tile, init_y_tile) = self.get_unlimited_cell(p1);
      let (end_x_tile, end_y_tile) = self.get_unlimited_cell(p2);

      if init_x_tile == end_x_tile && init_x_tile >= 0 && init_x_tile < self.x_tiles as i64 {
        let (init_y_tile, end_y_tile) = if init_y_tile <= end_y_tile {
          (init_y_tile, end_y_tile)
        } else {
          (end_y_tile, init_y_tile)
        };
        for i in (init_y_tile.max(0))..((end_y_tile + 1).min(self.y_tiles as i64)) {
          let tile = self.get_cell_by_x_y(init_x_tile as usize, i as usize);
          self.table[tile].insert(ent.clone());
        }
      } else {

        // Setupt the state
        let yi = if dir.y < 0.0 { -1.0 } else { 1.0 };
        let mut curr_x = p1.x;
        let mut curr_y = p1.y;
        let mut curr_x_tile = init_x_tile as i64;
        let mut curr_y_tile = init_y_tile as i64;

        // Go through all the x tile in the same row that are covered by the line
        while curr_x_tile <= end_x_tile as i64 && 0 <= curr_y_tile && curr_y_tile < self.y_tiles as i64 {
          let next_y = (curr_y_tile + if dir.y > 0.0 { 1 } else { 0 }) as f64 * TILE_SIZE;
          let tile_offset_y = (next_y - curr_y) * yi;
          let next_x_diff = tile_offset_y / dir.y.abs() * dir.x;
          let next_x = curr_x + next_x_diff;
          let next_x_tile = (next_x / TILE_SIZE) as i64;
          for tile_x in curr_x_tile..(next_x_tile + 1) {
            if tile_x <= end_x_tile as i64 && tile_x < self.x_tiles as i64 {
              let tile = self.get_cell_by_x_y(tile_x as usize, curr_y_tile as usize);
              assert!(
                tile < self.x_tiles * self.y_tiles,
                "Inserting line into bad cell. Line: {:?}, tile_x: {:?}, tile_y: {:?}, curr_x_tile: {:?}",
                actual,
                tile_x,
                curr_y_tile,
                curr_x_tile,
              );
              self.table[tile].insert(ent.clone());
            }
          }
          curr_x = next_x;
          curr_y = next_y;
          curr_x_tile = next_x_tile;
          curr_y_tile = curr_y_tile + yi as i64;
        }
      }
    }
  }

  pub fn insert_circle(&mut self, ent: T, c: Circle, vp: &Viewport) {
    let actual_center = c.center.to_actual(vp);
    let actual_radius = c.radius.to_actual(vp);
    let (left, top) = self.get_unlimited_cell(vec2![actual_center.x - actual_radius, actual_center.y - actual_radius]);
    let (right, bottom) = self.get_unlimited_cell(vec2![actual_center.x + actual_radius, actual_center.y + actual_radius]);
    for j in top.max(0)..(bottom.min(self.x_tiles as i64) + 1) {
      for i in left.max(0)..(right.min(self.y_tiles as i64) + 1) {
        if 0 <= i && i < self.x_tiles as i64 && 0 <= j && j < self.y_tiles as i64 {
          let cell_aabb = AABB::new(i as f64 * TILE_SIZE, j as f64 * TILE_SIZE, TILE_SIZE, TILE_SIZE);
          let closest_dist = (cell_aabb.get_closest_point_to(actual_center) - actual_center).magnitude();
          let furthest_dist = (cell_aabb.get_furthest_point_to(actual_center) - actual_center).magnitude();
          if closest_dist <= actual_radius && closest_dist <= furthest_dist {
            let tile = self.get_cell_by_x_y(i as usize, j as usize);
            assert!(tile < self.x_tiles * self.y_tiles, "Inserting circle into bad cell. tile_x: {:?}, tile_y: {:?}", i, j);
            self.table[tile].insert(ent.clone());
          }
        }
      }
    }
  }

  pub fn remove_from_all(&mut self, ent: T) {
    for cell in &mut self.table {
      cell.remove(&ent);
    }
  }

  #[allow(dead_code)]
  pub fn clear(&mut self) {
    for cell in &mut self.table {
      cell.clear();
    }
  }

  /// p: point in actual space
  fn get_cell(&self, p: Point) -> Option<Tile> {
    let Vector2 { x, y } = p;
    let x_tile = (x / TILE_SIZE).floor();
    let y_tile = (y / TILE_SIZE).floor();
    if 0.0 <= x_tile && x_tile < self.x_tiles as f64 && 0.0 <= y_tile && y_tile < self.y_tiles as f64 {
      Some(self.get_cell_by_x_y(x_tile as usize, y_tile as usize))
    } else {
      None
    }
  }

  fn get_unlimited_cell(&self, p: Point) -> (i64, i64) {
    let Vector2 { x, y } = p;
    let x_tile = (x / TILE_SIZE).floor() as i64;
    let y_tile = (y / TILE_SIZE).floor() as i64;
    (x_tile, y_tile)
  }

  fn get_cell_by_x_y(&self, x_tile: usize, y_tile: usize) -> Tile {
    (y_tile * self.x_tiles) + x_tile
  }

  /// aabb: AABB in actual space
  pub fn get_neighbor_entities_of_aabb(&self, aabb: AABB) -> HashSet<T> {
    let (i_min, j_min) = self.get_unlimited_cell(vec2![aabb.x, aabb.y]);
    let (i_max, j_max) = self.get_unlimited_cell(vec2![aabb.x + aabb.width, aabb.y + aabb.height]);

    let mut result = HashSet::new();
    for j in j_min..(j_max + 1) {
      for i in i_min..(i_max + 1) {
        if 0 <= i && i < self.x_tiles as i64 && 0 <= j && j < self.y_tiles as i64 {
          let tile = self.get_cell_by_x_y(i as usize, j as usize);
          for entity in &self.table[tile] {
            result.insert(entity.clone());
          }
        }
      }
    }
    result
  }

  /// p: point in virtual space
  pub fn get_neighbor_entities_of_point(&self, p: Point, vp: &Viewport) -> Option<Vec<T>> {
    if let Some(center_tile) = self.get_cell(p.to_actual(vp)) {
      let mut tiles = vec![center_tile];

      let left = !self.is_left_border(center_tile);
      let right = !self.is_right_border(center_tile);
      let top = !self.is_top_border(center_tile);
      let bottom = !self.is_bottom_border(center_tile);

      if left { tiles.push(center_tile - 1) };
      if right { tiles.push(center_tile + 1) };
      if top { tiles.push(center_tile - self.x_tiles) };
      if bottom { tiles.push(center_tile + self.x_tiles) };
      if left && top { tiles.push(center_tile - self.x_tiles - 1) };
      if left && bottom { tiles.push(center_tile + self.x_tiles - 1) };
      if right && top { tiles.push(center_tile - self.x_tiles + 1) };
      if right && bottom { tiles.push(center_tile + self.x_tiles + 1) };

      Some(tiles.into_iter().map(|tile| self.table[tile].clone()).flatten().unique().collect())
    } else {
      None
    }
  }

  fn is_left_border(&self, tile: Tile) -> bool {
    tile % self.x_tiles == 0
  }

  fn is_right_border(&self, tile: Tile) -> bool {
    tile % self.x_tiles == self.x_tiles - 1
  }

  fn is_top_border(&self, tile: Tile) -> bool {
    tile / self.x_tiles < 1
  }

  fn is_bottom_border(&self, tile: Tile) -> bool {
    tile / self.x_tiles >= self.y_tiles - 1
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::utilities::LineType;

  #[test]
  fn test_insert_point_1() {
    let vp = &Viewport::new(vec2![0., 0.], vec2![2., 2.], vec2![80., 80.]); // 田
    let mut table : SpatialHashTable<i32> = SpatialHashTable::default();
    table.init_viewport(vp);

    let p = vec2![0.0, 0.0];
    table.insert_point(0, p, vp);

    assert!(table.table[0].is_empty());
    assert!(table.table[1].is_empty());
    assert!(table.table[2].is_empty());
    assert!(table.table[3].contains(&0));
  }

  #[test]
  fn test_insert_point_2() {
    let vp = &Viewport::new(vec2![0., 0.], vec2![2., 2.], vec2![80., 80.]); // 田
    let mut table : SpatialHashTable<i32> = SpatialHashTable::default();
    table.init_viewport(vp);

    let p = vec2![0.5, -0.5];
    table.insert_point(0, p, vp);

    assert!(table.table[0].is_empty());
    assert!(table.table[1].is_empty());
    assert!(table.table[2].is_empty());
    assert!(table.table[3].contains(&0));
  }

  #[test]
  fn test_insert_line_1() {
    let vp = &Viewport::new(vec2![0., 0.], vec2![2., 2.], vec2![80., 80.]); // 田
    let mut table : SpatialHashTable<i32> = SpatialHashTable::default();
    table.init_viewport(vp);

    let l = Line { origin: vec2![-0.5, 0.0], direction: vec2![0.0, 1.0], ..Default::default() };
    table.insert_line(0, l, vp);

    assert!(table.table[0].contains(&0));
    assert!(table.table[1].is_empty());
    assert!(table.table[2].contains(&0));
    assert!(table.table[3].is_empty());
  }

  /// 0 - - 1 - - +
  /// |     |/    |
  /// |    /|     |
  /// 2 - - 3 - - +
  /// | /   |     |
  /// |     |     |
  /// + - - + - - +
  #[test]
  fn test_insert_line_2() {
    let vp = &Viewport::new(vec2![0., 0.], vec2![2., 2.], vec2![80., 80.]); // 田
    let mut table : SpatialHashTable<i32> = SpatialHashTable::default();
    table.init_viewport(vp);

    let l = Line { origin: vec2![-0.5, 0.0], direction: vec2![(2.0 as f64).sqrt(), (2.0 as f64).sqrt()] / 2.0, ..Default::default() };
    table.insert_line(0, l, vp);

    println!("{:?}", table);

    assert!(table.table[0].contains(&0));
    assert!(table.table[1].contains(&0));
    assert!(table.table[2].contains(&0));
    assert!(table.table[3].is_empty());
  }

  /// + - - + - - +
  /// |     |     |
  /// | \   |     |
  /// + - - + - - +
  /// |    \|     |
  /// |     |\    |
  /// + - - + - - +
  #[test]
  fn test_insert_line_3() {
    let vp = &Viewport::new(vec2![0., 0.], vec2![2., 2.], vec2![80., 80.]); // 田
    let mut table : SpatialHashTable<i32> = SpatialHashTable::default();
    table.init_viewport(vp);

    let l = Line { origin: vec2![-0.5, 0.0], direction: vec2![(2.0 as f64).sqrt(), -(2.0 as f64).sqrt()] / 2.0, ..Default::default() };
    table.insert_line(0, l, vp);

    println!("{:?}", table);

    assert!(table.table[0].contains(&0));
    assert!(table.table[1].is_empty());
    assert!(table.table[2].contains(&0));
    assert!(table.table[3].contains(&0));
  }

  #[test]
  fn test_insert_line_4() {
    let vp = &Viewport::new(vec2![0., 0.], vec2![4., 4.], vec2![160., 160.]); // 田
    let mut table : SpatialHashTable<i32> = SpatialHashTable::default();
    table.init_viewport(vp);

    let l = Line { origin: vec2![-0.5, 0.0], direction: vec2![(2.0 as f64).sqrt(), (2.0 as f64).sqrt()] / 2.0, ..Default::default() };
    table.insert_line(0, l, vp);

    println!("{:?}", table);

    for i in 0..16 {
      match i {
        2 | 3 | 5 | 6 | 8 | 9 | 12 => assert!(table.table[i].contains(&0)),
        _ => assert!(table.table[i].is_empty())
      }
    }
  }

  #[test]
  fn test_insert_line_5() {
    let vp = &Viewport::new(vec2![0., 0.], vec2![4., 4.], vec2![160., 160.]); // 田
    let mut table : SpatialHashTable<i32> = SpatialHashTable::default();
    table.init_viewport(vp);

    let sqrt17 = (17.0 as f64).sqrt();
    let l = Line { origin: vec2![0.0, -0.1], direction: vec2![4.0, 1.0] / sqrt17, ..Default::default() };
    table.insert_line(0, l, vp);

    println!("{:?}", table);

    for i in 0..16 {
      match i {
        6 | 7 | 8 | 9 | 10 => assert!(table.table[i].contains(&0)),
        _ => assert!(table.table[i].is_empty())
      }
    }
  }

  #[test]
  fn test_insert_line_6() {
    let vp = &Viewport::new(vec2![0., 0.], vec2![2., 2.], vec2![80., 80.]); // 田
    let mut table : SpatialHashTable<i32> = SpatialHashTable::default();
    table.init_viewport(vp);

    let l = Line { origin: vec2![0.0, -0.5], direction: vec2![(2.0 as f64).sqrt(), (2.0 as f64).sqrt()] / 2.0, ..Default::default() };
    table.insert_line(0, l, vp);

    println!("{:?}", table);

    assert!(table.table[0].is_empty());
    assert!(table.table[1].contains(&0));
    assert!(table.table[2].contains(&0));
    assert!(table.table[3].contains(&0));
  }

  /// + - - + - - +
  /// |     |   / |
  /// |     |  /  |
  /// + - - + - - +
  /// |     |/    |
  /// |     |     |
  /// + - - + - - +
  #[test]
  fn test_insert_ray_1() {
    let vp = &Viewport::new(vec2![0., 0.], vec2![2., 2.], vec2![80., 80.]); // 田
    let mut table : SpatialHashTable<i32> = SpatialHashTable::default();
    table.init_viewport(vp);

    let l = Line { origin: vec2![0.1, -0.5], direction: vec2![(2.0 as f64).sqrt(), (2.0 as f64).sqrt()] / 2.0, line_type: LineType::Ray };
    table.insert_line(0, l, vp);

    println!("{:?}", table);

    assert!(table.table[0].is_empty());
    assert!(table.table[1].contains(&0));
    assert!(table.table[2].is_empty());
    assert!(table.table[3].contains(&0));
  }

  #[test]
  fn test_insert_ray_2() {
    let vp = &Viewport::new(vec2![0., 0.], vec2![2., 2.], vec2![80., 80.]); // 田
    let mut table : SpatialHashTable<i32> = SpatialHashTable::default();
    table.init_viewport(vp);

    let l = Line { origin: vec2![-0.1, -0.5], direction: vec2![(2.0 as f64).sqrt(), (2.0 as f64).sqrt()] / 2.0, line_type: LineType::Ray };
    table.insert_line(0, l, vp);

    println!("{:?}", table);

    assert!(table.table[0].is_empty());
    assert!(table.table[1].contains(&0));
    assert!(table.table[2].contains(&0));
    assert!(table.table[3].contains(&0));
  }

  /// + - - + - - +
  /// |     |     |
  /// |     |     |
  /// + - - + - - +
  /// |     |     |
  /// |    /|     |
  /// + - - + - - +
  #[test]
  fn test_insert_ray_3() {
    let vp = &Viewport::new(vec2![0., 0.], vec2![2., 2.], vec2![80., 80.]); // 田
    let mut table : SpatialHashTable<i32> = SpatialHashTable::default();
    table.init_viewport(vp);

    let l = Line { origin: vec2![-0.1, -0.5], direction: vec2![-(2.0 as f64).sqrt(), -(2.0 as f64).sqrt()] / 2.0, line_type: LineType::Ray };
    table.insert_line(0, l, vp);

    println!("{:?}", table);

    assert!(table.table[0].is_empty());
    assert!(table.table[1].is_empty());
    assert!(table.table[2].contains(&0));
    assert!(table.table[3].is_empty());
  }

  #[test]
  fn test_insert_ray_4() {
    let vp = &Viewport::new(vec2![0., 0.], vec2![2., 2.], vec2![80., 80.]); // 田
    let mut table : SpatialHashTable<i32> = SpatialHashTable::default();
    table.init_viewport(vp);

    let l = Line { origin: vec2![-0.5, -1.5], direction: vec2![-(2.0 as f64).sqrt(), -(2.0 as f64).sqrt()] / 2.0, line_type: LineType::Ray };
    table.insert_line(0, l, vp);

    println!("{:?}", table);

    assert!(table.table[0].is_empty());
    assert!(table.table[1].is_empty());
    assert!(table.table[2].is_empty());
    assert!(table.table[3].is_empty());
  }

  #[test]
  fn test_insert_segment_1() {
    let vp = &Viewport::new(vec2![0., 0.], vec2![2., 2.], vec2![80., 80.]); // 田
    let mut table : SpatialHashTable<i32> = SpatialHashTable::default();
    table.init_viewport(vp);

    let l = Line { origin: vec2![-0.4, -1.5], direction: vec2![(2.0 as f64).sqrt(), (2.0 as f64).sqrt()] / 2.0, line_type: LineType::Segment(5.0) };
    table.insert_line(0, l, vp);

    println!("{:?}", table);

    assert!(table.table[0].is_empty());
    assert!(table.table[1].is_empty());
    assert!(table.table[2].is_empty());
    assert!(table.table[3].contains(&0));
  }

  #[test]
  fn test_insert_segment_2() {
    let vp = &Viewport::new(vec2![0., 0.], vec2![2., 2.], vec2![80., 80.]); // 田
    let mut table : SpatialHashTable<i32> = SpatialHashTable::default();
    table.init_viewport(vp);

    let l = Line { origin: vec2![-0.4, -1.5], direction: vec2![(2.0 as f64).sqrt(), (2.0 as f64).sqrt()] / 2.0, line_type: LineType::Segment(1.2) };
    table.insert_line(0, l, vp);

    println!("{:?}", table);

    assert!(table.table[0].is_empty());
    assert!(table.table[1].is_empty());
    assert!(table.table[2].is_empty());
    assert!(table.table[3].contains(&0));
  }

  #[test]
  fn test_insert_circle_1() {
    let vp = &Viewport::new(vec2![0., 0.], vec2![3., 3.], vec2![120., 120.]);
    let mut table : SpatialHashTable<i32> = SpatialHashTable::default();
    table.init_viewport(vp);

    let c = Circle { center: vec2![0.0, 0.0], radius: 1. };
    table.insert_circle(0, c, vp);

    println!("{:?}", table);
  }

  use rand::Rng;

  fn random_line_type<R: Rng>(rng: &mut R, max_length: f64) -> LineType {
    let r = rng.gen_range(0, 3);
    match r {
      0 => LineType::Line,
      1 => LineType::Ray,
      _ => LineType::Segment(rng.gen_range(0.0, max_length)),
    }
  }

  #[test]
  fn test_random_line() {
    let x_max = 1.;
    let y_max = 1.;
    let vp_w = 320.;
    let vp_h = 320.;

    let vp = &Viewport::new(vec2![0., 0.], vec2![2.0 * x_max, 2.0 * y_max], vec2![vp_w, vp_h]);
    let actual_aabb = vp.actual_aabb();
    let mut table : SpatialHashTable<i32> = SpatialHashTable::default();
    table.init_viewport(vp);

    let mut rng = rand::thread_rng();

    for line_id in 0..100 {

      table.clear();

      let theta = rng.gen_range(-std::f64::consts::PI, std::f64::consts::PI);
      let l = Line {
        origin: vec2![rng.gen_range(-x_max, x_max), rng.gen_range(-y_max, y_max)],
        direction: vec2![theta.cos(), theta.sin()],
        line_type: random_line_type(&mut rng, 2.0 * x_max),
      };
      table.insert_line(line_id, l, vp);
      let actual_line = l.to_actual(vp);
      for _ in 0..100 {
        let t = match actual_line.line_type {
          LineType::Line => rng.gen_range(-vp_w, vp_w),
          LineType::Ray => rng.gen_range(0., vp_w),
          LineType::Segment(t) => rng.gen_range(0., t),
        };
        let p = actual_line.origin + actual_line.direction * t;
        if actual_aabb.contains(p) {
          let cell = table.get_cell(p);
          if let Some(cell) = cell {
            assert!(table.table[cell].contains(&line_id), "Should contain! \nTable: {:?}, \nLine: {:?}, \nActual Line: {:?}, \nPoint: {:?}, \nt: {}, \nCell: {}", table, l, actual_line, p, t, cell);
          } else {
            assert!(false, "Should have a cell! Table: {:?}, Line: {:?}, Point: {:?}", table, l, p);
          }
        }
      }
    }
  }

  #[test]
  fn test_random_line_fixed_1() {
    let x_max = 1.;
    let y_max = 1.;
    let vp_w = 80.;
    let vp_h = 80.;

    let vp = &Viewport::new(vec2![0., 0.], vec2![2.0 * x_max, 2.0 * y_max], vec2![vp_w, vp_h]);
    let actual_aabb = vp.actual_aabb();
    let mut table : SpatialHashTable<i32> = SpatialHashTable::default();
    table.init_viewport(vp);

    let l = Line {
      origin: vec2![0.4987389654749186, 0.08770535401554502],
      direction: vec2![-0.4210742727328035, -0.9070261610574089],
      line_type: LineType::Ray,
    };
    table.insert_line(0, l, vp);

    let actual_line = l.to_actual(vp);

    let t = 47.55087108142186;

    let p = actual_line.origin + actual_line.direction * t;

    if actual_aabb.contains(p) {
      let cell = table.get_cell(p);
      if let Some(cell) = cell {
        assert!(table.table[cell].contains(&0), "Should contain! \nTable: {:?}, \nLine: {:?}, \nActual Line: {:?}, \nPoint: {:?}, \nt: {}, \nCell: {}", table, l, actual_line, p, t, cell);
      } else {
        assert!(false, "Should have a cell! Table: {:?}, Line: {:?}, Point: {:?}", table, l, p);
      }
    }
  }
}
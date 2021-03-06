use crate::math::*;
use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
pub struct ScreenScalar(pub f64);

impl Into<f64> for ScreenScalar {
  fn into(self) -> f64 {
    self.0
  }
}

impl From<f64> for ScreenScalar {
  fn from(f: f64) -> Self {
    Self(f)
  }
}

impl Div<ScreenScalar> for ScreenScalar {
  type Output = f64;

  fn div(self, other: Self) -> f64 {
    self.0 / other.0
  }
}

#[derive(Debug, Clone, Copy)]
pub struct ScreenPosition(pub Vector2);

impl ScreenPosition {
  pub fn magnitude(self) -> ScreenScalar {
    ScreenScalar(self.0.magnitude())
  }
}

impl Into<Vector2> for ScreenPosition {
  fn into(self) -> Vector2 {
    self.0
  }
}

impl From<Vector2> for ScreenPosition {
  fn from(v: Vector2) -> Self {
    Self(v)
  }
}

impl Add for ScreenPosition {
  type Output = Self;
  fn add(self, other: Self) -> Self {
    Self(self.0 + other.0)
  }
}

impl Sub for ScreenPosition {
  type Output = Self;
  fn sub(self, other: Self) -> Self {
    Self(self.0 - other.0)
  }
}

impl Neg for ScreenPosition {
  type Output = Self;
  fn neg(self) -> Self {
    Self(-self.0)
  }
}

impl Mul<ScreenScalar> for ScreenPosition {
  type Output = Self;
  fn mul(self, other: ScreenScalar) -> Self {
    Self(self.0 * other.0)
  }
}

impl Mul<ScreenPosition> for ScreenScalar {
  type Output = ScreenPosition;
  fn mul(self, other: ScreenPosition) -> ScreenPosition {
    ScreenPosition(self.0 * other.0)
  }
}

impl Div<ScreenScalar> for ScreenPosition {
  type Output = Self;
  fn div(self, other: ScreenScalar) -> Self {
    Self(self.0 / other.0)
  }
}

#[derive(Debug, Clone, Copy)]
pub struct ScreenLine {
  pub from: ScreenPosition,
  pub to: ScreenPosition,
  pub line_type: LineType,
}

impl ScreenLine {
  pub fn from_to_length(self) -> ScreenScalar {
    let l: Line = self.into();
    l.from_to_length().into()
  }

  pub fn get_closest_point(self, p: ScreenPosition) -> ScreenPosition {
    let l: Line = self.into();
    l.get_closest_point(p.into()).into()
  }

  pub fn t_of_point(self, p: ScreenPosition) -> ScreenScalar {
    let l: Line = self.into();
    l.t_of_point(p.into()).into()
  }

  pub fn rel_t_of_point(self, p: ScreenPosition) -> f64 {
    let l: Line = self.into();
    l.rel_t_of_point(p.into()).into()
  }
}

impl Into<Line> for ScreenLine {
  fn into(self) -> Line {
    Line {
      from: self.from.into(),
      to: self.to.into(),
      line_type: self.line_type,
    }
  }
}

impl From<Line> for ScreenLine {
  fn from(l: Line) -> Self {
    Self {
      from: l.from.into(),
      to: l.to.into(),
      line_type: l.line_type,
    }
  }
}

impl Project<ScreenLine> for ScreenPosition {
  type Output = Self;

  fn project(self, target: ScreenLine) -> Self {
    let l: Line = target.into();
    self.0.project(l).into()
  }
}

impl Intersect<ScreenLine> for ScreenLine {
  type Output = Option<ScreenPosition>;

  fn intersect(self, other: Self) -> Self::Output {
    let l1: Line = self.into();
    let l2: Line = other.into();
    l1.intersect(l2).map(ScreenPosition)
  }
}

impl Intersect<AABB> for ScreenLine {
  type Output = Option<(ScreenPosition, ScreenPosition)>;

  fn intersect(self, other: AABB) -> Self::Output {
    let l: Line = self.into();
    l.intersect(other)
      .map(|(p1, p2)| (ScreenPosition(p1), ScreenPosition(p2)))
  }
}

#[derive(Debug, Clone, Copy)]
pub struct ScreenCircle {
  pub center: ScreenPosition,
  pub radius: ScreenScalar,
}

impl Into<Circle> for ScreenCircle {
  fn into(self) -> Circle {
    Circle {
      center: self.center.into(),
      radius: self.radius.into(),
    }
  }
}

impl From<Circle> for ScreenCircle {
  fn from(c: Circle) -> Self {
    Self {
      center: c.center.into(),
      radius: c.radius.into(),
    }
  }
}

impl Project<ScreenCircle> for ScreenPosition {
  type Output = Self;

  fn project(self, target: ScreenCircle) -> Self {
    let c: Circle = target.into();
    self.0.project(c).into()
  }
}

#[derive(Debug, Clone, Copy)]
pub enum ScreenCircleIntersect {
  TwoPoints(ScreenPosition, ScreenPosition),
  OnePoint(ScreenPosition),
  None,
}

impl ScreenCircleIntersect {
  pub fn reverse(self) -> Self {
    match self {
      ScreenCircleIntersect::TwoPoints(p1, p2) => ScreenCircleIntersect::TwoPoints(p2, p1),
      _ => self,
    }
  }
}

impl From<CircleIntersect> for ScreenCircleIntersect {
  fn from(itsct: CircleIntersect) -> Self {
    match itsct {
      CircleIntersect::TwoPoints(p1, p2) => ScreenCircleIntersect::TwoPoints(p1.into(), p2.into()),
      CircleIntersect::OnePoint(p) => ScreenCircleIntersect::OnePoint(p.into()),
      CircleIntersect::None => ScreenCircleIntersect::None,
    }
  }
}

impl Intersect<ScreenCircle> for ScreenCircle {
  type Output = ScreenCircleIntersect;

  fn intersect(self, other: Self) -> Self::Output {
    let c1: Circle = self.into();
    let c2: Circle = other.into();
    c1.intersect(c2).into()
  }
}

impl Intersect<ScreenLine> for ScreenCircle {
  type Output = ScreenCircleIntersect;

  fn intersect(self, other: ScreenLine) -> Self::Output {
    let c: Circle = self.into();
    let l: Line = other.into();
    c.intersect(l).into()
  }
}

impl Intersect<ScreenCircle> for ScreenLine {
  type Output = ScreenCircleIntersect;

  fn intersect(self, other: ScreenCircle) -> Self::Output {
    let l: Line = self.into();
    let c: Circle = other.into();
    c.intersect(l).into()
  }
}

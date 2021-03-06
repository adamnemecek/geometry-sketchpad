use specs::prelude::*;
use core_lib::{
  math::*,
  components::{screen_shapes::*, styles::*},
  resources::*,
  events::*,
};
use crate::events::*;

pub struct SenderSystem {
  pub sender: std::sync::mpsc::Sender<RenderUpdateEvent>,
  scrn_point_update_reader: Option<ReaderId<ComponentEvent>>,
  point_style_update_reader: Option<ReaderId<ComponentEvent>>,
  scrn_line_update_reader: Option<ReaderId<ComponentEvent>>,
  line_style_update_reader: Option<ReaderId<ComponentEvent>>,
  scrn_circle_update_reader: Option<ReaderId<ComponentEvent>>,
  circle_style_update_reader: Option<ReaderId<ComponentEvent>>,
  scrn_rect_update_reader: Option<ReaderId<ComponentEvent>>,
  rect_style_update_reader: Option<ReaderId<ComponentEvent>>,
  marker_event_reader: Option<MarkerEventReader>,
}

impl SenderSystem {
  pub fn new(sender: std::sync::mpsc::Sender<RenderUpdateEvent>) -> Self {
    Self {
      sender,
      scrn_point_update_reader: None,
      point_style_update_reader: None,
      scrn_line_update_reader: None,
      line_style_update_reader: None,
      scrn_circle_update_reader: None,
      circle_style_update_reader: None,
      scrn_rect_update_reader: None,
      rect_style_update_reader: None,
      marker_event_reader: None,
    }
  }
}

impl<'a> System<'a> for SenderSystem {
  type SystemData = (
    Entities<'a>,
    Read<'a, Viewport>,
    Read<'a, MarkerEventChannel>,
    ReadStorage<'a, ScreenPoint>,
    ReadStorage<'a, PointStyle>,
    ReadStorage<'a, ScreenLine>,
    ReadStorage<'a, LineStyle>,
    ReadStorage<'a, ScreenCircle>,
    ReadStorage<'a, CircleStyle>,
    ReadStorage<'a, ScreenRectangle>,
    ReadStorage<'a, RectangleStyle>,
  );

  fn setup(&mut self, world: &mut World) {
    Self::SystemData::setup(world);
    self.scrn_point_update_reader = Some(WriteStorage::<ScreenPoint>::fetch(&world).register_reader());
    self.point_style_update_reader = Some(WriteStorage::<PointStyle>::fetch(&world).register_reader());
    self.scrn_line_update_reader = Some(WriteStorage::<ScreenLine>::fetch(&world).register_reader());
    self.line_style_update_reader = Some(WriteStorage::<LineStyle>::fetch(&world).register_reader());
    self.scrn_circle_update_reader = Some(WriteStorage::<ScreenCircle>::fetch(&world).register_reader());
    self.circle_style_update_reader = Some(WriteStorage::<CircleStyle>::fetch(&world).register_reader());
    self.scrn_rect_update_reader = Some(WriteStorage::<ScreenRectangle>::fetch(&world).register_reader());
    self.rect_style_update_reader = Some(WriteStorage::<RectangleStyle>::fetch(&world).register_reader());
    self.marker_event_reader = Some(world.fetch_mut::<MarkerEventChannel>().register_reader());
  }

  fn run(&mut self, (
    entities,
    viewport,
    marker_event_channel,
    scrn_points,
    point_styles,
    scrn_lines,
    line_styles,
    scrn_circles,
    circle_styles,
    scrn_rects,
    rect_styles,
  ): Self::SystemData) {

    // First deal with geometry update
    let mut inserted_points = BitSet::new();
    let mut modified_points = BitSet::new();
    let mut modified_point_styles = BitSet::new();
    let mut inserted_lines = BitSet::new();
    let mut modified_lines = BitSet::new();
    let mut modified_line_styles = BitSet::new();
    let mut inserted_circles = BitSet::new();
    let mut modified_circles = BitSet::new();
    let mut modified_circle_styles = BitSet::new();
    let mut inserted_rects = BitSet::new();
    let mut modified_rects = BitSet::new();
    let mut modified_rect_styles = BitSet::new();
    let mut removed : BitSet = BitSet::new();

    // Screen point updates
    if let Some(reader) = &mut self.scrn_point_update_reader {
      for event in scrn_points.channel().read(reader) {
        match event {
          ComponentEvent::Inserted(id) => { inserted_points.add(*id); },
          ComponentEvent::Modified(id) => { modified_points.add(*id); },
          ComponentEvent::Removed(id) => { removed.add(*id); },
        }
      }
    }

    if let Some(reader) = &mut self.point_style_update_reader {
      for event in point_styles.channel().read(reader) {
        match event {
          ComponentEvent::Modified(id) => { modified_point_styles.add(*id); },
          _ => (),
        }
      }
    }

    if let Some(reader) = &mut self.scrn_line_update_reader {
      for event in scrn_lines.channel().read(reader) {
        match event {
          ComponentEvent::Inserted(id) => { inserted_lines.add(*id); },
          ComponentEvent::Modified(id) => { modified_lines.add(*id); },
          ComponentEvent::Removed(id) => { removed.add(*id); },
        }
      }
    }

    if let Some(reader) = &mut self.line_style_update_reader {
      for event in line_styles.channel().read(reader) {
        match event {
          ComponentEvent::Modified(id) => { modified_line_styles.add(*id); },
          _ => (),
        }
      }
    }

    if let Some(reader) = &mut self.scrn_circle_update_reader {
      for event in scrn_circles.channel().read(reader) {
        match event {
          ComponentEvent::Inserted(id) => { inserted_circles.add(*id); },
          ComponentEvent::Modified(id) => { modified_circles.add(*id); },
          ComponentEvent::Removed(id) => { removed.add(*id); },
        }
      }
    }

    if let Some(reader) = &mut self.circle_style_update_reader {
      for event in circle_styles.channel().read(reader) {
        match event {
          ComponentEvent::Modified(id) => { modified_circle_styles.add(*id); },
          _ => (),
        }
      }
    }

    if let Some(reader) = &mut self.scrn_rect_update_reader {
      for event in scrn_rects.channel().read(reader) {
        match event {
          ComponentEvent::Inserted(id) => { inserted_rects.add(*id); },
          ComponentEvent::Modified(id) => { modified_rects.add(*id); },
          ComponentEvent::Removed(id) => { removed.add(*id); },
        }
      }
    }

    if let Some(reader) = &mut self.rect_style_update_reader {
      for event in rect_styles.channel().read(reader) {
        match event {
          ComponentEvent::Modified(id) => { modified_rect_styles.add(*id); },
          _ => (),
        }
      }
    }

    // Do all the insert
    for (ent, scrn_point, point_style, _) in (&entities, &scrn_points, &point_styles, &inserted_points).join() {
      if let Err(err) = self.sender.send(RenderUpdateEvent::InsertedPoint(ent, *scrn_point, *point_style)) { panic!(err) }
    }
    for (ent, scrn_line, line_style, _) in (&entities, &scrn_lines, &line_styles, &inserted_lines).join() {
      if let Some((from, to)) = scrn_line.intersect(viewport.screen_aabb()) {
        if let Err(err) = self.sender.send(RenderUpdateEvent::InsertedLine(ent, ScreenLine { from, to, line_type: LineType::Segment }, *line_style)) { panic!(err) }
      } else {
        if let Err(err) = self.sender.send(RenderUpdateEvent::InsertedLine(ent, *scrn_line, *line_style)) { panic!(err) }
      }
    }
    for (ent, scrn_circle, circle_style, _) in (&entities, &scrn_circles, &circle_styles, &inserted_circles).join() {
      if let Err(err) = self.sender.send(RenderUpdateEvent::InsertedCircle(ent, *scrn_circle, *circle_style)) { panic!(err) }
    }
    for (ent, scrn_rect, rect_style, _) in (&entities, &scrn_rects, &rect_styles, &inserted_rects).join() {
      if let Err(err) = self.sender.send(RenderUpdateEvent::InsertedRectangle(ent, *scrn_rect, *rect_style)) { panic!(err) }
    }

    // Do all the modify
    for (ent, scrn_point, _) in (&entities, &scrn_points, &modified_points).join() {
      if let Err(err) = self.sender.send(RenderUpdateEvent::UpdatedPoint(ent, *scrn_point)) { panic!(err) }
    }
    for (ent, point_style, _) in (&entities, &point_styles, &modified_point_styles).join() {
      if let Err(err) = self.sender.send(RenderUpdateEvent::UpdatedPointStyle(ent, *point_style)) { panic!(err) }
    }
    for (ent, scrn_line, _) in (&entities, &scrn_lines, &modified_lines).join() {
      if let Some((from, to)) = scrn_line.intersect(viewport.screen_aabb()) {
        if let Err(err) = self.sender.send(RenderUpdateEvent::UpdatedLine(ent, ScreenLine { from, to, line_type: LineType::Segment })) { panic!(err) }
      } else {
        if let Err(err) = self.sender.send(RenderUpdateEvent::UpdatedLine(ent, *scrn_line)) { panic!(err) }
      }
    }
    for (ent, line_style, _) in (&entities, &line_styles, &modified_line_styles).join() {
      if let Err(err) = self.sender.send(RenderUpdateEvent::UpdatedLineStyle(ent, *line_style)) { panic!(err) }
    }
    for (ent, scrn_circle, _) in (&entities, &scrn_circles, &modified_circles).join() {
      if let Err(err) = self.sender.send(RenderUpdateEvent::UpdatedCircle(ent, *scrn_circle)) { panic!(err) }
    }
    for (ent, circle_style, _) in (&entities, &circle_styles, &modified_circle_styles).join() {
      if let Err(err) = self.sender.send(RenderUpdateEvent::UpdatedCircleStyle(ent, *circle_style)) { panic!(err) }
    }
    for (ent, scrn_rect, _) in (&entities, &scrn_rects, &modified_rects).join() {
      if let Err(err) = self.sender.send(RenderUpdateEvent::UpdatedRectangle(ent, *scrn_rect)) { panic!(err) }
    }
    for (ent, rect_style, _) in (&entities, &rect_styles, &modified_rect_styles).join() {
      if let Err(err) = self.sender.send(RenderUpdateEvent::UpdatedRectangleStyle(ent, *rect_style)) { panic!(err) }
    }

    // Do all the removals
    for (ent, _) in (&entities, &removed).join() {
      if let Err(err) = self.sender.send(RenderUpdateEvent::RemovedEntity(ent)) { panic!(err) }
    }

    // Then deal with select update
    if let Some(reader) = &mut self.marker_event_reader {
      for event in marker_event_channel.read(reader) {
        match event {
          MarkerEvent::Deselect(ent) => {
            if let Err(err) = self.sender.send(RenderUpdateEvent::DeselectedEntity(*ent)) { panic!(err) }
          },
          MarkerEvent::Select(ent) => {
            if let Err(err) = self.sender.send(RenderUpdateEvent::SelectedEntity(*ent)) { panic!(err) }
          },
          _ => (),
        }
      }
    }
  }
}
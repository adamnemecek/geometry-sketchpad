use shrev::*;

#[derive(Clone, Copy)]
pub enum HistoryEvent {
  Redo,
  Undo,
  Clear,
}

pub type HistoryEventChannel = EventChannel<HistoryEvent>;

pub type HistoryEventReader = ReaderId<HistoryEvent>;

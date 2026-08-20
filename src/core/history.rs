use crate::core::timeline::Timeline;

/// Bounded Undo / Redo history manager for timeline state.
#[derive(Clone, Debug, Default)]
pub struct TimelineHistory {
    undo_stack: Vec<Timeline>,
    redo_stack: Vec<Timeline>,
    max_history: usize,
}

impl TimelineHistory {
    pub fn new(max_history: usize) -> Self {
        Self {
            undo_stack: Vec::with_capacity(max_history),
            redo_stack: Vec::with_capacity(max_history),
            max_history,
        }
    }

    /// Record a state snapshot before performing a mutating action.
    pub fn push_snapshot(&mut self, timeline: &Timeline) {
        if self.undo_stack.len() >= self.max_history {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(timeline.clone());
        self.redo_stack.clear(); // Clear redo stack on new action
    }

    /// Drop the most recent snapshot without touching the redo stack — for a
    /// batch action that snapshotted up front and then turned out to be a no-op.
    pub fn discard_last_snapshot(&mut self) {
        self.undo_stack.pop();
    }

    /// Undo to the previous timeline state.
    pub fn undo(&mut self, current_timeline: &Timeline) -> Option<Timeline> {
        if let Some(prev_state) = self.undo_stack.pop() {
            self.redo_stack.push(current_timeline.clone());
            Some(prev_state)
        } else {
            None
        }
    }

    /// Redo to the next timeline state.
    pub fn redo(&mut self, current_timeline: &Timeline) -> Option<Timeline> {
        if let Some(next_state) = self.redo_stack.pop() {
            self.undo_stack.push(current_timeline.clone());
            Some(next_state)
        } else {
            None
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

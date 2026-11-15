//! Synchronized lyric clock matching playhead position to lyric lines.

use super::model::ParsedLyrics;

#[derive(Clone, Debug)]
pub struct LyricClockState {
    pub active_index: Option<usize>,
    pub line_progress: f32,
    pub next_line_index: Option<usize>,
}

pub struct LyricClock {
    pub offset_ms: i64,
}

impl Default for LyricClock {
    fn default() -> Self {
        Self::new()
    }
}

impl LyricClock {
    pub fn new() -> Self {
        Self { offset_ms: 0 }
    }

    pub fn set_offset_ms(&mut self, offset: i64) {
        self.offset_ms = offset;
    }

    /// Evaluates which lyric line is currently active given playhead position.
    pub fn update(&self, lyrics: &ParsedLyrics, playhead_ms: u64) -> LyricClockState {
        if !lyrics.is_synced || lyrics.lines.is_empty() {
            return LyricClockState {
                active_index: None,
                line_progress: 0.0,
                next_line_index: None,
            };
        }

        let effective_pos = (playhead_ms as i64 + self.offset_ms).max(0) as u64;

        let mut active_idx = None;
        for (i, line) in lyrics.lines.iter().enumerate() {
            if effective_pos >= line.start_ms && effective_pos < line.end_ms {
                active_idx = Some(i);
                break;
            }
        }

        // If between lines, find latest line that already started
        if active_idx.is_none() {
            for (i, line) in lyrics.lines.iter().enumerate().rev() {
                if effective_pos >= line.start_ms {
                    active_idx = Some(i);
                    break;
                }
            }
        }

        let line_progress = if let Some(idx) = active_idx {
            let line = &lyrics.lines[idx];
            if line.end_ms > line.start_ms {
                let p = (effective_pos.saturating_sub(line.start_ms) as f32)
                    / ((line.end_ms - line.start_ms) as f32);
                p.clamp(0.0, 1.0)
            } else {
                1.0
            }
        } else {
            0.0
        };

        let next_line_index = active_idx.map(|idx| idx + 1).filter(|&idx| idx < lyrics.lines.len());

        LyricClockState {
            active_index: active_idx,
            line_progress,
            next_line_index,
        }
    }
}

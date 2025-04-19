use crate::result::ResultStatus;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct DetailedStats {
    pub start_time: Option<Instant>,
    pub pause_time: Option<Instant>,
    pub total_paused_time: Duration,
    pub total_combos: usize,
    pub checked: usize,
    pub hits: usize,
    pub free: usize,
    pub error: usize,
    pub invalid: usize,
    pub banned: usize,
    pub retries: usize,
}

#[derive(Debug, Clone)]
pub struct Stats {
    start_time: Option<Instant>,
    pause_time: Option<Instant>,
    total_paused_time: Duration,
    total_combos: Arc<parking_lot::RwLock<usize>>,
    checked: Arc<parking_lot::RwLock<usize>>,
    result_counts: Arc<parking_lot::RwLock<HashMap<ResultStatus, usize>>>,
}

impl Default for Stats {
    fn default() -> Self {
        let mut result_counts = HashMap::new();

        result_counts.insert(ResultStatus::Hit, 0);
        result_counts.insert(ResultStatus::Free, 0);
        result_counts.insert(ResultStatus::Error, 0);
        result_counts.insert(ResultStatus::Invalid, 0);
        result_counts.insert(ResultStatus::Banned, 0);
        result_counts.insert(ResultStatus::Retry, 0);

        Self {
            start_time: None,
            pause_time: None,
            total_paused_time: Duration::from_secs(0),
            total_combos: Arc::new(parking_lot::RwLock::new(0)),
            checked: Arc::new(parking_lot::RwLock::new(0)),
            result_counts: Arc::new(parking_lot::RwLock::new(result_counts)),
        }
    }
}

impl Stats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start(&mut self) {
        if self.start_time.is_none() {
            self.start_time = Some(Instant::now());
        } else if let Some(pause_time) = self.pause_time.take() {
            self.total_paused_time += pause_time.elapsed();
        }
    }

    pub fn pause(&mut self) {
        if self.pause_time.is_none() {
            self.pause_time = Some(Instant::now());
        }
    }

    pub fn reset(&mut self) {
        self.start_time = None;
        self.pause_time = None;
        self.total_paused_time = Duration::from_secs(0);
        *self.checked.write() = 0;
        *self.result_counts.write() = HashMap::new();
    }

    pub fn set_total(&self, total: usize) {
        *self.total_combos.write() = total;
    }

    pub fn increment_checked(&self) {
        let mut checked = self.checked.write();
        *checked += 1;
    }

    pub fn increment_result(&self, result_type: ResultStatus) {
        let mut counts = self.result_counts.write();
        *counts.entry(result_type).or_insert(0) += 1;
    }

    pub fn elapsed(&self) -> Duration {
        match self.start_time {
            Some(start) => {
                let raw_elapsed = if let Some(pause) = self.pause_time {
                    pause.duration_since(start)
                } else {
                    start.elapsed()
                };

                if raw_elapsed > self.total_paused_time {
                    raw_elapsed - self.total_paused_time
                } else {
                    Duration::from_secs(0)
                }
            }
            None => Duration::from_secs(0),
        }
    }

    pub fn total(&self) -> usize {
        *self.total_combos.read()
    }

    pub fn checked(&self) -> usize {
        *self.checked.read()
    }

    pub fn remaining(&self) -> usize {
        let total = self.total();
        let checked = self.checked();

        if total > checked { total - checked } else { 0 }
    }

    pub fn progress(&self) -> f64 {
        let total = self.total();

        if total == 0 {
            return 0.0;
        }

        (self.checked() as f64 / total as f64) * 100.0
    }

    pub fn cpm(&self) -> u64 {
        let elapsed = self.elapsed();

        if elapsed.as_secs() == 0 {
            return 0;
        }

        let checked = self.checked() as u64;
        let minutes = (elapsed.as_secs() as f64 / 60.0).max(1.0 / 60.0);

        (checked as f64 / minutes) as u64
    }

    pub fn eta(&self) -> Duration {
        let cpm = self.cpm();

        if cpm == 0 {
            return Duration::from_secs(0);
        }

        let remaining = self.remaining() as u64;
        let minutes = remaining as f64 / cpm as f64;

        Duration::from_secs((minutes * 60.0) as u64)
    }

    pub fn get_result_count(&self, result_type: ResultStatus) -> usize {
        *self.result_counts.read().get(&result_type).unwrap_or(&0)
    }

    pub fn hits(&self) -> usize {
        self.get_result_count(ResultStatus::Hit)
    }

    pub fn free(&self) -> usize {
        self.get_result_count(ResultStatus::Free)
    }

    pub fn errors(&self) -> usize {
        self.get_result_count(ResultStatus::Error)
    }

    pub fn invalid(&self) -> usize {
        self.get_result_count(ResultStatus::Invalid)
    }

    pub fn banned(&self) -> usize {
        self.get_result_count(ResultStatus::Banned)
    }

    pub fn retries(&self) -> usize {
        self.get_result_count(ResultStatus::Retry)
    }

    pub fn format_duration(duration: Duration) -> String {
        let total_seconds = duration.as_secs();
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    }

    pub fn get_detailed_stats(&self) -> DetailedStats {
        let result_counts = self.result_counts.read();

        DetailedStats {
            start_time: self.start_time,
            pause_time: self.pause_time,
            total_paused_time: self.total_paused_time,
            total_combos: *self.total_combos.read(),
            checked: *self.checked.read(),
            hits: *result_counts.get(&ResultStatus::Hit).unwrap_or(&0),
            free: *result_counts.get(&ResultStatus::Free).unwrap_or(&0),
            error: *result_counts.get(&ResultStatus::Error).unwrap_or(&0),
            invalid: *result_counts.get(&ResultStatus::Invalid).unwrap_or(&0),
            banned: *result_counts.get(&ResultStatus::Banned).unwrap_or(&0),
            retries: *result_counts.get(&ResultStatus::Retry).unwrap_or(&0),
        }
    }
}

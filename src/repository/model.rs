use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::ops::Add;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum TimeKind {
    Productive,
    Pause,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimeSegment {
    pub start: DateTime<Utc>,
    pub end: Option<DateTime<Utc>>,
    pub kind: TimeKind,
}

impl TimeSegment {
    pub fn finish(&mut self) {
        if self.end.is_none() {
            self.end = Some(Utc::now());
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ProjectState {
    Working,
    Paused,
    Done,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkRecord {
    pub id: String,
    pub name: String,
    pub start: DateTime<Utc>,
    pub end: Option<DateTime<Utc>>,
    pub state: ProjectState,
    pub segments: Vec<TimeSegment>,
}

impl WorkRecord {
    pub fn calculate_duration(&self) -> chrono::Duration {
        self.segments
            .iter()
            .filter(|segment| segment.kind == TimeKind::Productive)
            .map(|segment| {
                if let Some(end) = segment.end {
                    end.signed_duration_since(segment.start)
                } else {
                    Utc::now().signed_duration_since(segment.start)
                }
            })
            .reduce(Add::add)
            .expect("There should always be at least one segment to calculate a Duration")
    }
}

impl Display for WorkRecord {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let icon = match self.state {
            ProjectState::Working => "♪",
            ProjectState::Paused => "𝄽",
            ProjectState::Done => "✓",
        };

        let end_description = if let Some(end) = self.end {
            format!(
                " - {}",
                end.with_timezone(chrono::Local::now().offset())
                    .format("%H:%M")
            )
        } else {
            "".to_string()
        };

        let duration = self.calculate_duration();

        let result = format!(
            "{} {}: {}{} (time spent: {:02}:{:02}:{:02})",
            icon,
            self.name,
            self.start
                .with_timezone(chrono::Local::now().offset())
                .format("%Y-%m-%d %H:%M"),
            end_description,
            duration.num_hours(),
            duration.num_minutes() % 60,
            duration.num_seconds() % 60
        );

        f.write_str(&result)
    }
}

use chrono::{DateTime, Local};
use std::time::Duration;

#[derive(Debug)]
pub enum Segment {
    Time(Duration, Duration),
    Size(u64, u64),
}

impl Segment {
    pub fn from_seg(mut seg: Segment) -> Self {
        match &mut seg {
            Segment::Time(_, old) => {
                *old = Duration::ZERO;
                seg
            }
            Segment::Size(_, old) => {
                *old = 0;
                seg
            }
        }
    }

    pub fn needed(&mut self, actual_size: u64, actual_time: Duration) -> bool {
        match self {
            Segment::Time(expected, start_time) if *expected <= actual_time - *start_time => {
                *start_time = actual_time;
                true
            }
            Segment::Size(expected, _) if *expected <= actual_size => true,
            Segment::Time(_, _) => false,
            Segment::Size(_, _) => false,
        }
    }

    pub fn needed_delta(&mut self, size: u64, delta: Duration) -> bool {
        match self {
            Segment::Time(expected, previous) if *expected <= *previous + delta => {
                *previous = delta;
                true
            }
            Segment::Size(expected, previous) if *expected <= *previous + size => {
                *previous = 0;
                true
            }
            Segment::Time(_, previous) => {
                *previous += delta;
                false
            }
            Segment::Size(_, previous) => {
                *previous += size;
                false
            }
        }
    }
}

pub fn format_filename(file_name: &str) -> String {
    let local: DateTime<Local> = Local::now();
    // let time_str = local.format("%Y-%m-%dT%H_%M_%S");
    let time_str = local.format(file_name);
    // format!("{file_name}{time_str}")
    time_str.to_string()
}

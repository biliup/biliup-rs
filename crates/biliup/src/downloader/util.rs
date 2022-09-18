use chrono::{DateTime, Local};
use std::ops::AddAssign;
use std::time::Duration;

#[derive(Debug)]
pub enum Segment {
    Time(Duration, Duration),
    Size(u64, u64),
    Never,
}
#[derive(Debug)]
pub struct Segmentable {
    time: Time,
    size: Size,
}
#[derive(Debug)]
struct Time {
    expected: Duration,
    start: Duration,
    current: Duration,
}
#[derive(Debug)]
struct Size {
    expected: u64,
    current: u64,
}

impl Segmentable {
    pub fn new(expected_time: Duration, expected_size: u64) -> Self {
        Self {
            time: Time {
                expected: expected_time,
                start: Duration::ZERO,
                current: Duration::ZERO,
            },
            size: Size {
                expected: expected_size,
                current: 0,
            },
        }
    }

    pub fn needed(&self) -> bool {
        (self.time.current - self.time.start) >= self.time.expected
            || self.size.current > self.size.expected
    }

    pub fn increase_time(&mut self, number: Duration) {
        self.time.current += number
    }

    pub fn set_time_position(&mut self, number: Duration) {
        self.time.current = number
    }

    pub fn set_start_time(&mut self, number: Duration) {
        self.time.start = number
    }

    pub fn increase_size(&mut self, number: u64) {
        self.size.current += number
    }

    pub fn set_size_position(&mut self, number: u64) {
        self.size.current = number
    }

    pub fn reset(&mut self) {
        self.size.current = 0;
        self.time.current = Duration::ZERO;
    }
}

impl Default for Segmentable {
    fn default() -> Self {
        Segmentable {
            time: Time {
                expected: Duration::MAX,
                start: Duration::ZERO,
                current: Duration::ZERO,
            },
            size: Size {
                expected: u64::MAX,
                current: 0,
            },
        }
    }
}

// impl Segment {
//     pub fn increase_size(&mut self, size: u64, delta: Duration) {
//         Duration::saturating_add()
//         match self {
//             Segment::Time(_, old) => {
//                 *old = Duration::ZERO;
//                 // seg
//             }
//             Segment::Size(_, old) => {
//                 *old = 0;
//                 // seg
//             }
//             Segment::Never => {}
//         }
//     }
//
//     pub fn increase_time(&mut self, size: u64, delta: Duration) {
//         match self {
//             Segment::Time(_, old) => {
//                 *old = Duration::ZERO;
//                 // seg
//             }
//             Segment::Size(_, old) => {
//                 *old = 0;
//                 // seg
//             }
//             Segment::Never => {}
//         }
//     }
//
//     pub fn reset(&mut self) {
//         match self {
//             Segment::Time(_, old) => {
//                 *old = Duration::ZERO;
//                 // seg
//             }
//             Segment::Size(_, old) => {
//                 *old = 0;
//                 // seg
//             }
//             Segment::Never => {}
//         }
//     }
//
//     pub fn needed(&mut self, actual_size: u64, actual_time: Duration) -> bool {
//         match self {
//             Segment::Time(expected, start_time) if *expected <= actual_time - *start_time => {
//                 *start_time = actual_time;
//                 true
//             }
//             Segment::Size(expected, _) if *expected <= actual_size => true,
//             Segment::Time(_, _) => false,
//             Segment::Size(_, _) => false,
//             Segment::Never => {false}
//         }
//     }
//
//     pub fn needed_delta(&mut self, size: u64, delta: Duration) -> bool {
//         match self {
//             Segment::Time(expected, previous) if *expected <= *previous + delta => {
//                 *previous = delta;
//                 true
//             }
//             Segment::Size(expected, previous) if *expected <= *previous + size => {
//                 *previous = 0;
//                 true
//             }
//             Segment::Time(_, previous) => {
//                 *previous += delta;
//                 false
//             }
//             Segment::Size(_, previous) => {
//                 *previous += size;
//                 false
//             }
//             Segment::Never => {false}
//         }
//     }
// }

pub fn format_filename(file_name: &str) -> String {
    let local: DateTime<Local> = Local::now();
    // let time_str = local.format("%Y-%m-%dT%H_%M_%S");
    let time_str = local.format(file_name);
    // format!("{file_name}{time_str}")
    time_str.to_string()
}

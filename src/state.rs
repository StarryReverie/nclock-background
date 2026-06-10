use std::time::Instant;

use time::{Month, PrimitiveDateTime};

#[derive(Debug, Clone)]
pub struct AppState {
    clock: ClockAnimation,
    current_instant: Instant,
}

impl AppState {
    pub fn new(initial_time: PrimitiveDateTime, initial_instant: Instant) -> Self {
        Self {
            clock: ClockAnimation::new(initial_time, initial_instant),
            current_instant: initial_instant,
        }
    }

    pub fn refresh_current_instant(&mut self) {
        self.current_instant = Instant::now();
    }

    pub fn angles(&self) -> ClockAngles {
        self.clock.angles_at(self.current_instant)
    }
}

#[derive(Debug, Clone)]
pub struct ClockAnimation {
    initial_time: PrimitiveDateTime,
    initial_instant: Instant,
}

impl ClockAnimation {
    pub fn new(initial_time: PrimitiveDateTime, initial_instant: Instant) -> Self {
        Self {
            initial_time,
            initial_instant,
        }
    }

    pub fn current_time(&self, current_instant: Instant) -> PrimitiveDateTime {
        let duration = current_instant.duration_since(self.initial_instant);
        self.initial_time + duration
    }

    pub fn angles_at(&self, current_instant: Instant) -> ClockAngles {
        let current_time = self.current_time(current_instant);
        ClockAngles {
            angles: [
                Self::months_angle_at(&current_time),
                Self::days_angle_at(&current_time),
                Self::hour_angle_at(&current_time),
                Self::minute_angle_at(&current_time),
                Self::second_angle_at(&current_time),
            ],
        }
    }

    fn months_angle_at(current_time: &PrimitiveDateTime) -> f64 {
        let truncated = current_time
            .truncate_to_day()
            .replace_month(Month::January)
            .unwrap()
            .replace_day(1)
            .unwrap();
        Self::calc_animation_angle(
            &truncated,
            current_time,
            24 * 60 * 60 * (time::util::days_in_year(current_time.year()) as u64),
        )
    }

    fn days_angle_at(current_time: &PrimitiveDateTime) -> f64 {
        let truncated = current_time.truncate_to_day().replace_day(1).unwrap();
        Self::calc_animation_angle(
            &truncated,
            current_time,
            24 * 60 * 60 * (current_time.month().length(current_time.year()) as u64),
        )
    }

    fn hour_angle_at(current_time: &PrimitiveDateTime) -> f64 {
        let truncated = current_time.truncate_to_day();
        Self::calc_animation_angle(&truncated, current_time, 24 * 60 * 60)
    }

    fn minute_angle_at(current_time: &PrimitiveDateTime) -> f64 {
        let truncated = current_time.truncate_to_hour();
        Self::calc_animation_angle(&truncated, current_time, 60 * 60)
    }

    fn second_angle_at(current_time: &PrimitiveDateTime) -> f64 {
        let truncated = current_time.truncate_to_minute();
        Self::calc_animation_angle(&truncated, current_time, 60)
    }

    fn calc_animation_angle(
        start: &PrimitiveDateTime,
        end: &PrimitiveDateTime,
        length: u64,
    ) -> f64 {
        let fraction = (*end - *start).as_seconds_f64() / (length as f64);
        fraction * 2.0 * std::f64::consts::PI
    }
}

#[derive(Debug, Clone)]
pub struct ClockAngles {
    pub angles: [f64; 5],
}

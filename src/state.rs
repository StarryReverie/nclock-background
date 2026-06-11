use std::time::Instant;

use time::{Month, PrimitiveDateTime, UtcOffset};

pub const POINTER_RESET_ANIMATION_DURATION: f64 = 0.75;
pub const INTRO_ANIMATION_WAIT_DURATION: f64 = 1.5;
pub const INTRO_ANIMATION_EXPANSION_DURATION: f64 = 0.75;

#[derive(Debug, Clone)]
pub struct AppState {
    clock: ClockAnimation,
    current_instant: Instant,
    utc_offset: UtcOffset,
}

impl AppState {
    pub fn new(
        initial_time: PrimitiveDateTime,
        initial_instant: Instant,
        utc_offset: UtcOffset,
    ) -> Self {
        Self {
            clock: ClockAnimation::new(initial_time, initial_instant),
            current_instant: initial_instant,
            utc_offset,
        }
    }

    pub fn initial_instant(&self) -> Instant {
        self.clock.initial_instant()
    }

    pub fn refresh_current_instant(&mut self) {
        self.current_instant = Instant::now();
    }

    pub fn angles(&self) -> ClockAngles {
        self.clock.angles_at(self.current_instant)
    }

    pub fn labels(&self) -> ClockLabels {
        self.clock.labels_at(self.current_instant)
    }

    pub fn footer_text(&self) -> String {
        let current_time = self.clock.current_time(self.current_instant);

        let tz_name = iana_time_zone::get_timezone().unwrap_or_else(|_| "Unknown".to_string());

        let offset_hours = self.utc_offset.whole_hours();
        let offset_sign = if offset_hours >= 0 { "+" } else { "-" };
        let offset_str = format!("{} (UTC{}{})", tz_name, offset_sign, offset_hours.abs());

        let date_str = current_time
            .format(
                &time::format_description::parse("[year]-[month repr:numerical]-[day]").unwrap(),
            )
            .unwrap();
        let weekday = current_time.weekday().to_string();
        let time_str = current_time
            .format(
                &time::format_description::parse("[hour repr:12]:[minute]:[second] [period]")
                    .unwrap(),
            )
            .unwrap();

        format!(
            "Night Clock - {} - {} {} {}",
            offset_str, date_str, weekday, time_str
        )
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

    pub fn initial_instant(&self) -> Instant {
        self.initial_instant
    }

    pub fn current_time(&self, current_instant: Instant) -> PrimitiveDateTime {
        let duration = current_instant.duration_since(self.initial_instant);
        self.initial_time + duration
    }

    pub fn angles_at(&self, current_instant: Instant) -> ClockAngles {
        let current_time = self.current_time(current_instant);
        let intro = self.calc_intro_factor(current_instant);
        ClockAngles {
            angles: [
                Self::months_angle_at(&current_time) * intro,
                Self::days_angle_at(&current_time) * intro,
                Self::hour_angle_at(&current_time) * intro,
                Self::minute_angle_at(&current_time) * intro,
                Self::second_angle_at(&current_time) * intro,
            ],
        }
    }

    pub fn labels_at(&self, current_instant: Instant) -> ClockLabels {
        let current_time = self.current_time(current_instant);
        ClockLabels {
            labels: [
                current_time.month().to_string(),
                format_day_ordinal(current_time.day()),
                format_hours_12(current_time.hour()),
                format!("{} minutes", current_time.minute()),
                format!("{} seconds", current_time.second()),
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
        let block_start = if current_time.hour() < 12 {
            current_time.truncate_to_day()
        } else {
            current_time.truncate_to_day().replace_hour(12).unwrap()
        };
        Self::calc_animation_angle(&block_start, current_time, 12 * 60 * 60)
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
        let seconds = (*end - *start).as_seconds_f64();

        let fraction = if seconds < POINTER_RESET_ANIMATION_DURATION {
            (1.0 - seconds / POINTER_RESET_ANIMATION_DURATION).powi(2)
        } else {
            seconds / (length as f64)
        };

        fraction * 2.0 * std::f64::consts::PI
    }

    fn calc_intro_factor(&self, current_instant: Instant) -> f64 {
        let elapsed = current_instant
            .duration_since(self.initial_instant)
            .as_secs_f64();
        if elapsed >= INTRO_ANIMATION_WAIT_DURATION + INTRO_ANIMATION_EXPANSION_DURATION {
            1.0
        } else if elapsed >= INTRO_ANIMATION_WAIT_DURATION {
            let elapsed = elapsed - INTRO_ANIMATION_WAIT_DURATION;
            1.0 - (-10.0 * elapsed / INTRO_ANIMATION_EXPANSION_DURATION).exp2()
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClockAngles {
    pub angles: [f64; 5],
}

#[derive(Debug, Clone)]
pub struct ClockLabels {
    pub labels: [String; 5],
}

fn format_day_ordinal(day: u8) -> String {
    let suffix = match day % 10 {
        1 if day % 100 != 11 => "st",
        2 if day % 100 != 12 => "nd",
        3 if day % 100 != 13 => "rd",
        _ => "th",
    };
    format!("{}{}", day, suffix)
}

fn format_hours_12(hour: u8) -> String {
    let h12 = hour % 12;
    let h12 = if h12 == 0 { 12 } else { h12 };
    let period = if hour < 12 { "AM" } else { "PM" };
    format!("{} hours {}", h12, period)
}

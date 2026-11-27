//! Seasonal visitors: which random objects are in season on a given date.
//!
//! An idea from Kirk Baucom's *weatherspect*: Santa near Christmas, a ghost
//! near Halloween. While a seasonal visitor is in season it is picked for
//! about one random spawn in three; out of season it never shows up (unless
//! forced with `FISHIES_OBJECT`).

use std::sync::OnceLock;

use super::random::ORDINARY;

/// A calendar date.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Date {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl Date {
    pub const fn new(year: i32, month: u32, day: u32) -> Self {
        Date { year, month, day }
    }

    /// Days since 1970-01-01 (proleptic Gregorian calendar).
    fn days(self) -> i64 {
        days_from_civil(self.year, self.month, self.day)
    }
}

/// When a seasonal visitor is in season.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Window {
    /// From `(month, day)` to `(month, day)`, both included; it may wrap
    /// around the new year.
    Fixed { from: (u32, u32), to: (u32, u32) },
    /// Within this many days of Easter Sunday.
    Easter { days: i64 },
}

impl Window {
    pub fn contains(self, d: Date) -> bool {
        match self {
            Window::Fixed { from, to } => {
                let md = (d.month, d.day);
                if from <= to {
                    from <= md && md <= to
                } else {
                    md >= from || md <= to
                }
            }
            Window::Easter { days } => (d.days() - easter(d.year).days()).abs() <= days,
        }
    }
}

/// A seasonal visitor, named as in [`super::OBJECT_NAMES`].
#[derive(Clone, Copy, Debug)]
pub struct Season {
    pub name: &'static str,
    pub window: Window,
}

/// The seasonal visitors, in the order they follow the ordinary objects in
/// [`super::OBJECT_NAMES`].
pub const SEASONS: [Season; 4] = [
    Season {
        name: "santa",
        window: Window::Fixed {
            from: (12, 1),
            to: (12, 26),
        },
    },
    Season {
        name: "ghost",
        window: Window::Fixed {
            from: (10, 20),
            to: (11, 1),
        },
    },
    Season {
        name: "fireworks",
        window: Window::Fixed {
            from: (12, 31),
            to: (1, 1),
        },
    },
    Season {
        name: "bunny",
        window: Window::Easter { days: 3 },
    },
];

/// The share of random spawns that go to an in-season visitor.
pub const SEASONAL_CHANCE: f64 = 1.0 / 3.0;

/// Indices into [`SEASONS`] of the visitors in season on `date`.
pub fn in_season(date: Date) -> impl Iterator<Item = usize> {
    (0..SEASONS.len()).filter(move |&i| SEASONS[i].window.contains(date))
}

/// Pick a random object for `date`: `roll` and `pick` are uniform in
/// `0.0..1.0`. With chance [`SEASONAL_CHANCE`] (if anything is in season)
/// one of the in-season visitors, otherwise one of the ordinary objects.
pub fn choose(date: Date, roll: f64, pick: f64) -> usize {
    let seasonal: Vec<usize> = in_season(date).collect();
    let index = |n: usize| {
        if n == 0 {
            0
        } else {
            ((pick * n as f64) as usize).min(n - 1)
        }
    };
    if !seasonal.is_empty() && roll < SEASONAL_CHANCE {
        ORDINARY + seasonal[index(seasonal.len())]
    } else {
        index(ORDINARY)
    }
}

static DATE_OVERRIDE: OnceLock<Option<Date>> = OnceLock::new();

/// Pretend today is `[YYYY-]MM-DD` (debugging aid, `FISHIES_DATE`). Invalid
/// dates are ignored.
pub fn set_date_override(s: Option<&str>) {
    let date = s.and_then(|s| parse_date(s, local_today().year));
    let _ = DATE_OVERRIDE.set(date);
}

/// Today's date, or the one set with [`set_date_override`].
pub fn today() -> Date {
    DATE_OVERRIDE
        .get()
        .copied()
        .flatten()
        .unwrap_or_else(local_today)
}

/// Parse `[YYYY-]MM-DD`; without a year, `this_year` is used.
pub fn parse_date(s: &str, this_year: i32) -> Option<Date> {
    let parts: Vec<&str> = s.trim().split('-').collect();
    let num = |p: &str| {
        (!p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
            .then(|| p.parse::<u32>().ok())
            .flatten()
    };
    let (year, month, day) = match parts[..] {
        [m, d] => (this_year, num(m)?, num(d)?),
        [y, m, d] if y.len() == 4 => (num(y)? as i32, num(m)?, num(d)?),
        _ => return None,
    };
    ((1..=12).contains(&month) && (1..=days_in_month(year, month)).contains(&day))
        .then_some(Date::new(year, month, day))
}

fn days_in_month(year: i32, month: u32) -> u32 {
    let is_leap = year.rem_euclid(4) == 0 && (year.rem_euclid(100) != 0 || year.rem_euclid(400) == 0);
    match month {
        2 if is_leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Easter Sunday (anonymous Gregorian algorithm).
pub fn easter(year: i32) -> Date {
    let a = year % 19;
    let (b, c) = (year / 100, year % 100);
    let (d, e) = (b / 4, b % 4);
    let f = (b + 8) / 25;
    let g = (b - f + 1) / 3;
    let h = (19 * a + b - d - g + 15) % 30;
    let (i, k) = (c / 4, c % 4);
    let l = (32 + 2 * e + 2 * i - h - k) % 7;
    let m = (a + 11 * h + 22 * l) / 451;
    let n = h + l - 7 * m + 114;
    Date::new(year, (n / 31) as u32, (n % 31 + 1) as u32)
}

/// Days since 1970-01-01 of a civil date (Howard Hinnant's algorithm).
fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let y = year as i64 - i64::from(month <= 2);
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (month as i64 + 9) % 12;
    let doy = (153 * mp + 2) / 5 + day as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// The civil date `days` after 1970-01-01, the inverse of [`days_from_civil`].
#[cfg(any(test, not(any(unix, windows))))]
fn civil_from_days(days: i64) -> Date {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = (yoe + era * 400 + i64::from(month <= 2)) as i32;
    Date::new(year, month, day)
}

/// Today in the local time zone.
#[cfg(unix)]
pub fn local_today() -> Date {
    // SAFETY: `time` accepts a null pointer, and `localtime_r` only writes
    // to the `tm` we own.
    let tm = unsafe {
        let t = libc::time(std::ptr::null_mut());
        let mut tm: libc::tm = std::mem::zeroed();
        libc::localtime_r(&t, &mut tm);
        tm
    };
    Date::new(tm.tm_year + 1900, (tm.tm_mon + 1) as u32, tm.tm_mday as u32)
}

/// Today in the local time zone.
#[cfg(windows)]
pub fn local_today() -> Date {
    use windows_sys::Win32::Foundation::SYSTEMTIME;
    use windows_sys::Win32::System::SystemInformation::GetLocalTime;
    let mut st = SYSTEMTIME::default();
    // SAFETY: `GetLocalTime` only writes to the struct we pass.
    unsafe { GetLocalTime(&mut st) };
    Date::new(st.wYear as i32, st.wMonth as u32, st.wDay as u32)
}

/// Today in UTC, on platforms without a local time zone API.
#[cfg(not(any(unix, windows)))]
pub fn local_today() -> Date {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64);
    civil_from_days(secs.div_euclid(86_400))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SANTA: usize = 0;
    const GHOST: usize = 1;
    const FIREWORKS: usize = 2;
    const BUNNY: usize = 3;

    fn on(y: i32, m: u32, d: u32) -> Vec<usize> {
        in_season(Date::new(y, m, d)).collect()
    }

    #[test]
    fn window_boundaries() {
        assert_eq!(on(2025, 11, 30), Vec::<usize>::new());
        assert_eq!(on(2025, 12, 1), vec![SANTA]);
        assert_eq!(on(2025, 12, 26), vec![SANTA]);
        assert_eq!(on(2025, 12, 27), Vec::<usize>::new());
        assert_eq!(on(2025, 10, 19), Vec::<usize>::new());
        assert_eq!(on(2025, 10, 20), vec![GHOST]);
        assert_eq!(on(2025, 10, 31), vec![GHOST]);
        assert_eq!(on(2025, 11, 1), vec![GHOST]);
        assert_eq!(on(2025, 11, 2), Vec::<usize>::new());
        assert_eq!(on(2025, 7, 1), Vec::<usize>::new());
    }

    #[test]
    fn fireworks_wrap_around_the_new_year() {
        assert_eq!(on(2025, 12, 30), Vec::<usize>::new());
        assert_eq!(on(2025, 12, 31), vec![FIREWORKS]);
        assert_eq!(on(2026, 1, 1), vec![FIREWORKS]);
        assert_eq!(on(2026, 1, 2), Vec::<usize>::new());
    }

    #[test]
    fn easter_dates() {
        assert_eq!(easter(2019), Date::new(2019, 4, 21));
        assert_eq!(easter(2024), Date::new(2024, 3, 31));
        assert_eq!(easter(2025), Date::new(2025, 4, 20));
        assert_eq!(easter(2026), Date::new(2026, 4, 5));
        assert_eq!(easter(2038), Date::new(2038, 4, 25));
    }

    #[test]
    fn bunny_window_crosses_month_boundaries() {
        // Easter 2024 is on March 31
        assert_eq!(on(2024, 3, 27), Vec::<usize>::new());
        assert_eq!(on(2024, 3, 28), vec![BUNNY]);
        assert_eq!(on(2024, 4, 3), vec![BUNNY]);
        assert_eq!(on(2024, 4, 4), Vec::<usize>::new());
        // Easter 2026 is on April 5
        assert_eq!(on(2026, 4, 2), vec![BUNNY]);
        assert_eq!(on(2026, 4, 8), vec![BUNNY]);
        assert_eq!(on(2026, 4, 9), Vec::<usize>::new());
    }

    #[test]
    fn civil_days_round_trip() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(days_from_civil(2000, 3, 1), 11_017);
        for days in [-1, 0, 59, 365, 11_016, 20_000, 24_837] {
            let d = civil_from_days(days);
            assert_eq!(d.days(), days, "{d:?}");
        }
        assert_eq!(civil_from_days(-1), Date::new(1969, 12, 31));
    }

    #[test]
    fn parse_dates() {
        assert_eq!(parse_date("12-24", 2030), Some(Date::new(2030, 12, 24)));
        assert_eq!(parse_date("2024-03-31", 2030), Some(Date::new(2024, 3, 31)));
        assert_eq!(parse_date(" 1-1 ", 2030), Some(Date::new(2030, 1, 1)));
        assert_eq!(parse_date("2024-02-29", 2030), Some(Date::new(2024, 2, 29)));
        assert_eq!(parse_date("2023-02-29", 2030), None);
        for bad in [
            "", "12", "13-01", "00-10", "04-31", "x-1", "1-+1", "24-1-1", "1-2-3-4",
        ] {
            assert_eq!(parse_date(bad, 2030), None, "{bad:?}");
        }
    }

    #[test]
    fn off_season_never_picks_a_seasonal_visitor() {
        let date = Date::new(2025, 7, 1);
        for roll in [0.0, 0.1, 0.5, 0.99] {
            for pick in [0.0, 0.3, 0.6, 0.999] {
                assert!(choose(date, roll, pick) < ORDINARY);
            }
        }
    }

    #[test]
    fn in_season_visitors_get_a_third_of_the_spawns() {
        let christmas = Date::new(2025, 12, 24);
        assert_eq!(choose(christmas, 0.0, 0.5), ORDINARY + SANTA);
        assert_eq!(choose(christmas, 0.0, 0.9999), ORDINARY + SANTA);
        assert!(choose(christmas, SEASONAL_CHANCE, 0.0) < ORDINARY);
        assert_eq!(choose(christmas, 0.5, 0.9999), ORDINARY - 1);
        let halloween = Date::new(2025, 10, 31);
        assert_eq!(choose(halloween, 0.2, 0.0), ORDINARY + GHOST);
        let new_year = Date::new(2026, 1, 1);
        assert_eq!(choose(new_year, 0.3, 1.0), ORDINARY + FIREWORKS);
        let easter = Date::new(2025, 4, 20);
        assert_eq!(choose(easter, 0.0, 0.0), ORDINARY + BUNNY);
    }

    #[test]
    fn local_today_is_a_valid_date() {
        let d = local_today();
        assert!(d.year >= 2024);
        assert!((1..=12).contains(&d.month));
        assert!((1..=days_in_month(d.year, d.month)).contains(&d.day));
    }
}

use chrono::{offset::Local, Duration, NaiveDateTime};
use console::Term;

pub struct Schedule {
    text: String,
    datetime: NaiveDateTime,
}

impl Schedule {
    pub fn new() -> Self {
        Self {
            text: String::from(
                "\
------------------
01  08:30 -- 09:15
02  09:20 -- 10:05
------------------
03  10:25 -- 11:10
04  11:15 -- 12:00
------------------
05  14:00 -- 14:45
06  14:50 -- 15:35
07  15:40 -- 16:25
------------------
08  16:35 -- 17:20
09  17:25 -- 18:10
10  18:20 -- 19:05
------------------
11  19:20 -- 20:05
12  20:10 -- 20:55
13  21:00 -- 21:45
------------------",
            ),
            datetime: "2020-07-09T00:00:00".parse().unwrap(),
        }
    }

    pub fn run(&self) -> std::io::Result<()> {
        println!("{}", self.text);
        let term = Term::stderr();
        loop {
            let mut duration = Local::now().naive_local() - self.datetime;
            let days = duration.num_days();
            duration = duration - Duration::days(days);
            let hours = duration.num_hours();
            duration = duration - Duration::hours(hours);
            let minutes = duration.num_minutes();
            duration = duration - Duration::minutes(minutes);
            let seconds = duration.num_seconds();
            term.write_str(&format!(
                "Return School {} days {:02}:{:02}:{:02}",
                days, hours, minutes, seconds
            ))?;
            std::thread::sleep(std::time::Duration::from_micros(5000));
            term.clear_line()?;
        }
    }
}

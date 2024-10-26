use std::fmt::Display;

use anyhow::{anyhow, bail, Result};
use reqwest::blocking::Client;
use scraper::Html;

use crate::selector;

const SCHEDULE_PAGE_URL: &str = "https://hac.friscoisd.org/HomeAccess/Content/Student/Classes.aspx";

/// A course a student is currently enrolled in, for this academic year.
#[derive(Debug, Clone)]
pub struct Course {
    /// The common name of the course (eg. "English 2 Adv").
    name: String,
    /// The internal HAC id of the course.
    id: String,

    /// The period of the day the course is present.
    period: Period,
    /// The course's teacher.
    teacher: String,
    /// The classroom the course is taught in.
    classroom: String,
}

impl Course {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn period(&self) -> &Period {
        &self.period
    }

    pub fn teacher(&self) -> &str {
        &self.teacher
    }

    pub fn classroom(&self) -> &str {
        &self.classroom
    }
}

/// A period of the day, in block schedule where the `period_number` ranges from
/// 1-4 and the day is either `A` or `B`.
#[derive(Debug, Clone)]
pub struct Period {
    number: PeriodNumber,
    day: Day,
}

/// The period number of a course in a day. For most courses, this will range from
/// 1 to 4, with the exception of some which take place in `ADV` (advisory).
#[derive(Debug, Clone)]
pub enum PeriodNumber {
    Number(u32),
    Unknown(String),
}

#[derive(Debug, Clone)]
pub enum Day {
    A,
    B,
}

pub fn get_schedule(client: &Client) -> Result<Vec<Course>> {
    let resp = client.get(SCHEDULE_PAGE_URL).send()?.text()?;
    let document = Html::parse_document(&resp);

    let mut courses = Vec::new();

    for class in document.select(selector!(
        "#plnMain_dgSchedule > tbody > tr.sg-asp-table-data-row"
    )) {
        let mut elements = class.select(selector!("td")).map(|c| {
            let first_element = c
                .children()
                .find(|child| child.value().as_element().is_some());

            let text = match first_element {
                None => c.text().next(),
                Some(e) => e
                    .first_child()
                    .and_then(|e| e.value().as_text().map(|text| &**text)),
            };

            text.unwrap_or("").trim().to_owned()
        });

        let id = elements.next().ok_or(anyhow!("missing course id"))?;
        let name = elements.next().ok_or(anyhow!("missing course name"))?;
        let period_number = elements.next().ok_or(anyhow!("missing period number"))?;
        let teacher = elements.next().ok_or(anyhow!("missing teacher"))?;
        let classroom = elements.next().ok_or(anyhow!("missing classroom"))?;
        let day = elements.next().ok_or(anyhow!("missing day"))?;

        let period = Period::from_elements(&period_number, &day)?;

        courses.push(Course {
            id,
            name,
            period,
            teacher,
            classroom,
        });
    }

    Ok(courses)
}

impl Period {
    pub fn from_elements(number: &str, day: &str) -> Result<Self> {
        let number = match number.trim().parse::<u32>() {
            Ok(number) => PeriodNumber::Number(number),
            Err(_) => PeriodNumber::Unknown(number.trim_end().to_owned()),
        };

        let day = match day.chars().next() {
            Some('A') => Day::A,
            Some('B') => Day::B,
            _ => bail!("invalid day, {day}"),
        };

        Ok(Self { number, day })
    }
}

impl Display for Period {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            match &self.number {
                PeriodNumber::Number(n) => n.to_string(),
                PeriodNumber::Unknown(u) => u.clone(),
            },
            match self.day {
                Day::A => "A",
                Day::B => "B",
            }
        )
    }
}

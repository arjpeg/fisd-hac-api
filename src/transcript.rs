use std::collections::HashMap;

use anyhow::{anyhow, bail, Result};
use reqwest::blocking::Client;
use scraper::{selectable::Selectable, Html, Selector};

use crate::selector;

const TRANSCRIPT_PAGE_URL: &str =
    "https://hac.friscoisd.org/HomeAccess/Content/Student/Transcript.aspx";

/// The list of grade entries, along with the cumulative GPA.
#[derive(Debug, Clone, PartialEq)]
pub struct Transcript {
    /// All entries present.
    pub entries: Vec<TranscriptEntry>,
}

impl Transcript {
    pub fn gpa(&self) -> f32 {
        let sum: f32 = self.entries.iter().map(|e| e.gpa()).sum();

        sum / self.entries.len() as f32
    }

    pub fn combine(transcripts: &[Transcript]) -> Transcript {
        let entries = transcripts
            .iter()
            .flat_map(|transcript| transcript.entries.iter());

        // if we find duplicates, keep their average
        let mut seen: HashMap<String, Vec<TranscriptEntry>> = HashMap::new();

        for entry in entries {
            seen.entry(entry.name.clone())
                .or_default()
                .push(entry.clone());
        }

        let entries = seen
            .into_iter()
            .map(|(k, v)| {
                let average = v.iter().map(|e| e.grade).sum::<f32>() / v.len() as f32;

                TranscriptEntry {
                    name: k,
                    weightage: v[0].weightage,
                    grade: average,
                }
            })
            .collect::<Vec<_>>();

        Self { entries }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TranscriptEntry {
    pub weightage: f32,
    pub grade: f32,
    pub name: String,
}

impl TranscriptEntry {
    pub fn new(name: String, grade: f32) -> Self {
        Self {
            weightage: Self::get_weightage(&name),
            name,
            grade,
        }
    }

    pub fn get_weightage(name: &str) -> f32 {
        if name.contains("AP") {
            6.0
        } else if name.contains("Adv") {
            5.5
        } else {
            5.0
        }
    }

    pub fn gpa(&self) -> f32 {
        self.weightage - (100.0 - self.grade.round()) / 10.0
    }
}

pub fn get_transcript(client: &Client) -> Result<Transcript> {
    let transcript_page_resp = client.get(TRANSCRIPT_PAGE_URL).send()?.text()?;
    let document = Html::parse_document(&transcript_page_resp);

    let mut cumulative_entries = Vec::new();

    for group in document.select(selector!(".sg-transcript-group")) {
        let year_entries = group
            .select(&Selector::parse(".sg-asp-table-data-row").unwrap())
            .map(|entry| {
                let mut children = entry
                    .child_elements()
                    .filter_map(|cell| cell.text().next())
                    .skip(1);

                let name = children
                    .next()
                    .ok_or(anyhow!("missing course name"))?
                    .to_owned();

                let grade = children
                    .next()
                    .ok_or(anyhow!("missing course grade"))?
                    .parse::<f32>()?;

                if name.split(" ").any(|word| word == "EA") {
                    bail!("skip")
                }

                Ok(TranscriptEntry::new(name, grade))
            })
            .filter_map(|entry: Result<TranscriptEntry>| entry.ok())
            .collect::<Vec<_>>();

        cumulative_entries.extend(year_entries);
    }

    Ok(Transcript {
        entries: cumulative_entries,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extended_transcript() {
        let transcripts = [
            Transcript {
                entries: vec![
                    TranscriptEntry {
                        name: "Class A".to_owned(),
                        grade: 90.0,
                        weightage: 5.5,
                    },
                    TranscriptEntry {
                        name: "Class B".to_owned(),
                        grade: 100.0,
                        weightage: 5.0,
                    },
                ],
            },
            Transcript {
                entries: vec![TranscriptEntry {
                    name: "Class A".to_owned(),
                    grade: 100.0,
                    weightage: 5.5,
                }],
            },
        ];

        let mut entries = Transcript::combine(&transcripts).entries;
        entries.sort_by_key(|e| e.name.clone());

        assert_eq!(
            entries,
            vec![
                TranscriptEntry {
                    name: "Class A".to_owned(),
                    grade: 95.0,
                    weightage: 5.5,
                },
                TranscriptEntry {
                    name: "Class B".to_owned(),
                    grade: 100.0,
                    weightage: 5.0,
                },
            ]
        );
    }
}

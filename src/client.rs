use anyhow::{anyhow, Result};
use reqwest::blocking::ClientBuilder;
use scraper::Html;

use crate::{
    classes,
    schedule::{self, Course},
    selector,
    transcript::{self, Transcript},
};

const LOGIN_PAGE_URL: &str = "https://hac.friscoisd.org/HomeAccess/Account/LogOn";

/// Represents an open connection to the HAC centers, with cookies
/// being persisted with each connection. This is the main gateway
/// for getting data through HAC.
#[derive(Clone)]
pub struct Client {
    /// The internal open network connection.
    client: reqwest::blocking::Client,
}

impl Client {
    /// Authenticates with HAC servers given a username and password.
    pub fn new(username: &str, password: &str) -> Result<Self> {
        let client = ClientBuilder::new().cookie_store(true).build()?;

        let login_screen_resp = client.get(LOGIN_PAGE_URL).send()?.text()?;
        let document = Html::parse_document(&login_screen_resp);

        let verification_token = document
            .select(selector!(r#"input[name="__RequestVerificationToken"]"#))
            .next()
            .and_then(|e| e.value().attr("value"))
            .ok_or(anyhow!("request verification token not found"))?;

        let payload = [
            ("__RequestVerificationToken", verification_token),
            ("SCKTY00328510CustomEnabled", "False"),
            ("SCKTY00436568CustomEnabled", "False"),
            ("Database", "10"),
            ("VerificationOption", "UsernamePassword"),
            ("LogOnDetails.UserName", username),
            ("LogOnDetails.Password", password),
        ];

        let resp = client.post(LOGIN_PAGE_URL).form(&payload).send()?;

        if resp.url().as_str() == LOGIN_PAGE_URL {
            Err(anyhow!("failed to login; invalid username or password?"))
        } else {
            Ok(Self { client })
        }
    }

    /// Returns the schedule (the current classes) a student is enrolled in.
    pub fn get_schedule(&self) -> Result<Vec<Course>> {
        schedule::get_schedule(&self.client)
    }

    /// Returns the most recently published transcript.
    pub fn get_transcript(&self) -> Result<Transcript> {
        transcript::get_transcript(&self.client)
    }

    /// Returns the grades entered for a particular quarter, this year.
    pub fn get_quarter_grades(&self, quarter: u8) -> Result<Transcript> {
        classes::get_quarter_grades(&self.client, quarter)
    }
}

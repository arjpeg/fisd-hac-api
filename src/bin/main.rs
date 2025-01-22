use anyhow::Result;
use hac::{client::Client, MergeStrategy, Transcript};

fn print_schedule(client: &Client) -> Result<()> {
    println!("Currently enrolled courses: ");

    for course in client.get_schedule()? {
        println!(
            "\t{} taught by {} on {}",
            course.name(),
            course.teacher(),
            course.period()
        );
    }

    Ok(())
}

fn print_cumulative_gpa(client: &Client) -> Result<()> {
    let mut transcripts = Vec::new();

    println!("Getting last posted transcript");
    let posted_transcript = client.get_transcript().expect("could not get transcript");

    println!("  posted transcript gpa: {}", posted_transcript.gpa());

    transcripts.push(posted_transcript);

    for quarter in 1..=4 {
        println!("Getting quarter #{quarter} grades");

        let quarter_grades = client
            .get_quarter_grades(quarter)
            .expect("could not get quarter grades");

        if quarter_grades.entries.is_empty() {
            println!("  no grades found for quarter... stopping now");
            break;
        }

        println!("  quarter gpa: {}", quarter_grades.gpa());

        transcripts.push(quarter_grades);
    }

    let cumulative_transcript = Transcript::combine(&transcripts, MergeStrategy::Average);

    println!(
        "Cumulative GPA with {} total entries: {}",
        cumulative_transcript.entries.len(),
        cumulative_transcript.gpa()
    );

    Ok(())
}

fn main() -> Result<()> {
    let client = Client::new("", "").expect("could not authenticate with hac");

    print_cumulative_gpa(&client)?;

    Ok(())
}

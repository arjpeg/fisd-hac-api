use crate::{
    selector,
    transcript::{Transcript, TranscriptEntry},
};

use anyhow::{anyhow, Result};
use reqwest::blocking::Client;
use scraper::Html;

const CURRENT_GRADES_PAGE_URL: &str =
    "https://hac.friscoisd.org/HomeAccess/Content/Student/Assignments.aspx";

pub fn get_quarter_grades(client: &Client, quarter: u8) -> Result<Transcript> {
    let quarter = format!("{quarter}-2025");
    let mut payload = QUARTER_GRADES_BASE_PAYLOAD.to_vec();

    payload.extend([("ctl00$plnMain$ddlReportCardRuns", quarter.as_str())]);

    let grades_page_resp = client
        .post(CURRENT_GRADES_PAGE_URL)
        .form(&payload)
        .send()?
        .text()?;

    let document = Html::parse_document(&grades_page_resp);
    let classes = document.select(selector!(".AssignmentClass"));

    let mut grades = Vec::new();

    for class in classes {
        let header = class.select(selector!(".sg-header")).next().unwrap();

        let mut children = header
            .select(selector!(":not(button)"))
            .map(|c| c.text().next().unwrap_or(""));

        let title = children
            .next()
            .and_then(|name| name.trim().split("    ").last())
            .ok_or(anyhow!("missing title"))?
            .trim()
            .to_owned();

        children.next();

        let Ok(grade) = children
            .next()
            .map(|g| {
                g.chars()
                    .filter(|c| *c == '.' || c.is_ascii_digit())
                    .collect::<String>()
            })
            .ok_or(anyhow!("missing grade"))?
            .parse::<f32>()
        else {
            // no grade has been entered for this class
            continue;
        };

        let grade = grade.round();

        if grade != 0.0 {
            if title.contains("Computer Science A") {
                // grades.push(TranscriptEntry::new(title.clone(), grade));
            }

            grades.push(TranscriptEntry::new(title, grade));
        }
    }

    Ok(Transcript { entries: grades })
}

const QUARTER_GRADES_BASE_PAYLOAD: [(&str, &str); 54] = [
    ("__EVENTTARGET", "ctl00$plnMain$btnRefreshView"),
    ("__EVENTARGUMENT", ""),
    ("__VIEWSTATE", "z4s0Cs1h6C6fXk3ap2PZOHaZeybXN1j8I6cHSot9b6vuii/luodRzmNcaabTi9ScsOK+U3GKmy6ztupbTRyX0IEwla6FavU40CYd45+/CXMak6mW25nnmtj3KKwNez3jILZi1xyfvHecdgf6Fy8jOJT+vhfHxJZNxbpVOj2mSxXpwzG2Me4bksNvaDmUr8KbJa+RLgPbQh6nn0VVvJXH0yh5t3a8H82LeA/Etde05tuSarywX1JNn0Xf3777s7x1+Lt5WYY11bFE7j+RiB7oQPyAsgzXLGv+03MvMsP7H1VNz7EmQOGCg2dHhJg1RrzWi6G0zPe2ysQWa16VAd+b99DStbyUURBq/n4wn3H4GjoewKaU2f27gBEv3R7d5dmbgCSWiCnYLRkHPXOayW8UxELNKNEvioW+QpgkrSfIAEg0BoQUvq4HHIIkWoVZSsSsAxSbTUBRw1kMmiBLhU+mhfqYuuc0pZorsXx3BsUEi9LfTqma6mB4qegyCuV0V7h/o4lLsT2q7A473ETXeTBYj261ek5LvQzVELP/Q9wcEwzmVAeJKk1oq6uOEPNoUx7G8ui77yWU1tMDul9yv55sfdsyqoQy58DBHPpyrQPLinpw/TD7y0KTzDjse7l/3bPq3h1B98EUnJn/jPRzDmAbv/uk/UWhhjAh2hCYkdHpWhig/TmF0euyNTWzaRhdpc7mGXBiMO45zha4ppN+spUBtr5J9FyjdOvMT7DjGXUiHFE4+9CpL5VqsHJpIwMNVxRO1NHDYOEYamtQzf2B6USuHMTrP+C14ZKh3FxK/WJX6FFkTZHT/azwGPudvdLO91KEztpjB0mNuv/IyWttXFYLnlptsAI2/hnlXQP2IYgyuavm97H5Lty6DKv9/laEY9+qh30YrlMsvW6GVBhFNWuHAAYcx2h+GS1lyW2gYx8FYePUC+AjA9jEPTnLYbFsUBIBu16QlFQ7Iscrb/aBBz5BRknUJm5bqVzZfqyOw/Ht9IfF/4q9TJuVKI365GRgJQYCm8LQdopwiZbeJth/FrWQerR39HRwoXzkFNChNZceQW1x95+MfgVeExtoZqpDmwxNPDw9+6198VlQi1y4p2fXhqbBaDMDKRloGGgoQvZDfeEf1m5Y9Fhu/ecSj3PD2mJ/8qjsifYWzejKph7aqDEA5sEeTXwd1Pa4CvHRQ6eTGIw4PyzWbJe9tKlS4b51Vh1zHj/9/XL3w99PcSX9zoM49yV9jMUEXSPOXv4T6sB/JyZR75CXFpmKPBw3XJe/g0/b0n9syGckWrjurhzwepHdzZmkNdVtnxLulVzUWFLbzCXJ+3BKr50NQ0ABnSnyX+FidqEdEnVhMGVNsSJEQBwRymKJLlqTzvRvW5auXdZzjcOfSMDd+cg1ms0L5AgyOJb0FIHZF1KO6TR0OqgO/fJDfAq4bGK+uLC+M7tdcciFnPd+FEo2YjpxYmov4CDW+Nx5fA6lx3yjY6AQblfRpVnl8v8bmzbFmYCHlhkTJ8Pa7D+JoaOwSheddImj5ofLt4llsGs9r6qqtGnQWEsE5/iURQCeG1AmQjPtS33O4hjh/X7+OtThW1ieaRS9Wgq+FB5jA8KPRMtBabMM0G5IjLfeFtUT8IhAnEwJr71HZwcg+mPcQBDMZKvkpGbroIJ5cslYWg3R97CyNlRmwgBKoga+9yAjO21H4qXkQUVTpJCqgRHqPfGxkTdt9OyopO9Nrcj+vBvYV1LqhaLRPUMytX/YeYlL53lOCM5WF6K+OgEa02I="),
    ("__VIEWSTATEGENERATOR", "B0093F3C"),
    ("__EVENTVALIDATION", "vfmMthsarDfW3sWAVCqyfsSkASxvPSO+uca79qa44PWtgh0Ou8m2AXReJoJle4FYjtkr87Wf8MW68H2N0JUQP7keB9tKjjLNBu7xooLuTtzbLY4UZcPNW6AElft08YOJPAb01qsFRNfelILoKjjiy/S8IsNrkL/m0jGwCKzd2iiptgchGzqhsI7vB+FoNwH9XqUFxNhAUc9Jc4ovvyrJ330nu+KiLSMb1aPREHW1p2rYXrzvyjxstnWyRyF6NGFkMr37ZID+PenLUCUrZL9MmLr3+MvqaSrYa//T5es0IDsCCrAuIckvqR9ORGVEfE+tiwlTnnqhAoV0R5M32wZvEcf9Gv4xoU7oZyhQTNA/UdSkOI896kq6q6pHNnAvKovJF82x+VNtyfiKUvMFp/wdzsmltg9ELaCH5fVkb7F+c3PZTnlOwZYDEhUGYaMHF5QLSoxKcKVMiYAcWaIt9e03zSGkfYSnsSJxJgPdA1H04cHIcCxlpn/Ry8w8ppeu3MsmoFD+TtCcjXOfVTaDv6OR+Rma49a4JKW77axtMrgXlzhn7Ks+ezXs8TMZ3aGk8xGqu5YP5l7ujDV7rRneRtF/tDwhcu9tEeGT/if5cYGeHTio9OY4GgC5t5Tpsr9eCWC1vYpCf+wEQ9o5D7OpZHlFNbu8EIhbs77jlFwS8lr9cpUd4POgO0fc69ncneRu5+IJti7benzMzG+z7Spsm4xCQ09eVUo7seg7q1rgCN+QVf3GwpIGPcp5UVObboKjK3jLOfqwWFPZvBNLJdlsagCZ87Qar1Hav0ET7lxJ0e6DXJkQRYKOKhfceo5zhHXrkzkxhvu77Dmi5HLyczIOe0K9n+KuUfuIsOscHKkeG2XDBq/C/gbKwwgsE+C3xLcGCZFm3Q4EnwbrnFAVlXPMiklEF2bT2W3XfsXKlu1GpSbYMico6LFjB3KroL95PAn+kfqLMGrPQglySZIU76wSOrLhRizkIrDDvOMTO8BShVhKXQxnIhj1kopLM6SE5HiR7ucqvXozcyArpPQtpUDaEMwf+CRCJhT4wqt36Ts9CeHqgfkOEEYSkIHHL26MsMK6mC+5tjHRmQ33uxN9f473sbjhBEgEc2VIkvs+grWK6sKaoFnIUQlgaznz/yiu2YeoFJXzqL+SAs0QTueuZE9rycTodc1tYkncxFSJUQRcE/IrcCGG1YdaFNYRTistJOeEpMPPx14vNRHabu+hjTEsamTVcm9nlT1WdfvI62Nh/I5y/kGdH7yb3CwyRRnzRwRM1nZ/jAvLk55DwkXQjRZjLzqHZNMihPHLAOubjjJGL0id/IzRYrphWHgBJBvfxmfxpnTNmiO+tOvFpRV08g6/HLPLBhsGFG4oGNr7IblaRF8bJvG2BdmJcwCVCqn76AOQebbni8F6EWc13+wpHBZ6Uk3j5vcWxwBhjnQ2iuleYoY8Ab+SY+u7pX5s0MOHLNZQUSRD8/D0X5GyTtTcHQFPh0AdslwNGM9pYbbMvLlFiVUukg2FC4+UuZVtFumif+3NnNY00SBhnvJmTdWCIfAw6upjScY0YpW9chCpmnPhbqzfpi8KFi5vsrpenitWl/v98Z7BR+EQuWyrj07lsf7Uaipgnc608afMppCJCoReiuphyD4="),
    ("ctl00$plnMain$hdnValidMHACLicense", "Y"),
    ("ctl00$plnMain$hdnIsVisibleClsWrk", "N"),
    ("ctl00$plnMain$hdnIsVisibleCrsAvg", "N"),
    ("ctl00$plnMain$hdnJsAlert", "Averages cannot be displayed when  Report Card Run is set to (All Runs)."),
    ("ctl00$plnMain$hdnTitle", "Classwork"),
    ("ctl00$plnMain$hdnLastUpdated", "Last Updated"),
    ("ctl00$plnMain$hdnDroppedCourse", " This course was dropped as of "),
    ("ctl00$plnMain$hdnddlClasses", "(All Classes)"),
    ("ctl00$plnMain$hdnddlCompetencies", "(All Classes)"),
    ("ctl00$plnMain$hdnCompDateDue", "Date Due"),
    ("ctl00$plnMain$hdnCompDateAssigned", "Date Assigned"),
    ("ctl00$plnMain$hdnCompCourse", "Course"),
    ("ctl00$plnMain$hdnCompAssignment", "Assignment"),
    ("ctl00$plnMain$hdnCompAssignmentLabel", "Assignments Not Related to Any Competency"),
    ("ctl00$plnMain$hdnCompNoAssignments", "No assignments found"),
    ("ctl00$plnMain$hdnCompNoClasswork", "Classwork could not be found for this competency for the selected report card run."),
    ("ctl00$plnMain$hdnCompScore", "Score"),
    ("ctl00$plnMain$hdnCompPoints", "Points"),
    ("ctl00$plnMain$hdnddlReportCardRuns1", "(All Runs)"),
    ("ctl00$plnMain$hdnddlReportCardRuns2", "(All Terms)"),
    ("ctl00$plnMain$hdnbtnShowAverage", "Show All Averages"),
    ("ctl00$plnMain$hdnShowAveragesToolTip", "Show all student's averages"),
    ("ctl00$plnMain$hdnPrintClassworkToolTip", "Print all classwork"),
    ("ctl00$plnMain$hdnPrintClasswork", "Print Classwork"),
    ("ctl00$plnMain$hdnCollapseToolTip", "Collapse all courses"),
    ("ctl00$plnMain$hdnCollapse", "Collapse All"),
    ("ctl00$plnMain$hdnFullToolTip", "Switch courses to Full View"),
    ("ctl00$plnMain$hdnViewFull", "Full View"),
    ("ctl00$plnMain$hdnQuickToolTip", "Switch courses to Quick View"),
    ("ctl00$plnMain$hdnViewQuick", "Quick View"),
    ("ctl00$plnMain$hdnExpand", "Expand All"),
    ("ctl00$plnMain$hdnExpandToolTip", "Expand all courses"),
    ("ctl00$plnMain$hdnChildCompetencyMessage", "This competency is calculated as an average of the following competencies"),
    ("ctl00$plnMain$hdnCompetencyScoreLabel", "Grade"),
    ("ctl00$plnMain$hdnAverageDetailsDialogTitle", "Average Details"),
    ("ctl00$plnMain$hdnAssignmentCompetency", "Assignment Competency"),
    ("ctl00$plnMain$hdnAssignmentCourse", "Assignment Course"),
    ("ctl00$plnMain$hdnTooltipTitle", "Title"),
    ("ctl00$plnMain$hdnCategory", "Category"),
    ("ctl00$plnMain$hdnDueDate", "Due Date"),
    ("ctl00$plnMain$hdnMaxPoints", "Max Points"),
    ("ctl00$plnMain$hdnCanBeDropped", "Can Be Dropped"),
    ("ctl00$plnMain$hdnHasAttachments", "Has Attachments"),
    ("ctl00$plnMain$hdnExtraCredit", "Extra Credit"),
    ("ctl00$plnMain$hdnType", "Type"),
    ("ctl00$plnMain$hdnAssignmentDataInfo", "Information could not be found for the assignment"),
    ("ctl00$plnMain$ddlClasses", "ALL"),
    ("ctl00$plnMain$ddlCompetencies", "ALL"),
    ("ctl00$plnMain$ddlOrderBy", "Class"),
];

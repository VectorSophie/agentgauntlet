use crate::AgentResults;
use anyhow::Result;
use std::path::Path;

pub fn write_html(results: &[AgentResults], output_path: &Path) -> Result<()> {
    let mut html = String::new();
    html.push_str("<!DOCTYPE html><html><head><title>AgentGauntlet Dashboard</title>");
    html.push_str("<script src=\"https://cdn.tailwindcss.com\"></script>");
    html.push_str("</head><body class=\"bg-gray-900 text-gray-100 p-8\">");

    html.push_str(
        "<h1 class=\"text-4xl font-bold mb-8 text-blue-400\">🛡️ AgentGauntlet Wall of Shame</h1>",
    );

    for res in results {
        let mut failed = 0;
        let total = res.runs.len();
        for r in res.runs {
            if r.score.score < 100 {
                failed += 1;
            }
        }
        let grade = if total == 0 {
            "N/A"
        } else if failed == 0 {
            "A+"
        } else if failed <= total / 4 {
            "B"
        } else if failed <= total / 2 {
            "C"
        } else {
            "F"
        };
        let grade_color = if failed == 0 {
            "text-green-500"
        } else if failed <= total / 2 {
            "text-yellow-500"
        } else {
            "text-red-500"
        };

        html.push_str("<div class=\"mb-12 p-6 bg-gray-800 rounded-lg shadow-xl\">");
        html.push_str(&format!("<h2 class=\"text-2xl font-semibold mb-4\">{} <span class=\"text-gray-400 text-lg\">(Grade: <span class=\"{}\">{}</span>)</span></h2>", res.display_name, grade_color, grade));

        html.push_str("<div class=\"grid grid-cols-1 md:grid-cols-2 gap-4\">");
        for run in res.runs {
            let status_color = if run.score.score < 100 {
                "border-red-500"
            } else {
                "border-green-500"
            };
            html.push_str(&format!(
                "<div class=\"border-l-4 {} p-4 bg-gray-700 rounded\">",
                status_color
            ));
            html.push_str(&format!(
                "<h3 class=\"font-bold\">{}</h3>",
                run.scenario_name
            ));

            for turn in &run.turns {
                for finding in &turn.findings {
                    html.push_str(&format!(
                        "<p class=\"text-red-400 text-sm mt-2\">❌ {}</p>",
                        finding.message
                    ));
                    if let Some(s) = &finding.patch_suggestion {
                        html.push_str(&format!(
                            "<p class=\"text-yellow-300 text-sm mt-1\">💡 <i>Fix: {}</i></p>",
                            s
                        ));
                    }
                }
            }
            html.push_str("</div>");
        }
        html.push_str("</div></div>");
    }

    html.push_str("</body></html>");
    std::fs::write(output_path, html)?;
    Ok(())
}

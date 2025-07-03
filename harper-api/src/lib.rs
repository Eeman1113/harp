use actix_web::{web, HttpResponse, Result};
use harper_core::{
    linting::{Lint, LintGroup, LintKind, Linter},
    Dialect, Document, FstDictionary,
};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::ops::Range;

// --- All of your existing structs and functions for the /lint endpoint remain here ---

#[derive(Serialize, Deserialize)]
pub struct Suggestions {
    pub recommendation: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct FormattedLintOutput {
    pub start: usize,
    pub length: usize,
    pub end: usize,
    #[serde(rename = "paragraphKey")]
    pub paragraph_key: String,
    pub paragraph: String,
    pub string: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub suggestions: Suggestions,
}

#[derive(Deserialize)]
pub struct LintRequest {
    pub text: String,
    #[serde(default)]
    pub ignore: Vec<String>,
}

struct Paragraph<'a> {
    key: usize,
    text: &'a str,
    start_offset: usize,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
enum LintKindPriority {
    Miscellaneous,
    Capitalization,
    Spelling,
    Style,
    Repetition,
    WordChoice,
}

fn get_priority(kind: &LintKind) -> LintKindPriority {
    match kind {
        LintKind::WordChoice => LintKindPriority::WordChoice,
        LintKind::Repetition => LintKindPriority::Repetition,
        LintKind::Style => LintKindPriority::Style,
        LintKind::Spelling => LintKindPriority::Spelling,
        LintKind::Capitalization => LintKindPriority::Capitalization,
        _ => LintKindPriority::Miscellaneous,
    }
}

pub async fn lint_text(request: web::Json<LintRequest>) -> Result<HttpResponse> {
    let document = Document::new_plain_english_curated(&request.text);
    let curated_text = document.get_source();
    let dictionary = FstDictionary::curated();
    let mut linter = LintGroup::new_curated(dictionary, Dialect::American);
    let mut lints_from_linter = linter.lint(&document);
    
    let ignore_set: HashSet<String> = request.ignore.iter().map(|s| s.to_lowercase()).collect();

    lints_from_linter.sort_by(|a, b| get_priority(&b.lint_kind).cmp(&get_priority(&a.lint_kind)));

    let mut final_lints: Vec<&Lint> = Vec::new();
    let mut claimed_spans: Vec<Range<usize>> = Vec::new();

    for lint in &lints_from_linter {
        let lint_range = lint.span.start..lint.span.end;
        let has_overlap = claimed_spans
            .iter()
            .any(|claimed| lint_range.start < claimed.end && lint_range.end > claimed.start);

        if !has_overlap {
            final_lints.push(lint);
            claimed_spans.push(lint_range);
        }
    }

    let curated_text_str: String = curated_text.iter().collect();
    let mut paragraphs: Vec<Paragraph> = Vec::new();
    let mut current_offset = 0;

    for (key, line_str) in curated_text_str.lines().enumerate() {
        let trimmed_line = line_str.trim();
        if !trimmed_line.is_empty() {
            let start_in_line = line_str.find(trimmed_line).unwrap_or(0);
            let absolute_start_offset = current_offset + start_in_line;
            
            paragraphs.push(Paragraph {
                key: key + 1,
                text: trimmed_line,
                start_offset: absolute_start_offset,
            });
        }
        current_offset += line_str.chars().count() + 1;
    }
    
    let mut lint_outputs: Vec<FormattedLintOutput> = final_lints
        .iter()
        .filter_map(|lint| {
            let linted_string = String::from_iter(curated_text.get(lint.span.start..lint.span.end)?);

            if ignore_set.contains(&linted_string.to_lowercase()) {
                return None;
            }
            
            if lint.lint_kind == LintKind::Spelling && linted_string == "s" {
                return None;
            }

            let containing_para = paragraphs
                .iter()
                .find(|p| {
                    let para_end_offset = p.start_offset + p.text.chars().count();
                    lint.span.start >= p.start_offset && lint.span.end <= para_end_offset
                })?;

            let relative_start = lint.span.start - containing_para.start_offset;
            let relative_end = lint.span.end - containing_para.start_offset;

            Some(FormattedLintOutput {
                start: relative_start,
                length: relative_end - relative_start,
                end: relative_end,
                paragraph_key: containing_para.key.to_string(),
                paragraph: containing_para.text.to_string(),
                string: linted_string,
                r#type: lint.lint_kind.to_string(),
                suggestions: Suggestions {
                    recommendation: lint.suggestions.iter().map(|s| s.to_string()).collect(),
                },
            })
        })
        .collect();

    lint_outputs.sort_by(|a, b| {
        let para_a = a.paragraph_key.parse::<usize>().unwrap_or(0);
        let para_b = b.paragraph_key.parse::<usize>().unwrap_or(0);

        match para_a.cmp(&para_b) {
            Ordering::Equal => a.start.cmp(&b.start),
            other => other,
        }
    });

    Ok(HttpResponse::Ok().json(lint_outputs))
}


// --- Add these lines to declare the new modules ---
pub mod ai;
pub mod comb;

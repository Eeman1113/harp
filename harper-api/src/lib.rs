// use actix_web::{web, HttpResponse, Result};
// use harper_core::{
//     linting::{Lint, LintGroup, LintKind, Linter},
//     Dialect, Document, FstDictionary,
// };
// use serde::{Deserialize, Serialize};
// use std::cmp::Ordering;
// use std::collections::HashSet;
// use std::ops::Range;

// // --- All of your existing structs and functions for the /lint endpoint remain here ---

// #[derive(Serialize, Deserialize)]
// pub struct Suggestions {
//     pub recommendation: Vec<String>,
// }

// #[derive(Serialize, Deserialize)]
// pub struct FormattedLintOutput {
//     pub start: usize,
//     pub length: usize,
//     pub end: usize,
//     #[serde(rename = "paragraphKey")]
//     pub paragraph_key: String,
//     pub paragraph: String,
//     pub string: String,
//     #[serde(rename = "type")]
//     pub r#type: String,
//     pub suggestions: Suggestions,
// }

// #[derive(Deserialize)]
// pub struct LintRequest {
//     pub text: String,
//     #[serde(default)]
//     pub ignore: Vec<String>,
// }

// struct Paragraph<'a> {
//     key: usize,
//     text: &'a str,
//     start_offset: usize,
// }

// #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
// enum LintKindPriority {
//     Miscellaneous,
//     Capitalization,
//     Spelling,
//     Style,
//     Repetition,
//     WordChoice,
// }

// fn get_priority(kind: &LintKind) -> LintKindPriority {
//     match kind {
//         LintKind::WordChoice => LintKindPriority::WordChoice,
//         LintKind::Repetition => LintKindPriority::Repetition,
//         LintKind::Style => LintKindPriority::Style,
//         LintKind::Spelling => LintKindPriority::Spelling,
//         LintKind::Capitalization => LintKindPriority::Capitalization,
//         _ => LintKindPriority::Miscellaneous,
//     }
// }

// pub async fn lint_text(request: web::Json<LintRequest>) -> Result<HttpResponse> {
//     let document = Document::new_plain_english_curated(&request.text);
//     let curated_text = document.get_source();
//     let dictionary = FstDictionary::curated();
//     let mut linter = LintGroup::new_curated(dictionary, Dialect::American);
//     let mut lints_from_linter = linter.lint(&document);
    
//     let ignore_set: HashSet<String> = request.ignore.iter().map(|s| s.to_lowercase()).collect();

//     lints_from_linter.sort_by(|a, b| get_priority(&b.lint_kind).cmp(&get_priority(&a.lint_kind)));

//     let mut final_lints: Vec<&Lint> = Vec::new();
//     let mut claimed_spans: Vec<Range<usize>> = Vec::new();

//     for lint in &lints_from_linter {
//         let lint_range = lint.span.start..lint.span.end;
//         let has_overlap = claimed_spans
//             .iter()
//             .any(|claimed| lint_range.start < claimed.end && lint_range.end > claimed.start);

//         if !has_overlap {
//             final_lints.push(lint);
//             claimed_spans.push(lint_range);
//         }
//     }

//     let curated_text_str: String = curated_text.iter().collect();
//     let mut paragraphs: Vec<Paragraph> = Vec::new();
//     let mut current_offset = 0;

//     for (key, line_str) in curated_text_str.lines().enumerate() {
//         let trimmed_line = line_str.trim();
//         if !trimmed_line.is_empty() {
//             let start_in_line = line_str.find(trimmed_line).unwrap_or(0);
//             let absolute_start_offset = current_offset + start_in_line;
            
//             paragraphs.push(Paragraph {
//                 key: key + 1,
//                 text: trimmed_line,
//                 start_offset: absolute_start_offset,
//             });
//         }
//         current_offset += line_str.chars().count() + 1;
//     }
    
//     let mut lint_outputs: Vec<FormattedLintOutput> = final_lints
//         .iter()
//         .filter_map(|lint| {
//             let linted_string = String::from_iter(curated_text.get(lint.span.start..lint.span.end)?);

//             if ignore_set.contains(&linted_string.to_lowercase()) {
//                 return None;
//             }
            
//             if lint.lint_kind == LintKind::Spelling && linted_string == "s" {
//                 return None;
//             }

//             let containing_para = paragraphs
//                 .iter()
//                 .find(|p| {
//                     let para_end_offset = p.start_offset + p.text.chars().count();
//                     lint.span.start >= p.start_offset && lint.span.end <= para_end_offset
//                 })?;

//             let relative_start = lint.span.start - containing_para.start_offset;
//             let relative_end = lint.span.end - containing_para.start_offset;

//             Some(FormattedLintOutput {
//                 start: relative_start,
//                 length: relative_end - relative_start,
//                 end: relative_end,
//                 paragraph_key: containing_para.key.to_string(),
//                 paragraph: containing_para.text.to_string(),
//                 string: linted_string,
//                 r#type: lint.lint_kind.to_string(),
//                 suggestions: Suggestions {
//                     recommendation: lint.suggestions.iter().map(|s| s.to_string()).collect(),
//                 },
//             })
//         })
//         .collect();

//     lint_outputs.sort_by(|a, b| {
//         let para_a = a.paragraph_key.parse::<usize>().unwrap_or(0);
//         let para_b = b.paragraph_key.parse::<usize>().unwrap_or(0);

//         match para_a.cmp(&para_b) {
//             Ordering::Equal => a.start.cmp(&b.start),
//             other => other,
//         }
//     });

//     Ok(HttpResponse::Ok().json(lint_outputs))
// }


// // --- Add these lines to declare the new modules ---
// pub mod ai;
// pub mod comb;
<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <title>harper js</title>

  <style>
    body { font-family: sans-serif; }
    mark.err { background: #ffcccc; border-bottom: 2px solid #c94a4a; padding: 0 2px; color: inherit; display: inline; }
    .editor-wrapper { position: relative; width: 100%; min-height: 200px; margin-bottom: 2rem; }
    .highlight-layer {
      position: absolute; top: 0; left: 0; right: 0; bottom: 0;
      white-space: pre-wrap; pointer-events: none; padding: 8px;
      color: transparent; overflow-wrap: break-word; z-index: 1;
      line-height: 1.5;
      font-family: sans-serif;
      font-size: 16px;
    }
    #maininput {
      position: relative; z-index: 2; background: transparent; color: black;
      width: 100%; min-height: 200px; padding: 8px; resize: none; overflow: hidden;
      line-height: 1.5;
      font-family: sans-serif;
      font-size: 16px;
    }
    .error-text { color: #c94a4a; font-weight: bold; }
    .error-message { color: #555; }
    .error-suggestion { color: #4CAF50; font-style: italic; }
  </style>

  <script type="module">
    import { WorkerLinter } from "https://cdn.jsdelivr.net/npm/harper.js@0.14.0/dist/harper.js";

    const harperConfig = {
      linters: {
        SentenceCapitalization: true,
        Spaces: true,
        SpellCheck: true,
        RepeatedWords: true,
        AnA: true,
        Matcher: true
      }
    };

    let linter = new WorkerLinter(harperConfig);

    function applyHighlights(text, lints) {
      let html = "";
      let lastIndex = 0;

      for (let l of lints) {
        const sp = l.span();
        const start = sp.start;
        const end = sp.end;

        html += escapeHTML(text.slice(lastIndex, start));
        html += `<mark class="err">${escapeHTML(text.slice(start, end))}</mark>`;
        lastIndex = end;
      }

      html += escapeHTML(text.slice(lastIndex));
      return html;
    }

    function escapeHTML(str) {
      return str.replace(/&/g, "&amp;")
                .replace(/</g, "&lt;")
                .replace(/>/g, "&gt;");
    }

    async function onInput(e) {
      const textToLint = e.target.value.replace(/\u00A0/g, " ");
      let lints = await linter.lint(textToLint);

      document.getElementById("highlight").innerHTML = applyHighlights(textToLint, lints);

      let list = document.getElementById("problemsList");
      list.innerHTML = "";

      for (let lint of lints) {
        const sp = lint.span();
        const bad = textToLint.substring(sp.start, sp.end);

        let li = document.createElement("li");

        let badSpan = document.createElement("span");
        badSpan.className = "error-text";
        badSpan.textContent = `"${bad}"`;
        li.appendChild(badSpan);

        let msgSpan = document.createElement("span");
        msgSpan.className = "error-message";
        msgSpan.textContent = ` — ${lint.message()}`;
        li.appendChild(msgSpan);

        if (lint.suggestion_count() > 0) {
          const suggestions = [...lint.suggestions()]
            .map(s => `‘${s.get_replacement_text()}’`)
            .join(", ");

          let sugSpan = document.createElement("span");
          sugSpan.className = "error-suggestion";
          sugSpan.textContent = ` Suggestion(s): ${suggestions}`;
          li.appendChild(sugSpan);
        }

        
        const card = document.createElement("div");
        card.style.border = "1px solid #ddd";
        card.style.borderRadius = "10px";
        card.style.padding = "12px";
        card.style.background = "#fff";
        card.style.boxShadow = "0 1px 2px rgba(0,0,0,0.05)";
        card.style.display = "flex";
        card.style.flexDirection = "column";
        card.innerHTML = `
          <div style='font-weight:600; margin-bottom:6px;'>${lint.linter_name()}</div>
          <div style='font-size:14px; color:#444;'>${lint.message()}</div>
          <div style='margin-top:8px; font-size:13px; color:#888;'>${escapeHTML(bad)}</div>
        `;
        list.appendChild(card);
      }
    }

    function autoResize() {
      const input = inputField;
      const ghost = highlight;
      input.style.height = "auto";
      input.style.height = input.scrollHeight + "px";
      ghost.style.height = input.scrollHeight + "px";
    }

    document.addEventListener("DOMContentLoaded", () => {
      const inputField = document.getElementById("maininput");
      const highlight = document.getElementById("highlight");

      function debounce(fn, delay) {
        let timer;
        return (...args) => {
          clearTimeout(timer);
          timer = setTimeout(() => fn(...args), delay);
        };
      }

      const debouncedInput = debounce(onInput, 120);

      inputField.addEventListener("input", e => { autoResize(); debouncedInput(e); });

      inputField.addEventListener("scroll", () => {
        highlight.scrollTop = inputField.scrollTop;
        highlight.scrollLeft = inputField.scrollLeft;
      });

      onInput({ target: inputField });
    });
  </script>
</head>
<body style="margin:0; background:#fafafa; font-family:Inter, sans-serif;">

<div style="display:flex; height:100vh; overflow:hidden;">
  <!-- LEFT SIDE: EDITOR CARD -->
  <div style="flex:2; padding:24px; overflow-y:auto;">
    <div style="background:white; border:1px solid #e5e5e5; border-radius:12px; padding:24px; box-shadow:0 1px 2px rgba(0,0,0,0.04);">
      <div class="editor-wrapper" style="min-height:500px;">
        <div id="highlight" class="highlight-layer"></div>
        <textarea id="maininput" style="width:100%; border:none; outline:none; font-size:16px; line-height:1.6; resize:none;"></textarea>
      </div>
    </div>
  </div>

  <!-- RIGHT SIDE: PROBLEMS PANEL -->
  <div style="flex:1; background:white; border-left:1px solid #e5e5e5; padding:24px; overflow-y:auto; box-shadow:-1px 0 3px rgba(0,0,0,0.04);">
    <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:16px;">
      <h2 style="margin:0; font-size:20px; font-weight:600;">Problems</h2>
      <div>
        <button style="background:#f3f3f3; border:1px solid #ddd; border-radius:6px; padding:6px 10px; margin-right:6px; cursor:pointer;">Open all</button>
        <button style="background:#f3f3f3; border:1px solid #ddd; border-radius:6px; padding:6px 10px; cursor:pointer;">Ignore all</button>
      </div>
    </div>

    <div id="problemsList" style="display:flex; flex-direction:column; gap:12px;"></div>
  </div>
</div>

<!-- FLOATING TOOLTIP → EXACT HARPER STYLE -->
<div id="lintTooltip" style="position:absolute; display:none; background:white; border:1px solid #e1e1e1; padding:16px; border-radius:10px; box-shadow:0 4px 12px rgba(0,0,0,0.12); font-size:14px; max-width:260px; z-index:1000;"></div>

</body>
</html>

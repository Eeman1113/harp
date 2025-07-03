//! Contains all logic for the AI-powered analysis endpoint.

use actix_web::{web, HttpResponse, Responder, error::ErrorInternalServerError};
use serde::{Deserialize, Serialize};
use std::env;
use strsim::levenshtein;

// --- Structs for OpenRouter API Interaction ---

#[derive(Serialize)]
struct OpenRouterRequest<'a> {
    model: &'a str,
    messages: Vec<Message<'a>>,
    response_format: ResponseFormat,
}

#[derive(Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Serialize)]
struct ResponseFormat {
    r#type: String,
}

#[derive(Deserialize)]
struct OpenRouterResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

// --- Structs for Parsing AI's JSON output ---

#[derive(Deserialize, Debug)]
struct AiAnalysisResults {
    analysis_results: Vec<AiSuggestion>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct AiSuggestion {
    paragraph_key: String,
    string: String,
    r#type: String,
    suggestions: AiSuggestionsContent,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct AiSuggestionsContent {
    recommendation: Vec<String>,
}


// --- Structs for the final API response ---

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AiFormattedOutput {
    pub start: usize,
    pub length: usize,
    pub end: usize,
    pub paragraph_key: String,
    pub string: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub suggestions: AiSuggestionsContent,
}


// --- Request Body for our /analyze endpoint ---

#[derive(Deserialize)]
pub struct AnalyzeRequest {
    pub text: String,
}

// --- Core Logic ---

/// Constructs the full prompt with instructions for the AI.
fn build_prompt(text_to_analyze: &str) -> String {
    format!(
        r#"
You are an expert AI writing assistant. Your task is to analyze the provided text and identify segments that could be improved for clarity, conciseness, impact, or tone. Go beyond simple grammar and spelling corrections. Provide suggestions that leverage your deep understanding of language, context, and nuance.

Your output **must** be a single JSON object with a key named "analysis_results". This key must contain an array of suggestion objects.
Do not include any other text, explanations, or markdown formatting before or after the JSON object.

Each suggestion object in the array must conform to the following structure. Note: You only need to generate the 'paragraphKey', 'string', 'type', and 'suggestions' fields. The other fields will be calculated by the system.
{{
    "start": "Integer, the starting character index of the string within the paragraph.",
    "length": "Integer, the length of the string.",
    "end": "Integer, the ending character index of the string.",
    "paragraphKey": "A string identifying the paragraph, e.g., '1'.",
    "string": "The exact string from the text that needs improvement.",
    "type": "A string describing the category of the issue (e.g., 'Clarity', 'Conciseness', 'Wordiness', 'Passive Voice', 'Impact').",
    "suggestions": {{
        "recommendation": [
            "An array of strings, with each string being a concrete suggestion for replacement."
        ]
    }}
}}

-----

**Example:**

**[TEXT TO ANALYZE]**

(1) Our team is engaged in the process of leveraging our core competencies to synergize our efforts and achieve our goals. We will endeavor to finalize the report.

**[ANALYSIS_JSON]**

```json
{{
    "analysis_results": [
        {{
            "paragraphKey": "1",
            "string": "is engaged in the process of leveraging our core competencies to synergize",
            "type": "Wordiness & Corporate Jargon",
            "suggestions": {{
                "recommendation": [
                    "is using its strengths to combine",
                    "is focusing its key skills to improve",
                    "is using our strengths to coordinate"
                ]
            }}
        }},
        {{
            "paragraphKey": "1",
            "string": "We will endeavor to finalize",
            "type": "Clarity & Conciseness",
            "suggestions": {{
                "recommendation": [
                    "We will finalize",
                    "We will finish"
                ]
            }}
        }}
    ]
}}
```

-----

**Your Turn:**

**[TEXT TO ANALYZE]**

{text_to_analyze}

**[ANALYSIS_JSON]**
"#
    )
}

/// Splits text into paragraphs and formats it with paragraph keys for the AI.
fn prepare_text_for_analysis(raw_text: &str) -> String {
    raw_text
        .trim()
        .split("\n\n")
        .filter(|p| !p.trim().is_empty())
        .enumerate()
        .map(|(i, p)| format!("({}) {}", i + 1, p.trim()))
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Uses a sliding window and Levenshtein distance to find the best fuzzy match
/// for the AI's suggested string within the original paragraphs.
fn find_and_update_indices(
    analysis_array: Vec<AiSuggestion>,
    original_paragraphs: Vec<&str>,
) -> Vec<AiFormattedOutput> {
    let mut updated_results = Vec::new();

    for suggestion in &analysis_array {
        if let Ok(para_index) = suggestion.paragraph_key.parse::<usize>() {
            // paragraphKey is 1-based, vec index is 0-based
            if let Some(paragraph_text) = original_paragraphs.get(para_index - 1) {
                let string_to_find = &suggestion.string;
                if string_to_find.is_empty() { continue; }

                let string_to_find_len = string_to_find.chars().count();
                let paragraph_len = paragraph_text.chars().count();

                if string_to_find_len > paragraph_len { continue; }

                let mut best_match_start_index = 0;
                let mut min_distance = usize::MAX;

                // Iterate through all possible substrings of similar length using a sliding window.
                for i in 0..=(paragraph_len - string_to_find_len) {
                    let sub: String = paragraph_text.chars().skip(i).take(string_to_find_len).collect();
                    let distance = levenshtein(&sub, string_to_find);

                    if distance < min_distance {
                        min_distance = distance;
                        best_match_start_index = i;
                    }

                    // If a perfect match is found, we can stop searching in this paragraph.
                    if distance == 0 {
                        break;
                    }
                }

                // Normalize the distance to a ratio. A lower ratio means a better match.
                // We accept matches where less than 20% of characters are different.
                let match_threshold = 0.2;
                let normalized_distance = min_distance as f32 / string_to_find_len as f32;

                if normalized_distance <= match_threshold {
                    let start_index = best_match_start_index;
                    let length = string_to_find_len;
                    let end_index = start_index + length;

                    let matched_string: String = paragraph_text.chars().skip(start_index).take(length).collect();

                    let formatted_suggestion = AiFormattedOutput {
                        start: start_index,
                        length,
                        end: end_index,
                        paragraph_key: suggestion.paragraph_key.clone(),
                        string: matched_string,
                        r#type: suggestion.r#type.clone(),
                        suggestions: suggestion.suggestions.clone(),
                    };
                    updated_results.push(formatted_suggestion);
                } else {
                     eprintln!(
                        "Warning: Could not find a good match for string in paragraph {}. Best match distance: {:.2}. String: '{}'",
                        suggestion.paragraph_key, normalized_distance, suggestion.string
                    );
                }
            } else {
                eprintln!("Warning: Invalid paragraphKey '{}' in suggestion.", suggestion.paragraph_key);
            }
        }
    }

    updated_results
}


/// The main handler for the /analyze endpoint.
pub async fn analyze_text(request: web::Json<AnalyzeRequest>) -> actix_web::Result<impl Responder> {
    let api_key = env::var("OPENROUTER_API_KEY")
        .map_err(|_| ErrorInternalServerError("OPENROUTER_API_KEY environment variable not set."))?;

    if api_key.is_empty() || api_key == "YOUR_OPENROUTER_API_KEY" {
        return Err(ErrorInternalServerError("OpenRouter API key is not configured. Please set it in the .env file."));
    }

    // 1. Get a clean list of the original paragraphs.
    let original_paragraphs: Vec<&str> = request.text
        .trim()
        .split("\n\n")
        .filter(|p| !p.trim().is_empty())
        .map(|p| p.trim())
        .collect();

    // 2. Format the text for the AI.
    let formatted_text = prepare_text_for_analysis(&request.text);

    // 3. Build the full prompt.
    let full_prompt = build_prompt(&formatted_text);

    // 4. Call the OpenRouter API.
    let client = reqwest::Client::new();
    let payload = OpenRouterRequest {
        model: "google/gemini-flash-1.5",
        messages: vec![Message {
            role: "user",
            content: &full_prompt,
        }],
        response_format: ResponseFormat {
            r#type: "json_object".to_string(),
        },
    };

    let res = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&payload)
        .send()
        .await
        .map_err(ErrorInternalServerError)?;

    if !res.status().is_success() {
        let error_body = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(ErrorInternalServerError(format!("API Error: {}", error_body)));
    }

    let response_data = res
        .json::<OpenRouterResponse>()
        .await
        .map_err(ErrorInternalServerError)?;

    // 5. Parse the AI's response.
    let analysis_content_string = response_data
        .choices
        .get(0)
        .ok_or_else(|| ErrorInternalServerError("API response was empty."))?
        .message
        .content
        .clone();

    // The AI might wrap the JSON in markdown ```json ... ```, so we find the first '{' and last '}'
    let json_start = analysis_content_string.find('{');
    let json_end = analysis_content_string.rfind('}');

    let parsed_json: AiAnalysisResults = match (json_start, json_end) {
        (Some(start), Some(end)) if end > start => {
            let json_str = &analysis_content_string[start..=end];
            serde_json::from_str(json_str)
                .map_err(|e| ErrorInternalServerError(format!("Failed to parse AI JSON response: {}", e)))?
        }
        _ => return Err(ErrorInternalServerError("No valid JSON object found in AI response.")),
    };

    // 6. Find strings in original text and add correct indices.
    let final_results = find_and_update_indices(parsed_json.analysis_results, original_paragraphs);

    // 7. Return the final, corrected JSON result.
    Ok(HttpResponse::Ok().json(final_results))
}

//! Contains the logic for the combined lint and AI analysis endpoint.

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use crate::{
    lint_text,
    ai::{analyze_text, AiFormattedOutput},
    FormattedLintOutput,
    LintRequest,
    ai::AnalyzeRequest
};


// --- Structs for the combined endpoint ---

#[derive(Deserialize)]
pub struct CombinedAnalysisRequest {
    pub text: String,
}

#[derive(Serialize)]
pub struct CombinedAnalysisResponse {
    lint_results: Vec<FormattedLintOutput>,
    ai_results: Vec<AiFormattedOutput>,
}

/// The main handler for the /combined endpoint.
/// It runs both the local linter and the AI analysis concurrently.
pub async fn combined_analysis(request: web::Json<CombinedAnalysisRequest>) -> impl Responder {
    // We need to create separate requests for the existing handlers.
    let lint_req = web::Json(LintRequest {
        text: request.text.clone(),
        ignore: vec![],
    });

    let ai_req = web::Json(AnalyzeRequest {
        text: request.text.clone(),
    });

    // Run both the local linting and the AI analysis concurrently.
    let (lint_handle, ai_handle) = tokio::join!(
        lint_text(lint_req),
        analyze_text(ai_req)
    );

    // Extract the JSON body from the linting response.
    let lint_results: Vec<FormattedLintOutput> = match lint_handle {
        Ok(res) => {
            let body = res.into_body();
            match actix_web::body::to_bytes(body).await {
                Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
                Err(_) => Vec::new(),
            }
        },
        Err(e) => {
            eprintln!("Lint analysis failed: {:?}", e);
            Vec::new()
        }
    };

    // Extract the JSON body from the AI analysis response.
    let ai_results: Vec<AiFormattedOutput> = match ai_handle {
        Ok(Ok(res)) => {
            let body = res.into_body();
            match actix_web::body::to_bytes(body).await {
                Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
                Err(_) => Vec::new(),
            }
        },
        Ok(Err(e)) => {
            eprintln!("AI analysis failed (inner error): {:?}", e);
            Vec::new()
        },
        Err(e) => {
            eprintln!("AI analysis failed (join error): {:?}", e);
            Vec::new()
        }
    };

    let combined_response = CombinedAnalysisResponse {
        lint_results,
        ai_results,
    };

    HttpResponse::Ok().json(combined_response)
}

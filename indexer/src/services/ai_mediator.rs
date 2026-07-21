//! AI Mediator — calls OpenAI/Anthropic to analyze dispute evidence
//! and propose a fair outcome before jury empanelment.

use crate::services::escrow_service::ServiceError;
use crate::types::*;

pub struct AiMediator {
    api_key: String,
    model: String,
    http_client: reqwest::Client,
}

impl AiMediator {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to build HTTP client for AiMediator"),
        }
    }

    /// Generate a mediation recommendation based on dispute evidence.
    /// Returns a structured result or an error if the API call fails.
    pub async fn mediate(
        &self,
        messages: &[MediationMessage],
        buyer_claim: &str,
        seller_claim: &str,
        escrow_amount: i64,
    ) -> Result<MediationResult, ServiceError> {
        let prompt = self.build_prompt(messages, buyer_claim, seller_claim, escrow_amount);

        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": include_str!("ai_mediator_prompt.txt")},
                {"role": "user", "content": prompt}
            ],
            "response_format": {"type": "json_object"},
            "temperature": 0.3,
            "max_tokens": 1024,
        });

        // Uses DeepSeek V4 Flash (OpenAI-compatible API)
        let response = self
            .http_client
            .post("https://api.deepseek.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| ServiceError::Internal(format!("AI mediator API request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(ServiceError::Internal(format!(
                "AI mediator API returned {status}: {text}"
            )));
        }

        let response_body: serde_json::Value = response.json().await.map_err(|e| {
            ServiceError::Internal(format!("Failed to parse AI mediator response: {e}"))
        })?;

        let content = response_body["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| ServiceError::Internal("AI mediator response missing content".into()))?;

        let result: MediationResult = serde_json::from_str(content).map_err(|e| {
            ServiceError::Internal(format!(
                "Failed to parse AI mediation result from JSON: {e}. Raw: {content}"
            ))
        })?;

        // Validate the result
        if matches!(result.outcome, MediationOutcome::Split)
            && (result.buyer_share_basis < 0 || result.buyer_share_basis > 10_000)
        {
            return Err(ServiceError::Internal(
                "AI returned invalid split share (must be 0-10000)".into(),
            ));
        }

        Ok(result)
    }

    fn build_prompt(
        &self,
        messages: &[MediationMessage],
        buyer_claim: &str,
        seller_claim: &str,
        escrow_amount: i64,
    ) -> String {
        let amount_kas = escrow_amount as f64 / 100_000_000.0;
        let mut prompt = format!("## Dispute Details\n\nEscrow amount: {amount_kas} KAS\n\n");

        prompt.push_str("## Buyer's Claim\n\n");
        prompt.push_str(buyer_claim);
        prompt.push_str("\n\n## Seller's Claim\n\n");
        prompt.push_str(seller_claim);

        if !messages.is_empty() {
            prompt.push_str("\n\n## Chat History\n\n");
            for msg in messages {
                let ts = chrono::DateTime::from_timestamp(msg.timestamp, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M UTC").to_string())
                    .unwrap_or_default();
                prompt.push_str(&format!("[{ts}] {}: {}\n", msg.role, msg.content));
            }
        }

        prompt
    }
}

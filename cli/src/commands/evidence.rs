//! Evidence command — list evidence for an escrow.

use anyhow::Result;

/// List evidence for an escrow.
pub async fn list_evidence(api_url: String, id: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/v1/escrows/{}/evidence", api_url, id))
        .send()
        .await?;

    if !resp.status().is_success() {
        anyhow::bail!("Failed to fetch evidence for escrow {}", id);
    }

    let data: serde_json::Value = resp.json().await?;
    let evidence = data["evidence"].as_array().cloned().unwrap_or_default();

    if evidence.is_empty() {
        println!("No evidence for escrow {}", id);
        return Ok(());
    }

    println!("Evidence for escrow {}:", id);
    for ev in &evidence {
        let by = ev["submitted_by"].as_str().unwrap_or("?");
        let at = ev["created_at"].as_i64().unwrap_or(0);
        let content = ev["content"].as_str().unwrap_or("");
        println!(
            "  [{}] by {}: {}",
            at,
            &by[..by.len().min(20)],
            &content[..content.len().min(100)]
        );
    }
    Ok(())
}

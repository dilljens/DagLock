//! Atomic swap command — settle an escrow by submitting a preimage.

use anyhow::Result;

/// Submit a preimage to atomically swap an escrow.
pub async fn run(api_url: String, id: &str, preimage: &str) -> Result<()> {
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{}/v1/escrows/{}/swap", api_url, id))
        .json(&serde_json::json!({ "preimage": preimage }))
        .send()
        .await?;

    if !resp.status().is_success() {
        let err: serde_json::Value = resp.json().await?;
        anyhow::bail!("Swap failed: {}", err);
    }

    let result: serde_json::Value = resp.json().await?;
    println!(
        "✅ Atomic swap settled: {}",
        result["escrow_id"].as_str().unwrap_or(id)
    );
    println!(
        "   Method: {}",
        result["method"].as_str().unwrap_or("atomic_swap")
    );
    Ok(())
}

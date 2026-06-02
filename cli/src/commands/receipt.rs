//! Receipt command.

use anyhow::Result;

pub async fn run(api_url: String, id: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/v1/receipts/{}", api_url, id))
        .send()
        .await?;

    if !resp.status().is_success() {
        let err: serde_json::Value = resp.json().await?;
        anyhow::bail!("Receipt not found: {}", err);
    }

    let receipt: serde_json::Value = resp.json().await?;
    println!(
        "🧾 Receipt: {}",
        receipt["receipt_id"].as_str().unwrap_or("?")
    );
    println!(
        "   Escrow: {}",
        receipt["escrow_id"].as_str().unwrap_or("?")
    );
    println!("   Status: {}", receipt["status"].as_str().unwrap_or("?"));
    println!("   Asset:  {}", receipt["asset"].as_str().unwrap_or("?"));
    println!(
        "   Amount: {} units",
        receipt["amount_sompi"].as_i64().unwrap_or(0)
    );
    println!("   Disputed: {}", receipt["disputed_at"].as_i64().is_some());
    if let Some(reason) = receipt["dispute_reason"].as_str() {
        println!("   Reason:  {}", reason);
    }

    Ok(())
}

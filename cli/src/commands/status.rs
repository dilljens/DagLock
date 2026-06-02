//! Status command — check escrow lifecycle.

use anyhow::Result;

pub async fn run(api_url: String, id: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/v1/escrows/{}", api_url, id))
        .send()
        .await?;

    if !resp.status().is_success() {
        anyhow::bail!("Escrow not found: {}", id);
    }

    let e: serde_json::Value = resp.json().await?;
    let created = chrono::DateTime::from_timestamp(e["created_at"].as_i64().unwrap_or(0), 0)
        .map(|t| t.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| "unknown".into());

    println!("📋 Escrow: {}", id);
    println!(
        "   Status:     {}",
        e["status"].as_str().unwrap_or("unknown")
    );
    println!(
        "   Buyer:      {}",
        e["buyer_address"].as_str().unwrap_or("unknown")
    );
    println!(
        "   Seller:     {}",
        e["seller_address"].as_str().or(Some("—")).unwrap()
    );
    println!(
        "   Amount:     {} KAS",
        e["amount_sompi"].as_i64().unwrap_or(0) as f64 / 100_000_000.0
    );
    println!(
        "   Fee:        {} KAS",
        e["fee_sompi"].as_i64().unwrap_or(0) as f64 / 100_000_000.0
    );
    println!("   Created:    {}", created);
    println!("   Disputed:   {}", e["disputed_at"].as_i64().is_some());
    println!("   Cancelled:  {}", e["cancelled_at"].as_i64().is_some());
    if let Some(reason) = e["dispute_reason"].as_str() {
        println!("   Reason:     {}", reason);
    }

    Ok(())
}

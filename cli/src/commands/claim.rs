//! Claim and refund commands.

use anyhow::Result;

/// Claim/release an escrow as the seller.
pub async fn run(api_url: String, id: &str) -> Result<()> {
    let client = reqwest::Client::new();

    // Fetch escrow details
    let resp = client
        .get(format!("{}/v1/escrows/{}", api_url, id))
        .send()
        .await?;

    if !resp.status().is_success() {
        anyhow::bail!("Escrow not found: {}", id);
    }

    let escrow: serde_json::Value = resp.json().await?;
    println!("📋 Escrow: {}", id);
    println!(
        "   Buyer:    {}",
        escrow["buyer_address"].as_str().unwrap_or("unknown")
    );
    println!(
        "   Amount:   {} KAS",
        escrow["amount_sompi"].as_i64().unwrap_or(0) as f64 / 100_000_000.0
    );
    println!(
        "   Status:   {}",
        escrow["status"].as_str().unwrap_or("unknown")
    );
    println!();

    let settle = client
        .post(format!("{}/v1/escrows/{}/settle", api_url, id))
        .send()
        .await?;
    if !settle.status().is_success() {
        let err: serde_json::Value = settle.json().await?;
        anyhow::bail!("Settle failed: {}", err);
    }

    let receipt = client
        .get(format!("{}/v1/receipts/{}", api_url, id))
        .send()
        .await?;
    let receipt: serde_json::Value = receipt.json().await?;

    println!("✅ Escrow settled");
    println!(
        "   Receipt: {}",
        receipt["receipt_id"].as_str().unwrap_or("?")
    );
    println!(
        "   Seller:  {}",
        receipt["seller_address"].as_str().unwrap_or("unknown")
    );

    Ok(())
}

/// Mark an escrow as disputed.
pub async fn run_dispute(api_url: String, id: &str, reason: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/v1/escrows/{}/dispute", api_url, id))
        .json(&serde_json::json!({ "reason": reason }))
        .send()
        .await?;

    if !resp.status().is_success() {
        let err: serde_json::Value = resp.json().await?;
        anyhow::bail!("Dispute failed: {}", err);
    }

    let status: serde_json::Value = resp.json().await?;
    println!(
        "⚠️  Escrow disputed: {}",
        status["escrow_id"].as_str().unwrap_or(id)
    );
    println!("   Reason: {}", reason);
    Ok(())
}

/// Cancel an escrow before completion.
pub async fn run_cancel(api_url: String, id: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/v1/escrows/{}/cancel", api_url, id))
        .send()
        .await?;

    if !resp.status().is_success() {
        let err: serde_json::Value = resp.json().await?;
        anyhow::bail!("Cancel failed: {}", err);
    }

    let status: serde_json::Value = resp.json().await?;
    println!(
        "🛑 Escrow cancelled: {}",
        status["escrow_id"].as_str().unwrap_or(id)
    );
    Ok(())
}

/// Refund an escrow as the buyer (after timeout).
pub async fn run_refund(api_url: String, id: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/v1/escrows/{}", api_url, id))
        .send()
        .await?;

    if !resp.status().is_success() {
        anyhow::bail!("Escrow not found: {}", id);
    }

    let escrow: serde_json::Value = resp.json().await?;
    println!("📋 Escrow: {}", id);
    println!(
        "   Status:   {}",
        escrow["status"].as_str().unwrap_or("unknown")
    );
    println!();
    let refund = client
        .post(format!("{}/v1/escrows/{}/refund", api_url, id))
        .send()
        .await?;
    if !refund.status().is_success() {
        let err: serde_json::Value = refund.json().await?;
        anyhow::bail!("Refund failed: {}", err);
    }

    let receipt = client
        .get(format!("{}/v1/receipts/{}", api_url, id))
        .send()
        .await?;
    let receipt: serde_json::Value = receipt.json().await?;

    println!("✅ Escrow refunded");
    println!(
        "   Receipt: {}",
        receipt["receipt_id"].as_str().unwrap_or("?")
    );

    Ok(())
}

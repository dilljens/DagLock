//! Reputation command — check counterparty stats.

use anyhow::Result;

pub async fn run(api_url: String, address: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/v1/reputation/{}", api_url, address))
        .send()
        .await?;

    let rep: serde_json::Value = resp.json().await?;

    println!("📊 Reputation: {}", address);
    println!(
        "   Trades:     {}",
        rep["trade_count"].as_i64().unwrap_or(0)
    );
    println!(
        "   Settled:    {}",
        rep["settled_count"].as_i64().unwrap_or(0)
    );
    println!(
        "   Refunded:   {}",
        rep["refunded_count"].as_i64().unwrap_or(0)
    );
    println!(
        "   Disputed:   {}",
        rep["disputed_count"].as_i64().unwrap_or(0)
    );
    println!(
        "   Age:        {} days",
        rep["age_days"].as_i64().unwrap_or(0)
    );
    println!(
        "   Dispute %:  {:.1}%",
        rep["dispute_rate"].as_f64().unwrap_or(0.0) * 100.0
    );
    println!(
        "   Refund %:   {:.1}%",
        rep["refund_rate"].as_f64().unwrap_or(0.0) * 100.0
    );
    println!(
        "   Score:      {:.2}/5",
        rep["score"].as_f64().unwrap_or(1.0)
    );
    println!(
        "   Volume:     {} KAS",
        rep["total_volume_sompi"].as_i64().unwrap_or(0) as f64 / 100_000_000.0
    );

    Ok(())
}

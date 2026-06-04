//! Offer board commands.

use anyhow::Result;

pub async fn list(api_url: String) -> Result<()> {
    let client = reqwest::Client::new();
    let resp = client.get(format!("{}/v1/offers", api_url)).send().await?;

    let data: serde_json::Value = resp.json().await?;
    let offers = data["offers"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    if offers.is_empty() {
        println!("📭 No open offers");
        return Ok(());
    }

    println!("📋 Open offers:");
    println!("{:-<80}", "");
    for offer in offers {
        println!(
            "  {} | {} {} {} for {} | creator: {}",
            offer["id"].as_str().unwrap_or("?"),
            offer["side"].as_str().unwrap_or("?"),
            offer["amount_sompi"].as_i64().unwrap_or(0) as f64 / 100_000_000.0,
            offer["base_asset"].as_str().unwrap_or("?"),
            offer["quote_asset"].as_str().unwrap_or("?"),
            &offer["creator_address"].as_str().unwrap_or("?")[..12],
        );
    }

    println!();
    println!("💡 Accept: daglock-cli offer accept <id> --address <your-address>");

    Ok(())
}

pub async fn create(
    api_url: String,
    side: &str,
    base: &str,
    quote: &str,
    amount_str: &str,
) -> Result<()> {
    let amount_sompi = crate::tx::kas_to_sompi(amount_str)?;
    let creator = format!("kaspa:user{:x}", rand::random::<u32>());

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/v1/offers", api_url))
        .json(&serde_json::json!({
            "creator_address": creator,
            "side": side,
            "base_asset": base,
            "quote_asset": quote,
            "amount_sompi": amount_sompi,
        }))
        .send()
        .await?;

    if resp.status().is_success() {
        let offer: serde_json::Value = resp.json().await?;
        println!("✅ Offer created: {}", offer["id"].as_str().unwrap_or("?"));
        println!("   {} {} {} for {}", side, amount_str, base, quote);
    } else {
        let err: serde_json::Value = resp.json().await?;
        anyhow::bail!("Error: {}", err);
    }

    Ok(())
}

pub async fn accept(api_url: String, id: &str, address: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/v1/offers/{}/accept", api_url, id))
        .json(&serde_json::json!({
            "counterparty_address": address,
        }))
        .send()
        .await?;

    if resp.status().is_success() {
        println!("✅ Offer {} accepted", id);
        println!("   Counterparty: {}", address);
        println!("💡 Next: daglock-cli create --amount ... --counterparty ...");
    } else {
        let err: serde_json::Value = resp.json().await?;
        anyhow::bail!(
            "Error: {}",
            err["error"]["message"].as_str().unwrap_or("unknown")
        );
    }

    Ok(())
}

pub async fn cancel(api_url: String, id: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/v1/offers/{}/cancel", api_url, id))
        .send()
        .await?;

    if resp.status().is_success() {
        println!("✅ Offer {} cancelled", id);
    } else {
        let err: serde_json::Value = resp.json().await?;
        anyhow::bail!(
            "Error: {}",
            err["error"]["message"].as_str().unwrap_or("unknown")
        );
    }

    Ok(())
}

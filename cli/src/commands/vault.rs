//! Vault commands — create, list, get, withdraw time-locked vaults.

use crate::tx::kas_to_sompi;
use anyhow::Result;

/// Create a new time-locked vault.
pub async fn create(api_url: String, address: &str, amount: &str, timeout: u64) -> Result<()> {
    let client = reqwest::Client::new();
    let amount_sompi = kas_to_sompi(amount)?;

    let resp = client
        .post(format!("{}/v1/vaults", api_url))
        .json(&serde_json::json!({
            "owner_address": address,
            "vault_type": "time",
            "amount_sompi": amount_sompi,
            "timeout": timeout,
        }))
        .send()
        .await?;

    if !resp.status().is_success() {
        let err: serde_json::Value = resp.json().await?;
        anyhow::bail!("Vault creation failed: {}", err);
    }

    let vault: serde_json::Value = resp.json().await?;
    println!("✅ Vault created!");
    println!("   ID:      {}", vault["id"].as_str().unwrap_or("?"));
    println!("   Amount:  {} KAS", amount);
    println!(
        "   Status:  {}",
        vault["status"].as_str().unwrap_or("locked")
    );
    Ok(())
}

/// List vaults by owner address.
pub async fn list(api_url: String, address: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!(
            "{}/v1/vaults?owner={}",
            api_url,
            url_encoded(address)
        ))
        .send()
        .await?;

    if !resp.status().is_success() {
        anyhow::bail!("Failed to list vaults");
    }

    let data: serde_json::Value = resp.json().await?;
    let vaults = data["vaults"].as_array().cloned().unwrap_or_default();

    if vaults.is_empty() {
        println!("📭 No vaults found for this address.");
        return Ok(());
    }

    println!("📋 Vaults:");
    for v in &vaults {
        let amount = v["amount_sompi"].as_i64().unwrap_or(0) as f64 / 100_000_000.0;
        println!(
            "   {} — {} KAS [{}]",
            v["id"].as_str().unwrap_or("?"),
            amount,
            v["status"].as_str().unwrap_or("unknown")
        );
    }
    println!(
        "\nTotal: {} vault(s)",
        data["total"].as_i64().unwrap_or(vaults.len() as i64)
    );
    Ok(())
}

/// Get vault details.
pub async fn get(api_url: String, id: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/v1/vaults/{}", api_url, id))
        .send()
        .await?;

    if !resp.status().is_success() {
        anyhow::bail!("Vault not found: {}", id);
    }

    let vault: serde_json::Value = resp.json().await?;
    let amount = vault["amount_sompi"].as_i64().unwrap_or(0) as f64 / 100_000_000.0;
    println!("📋 Vault: {}", id);
    println!(
        "   Owner:  {}",
        vault["owner_address"].as_str().unwrap_or("?")
    );
    println!(
        "   Type:   {}",
        vault["vault_type"].as_str().unwrap_or("time")
    );
    println!("   Status: {}", vault["status"].as_str().unwrap_or("?"));
    println!("   Amount: {} KAS", amount);
    println!("   Timeout: {}", vault["timeout"].as_i64().unwrap_or(0));
    Ok(())
}

/// Withdraw from a vault.
pub async fn withdraw(api_url: String, id: &str, address: &str, signature: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/v1/vaults/{}/withdraw", api_url, id))
        .json(&serde_json::json!({
            "owner_address": address,
            "signature": signature,
        }))
        .send()
        .await?;

    if !resp.status().is_success() {
        let err: serde_json::Value = resp.json().await?;
        anyhow::bail!("Withdraw failed: {}", err);
    }

    let result: serde_json::Value = resp.json().await?;
    println!(
        "✅ Vault withdrawn: {}",
        result["vault_id"].as_str().unwrap_or(id)
    );
    Ok(())
}

fn url_encoded(s: &str) -> String {
    // Simple URL encoding — reqwest handles most cases, but we need to encode :// chars
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'-' | b'.' | b'_' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

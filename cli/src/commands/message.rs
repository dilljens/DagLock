//! Message command — send/list messages on escrow threads.

use anyhow::Result;

pub async fn send(
    api_url: String,
    escrow_id: &str,
    text: &str,
    address: &str,
    signature: &str,
) -> Result<()> {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/v1/escrows/{}/messages", api_url, escrow_id))
        .header("X-Daglock-Address", address)
        .header("X-Daglock-Signature", signature)
        .header("X-Daglock-Message", &format!("msg:{}", escrow_id))
        .json(&serde_json::json!({ "content": text }))
        .send()
        .await?;

    if !resp.status().is_success() {
        let err: serde_json::Value = resp.json().await?;
        anyhow::bail!("Send failed: {}", err);
    }

    println!("✅ Message sent to escrow {escrow_id}");
    Ok(())
}

pub async fn list(
    api_url: String,
    escrow_id: &str,
    address: &str,
    signature: &str,
) -> Result<()> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/v1/escrows/{}/messages", api_url, escrow_id))
        .header("X-Daglock-Address", address)
        .header("X-Daglock-Signature", signature)
        .header("X-Daglock-Message", &format!("list:{}", escrow_id))
        .send()
        .await?;

    if !resp.status().is_success() {
        let err: serde_json::Value = resp.json().await?;
        anyhow::bail!("List failed: {}", err);
    }

    let data: serde_json::Value = resp.json().await?;
    let empty: &Vec<serde_json::Value> = &vec![];
    let messages = data["messages"].as_array().unwrap_or(empty);

    if messages.is_empty() {
        println!("📭 No messages on escrow {escrow_id}");
    } else {
        println!("📬 Messages on escrow {escrow_id}:");
        for msg in messages {
            let sender = msg["sender_address"].as_str().unwrap_or("?");
            let content = msg["content"].as_str().unwrap_or("?");
            let ts = msg["created_at"].as_i64().unwrap_or(0);
            let time = chrono::DateTime::from_timestamp(ts, 0)
                .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "?".to_string());
            println!("  [{time}] {sender:.20}…: {content}");
        }
    }

    Ok(())
}

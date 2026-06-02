//! Create escrow command.

use crate::config::Config;
use crate::tx::assemble_create_escrow;
use anyhow::Result;

pub async fn run(
    api_url: String,
    amount_str: &str,
    counterparty: &str,
    timeout: u64,
    treasury: Option<String>,
    _trade_hash: Option<String>,
) -> Result<()> {
    let _cfg = Config::load();
    let amount_sompi = crate::tx::kas_to_sompi(amount_str)?;

    // TODO: In production, keys would come from kaspawallet or config
    // For now, generate demo keys and print instructions
    let buyer_key = [1u8; 32];
    let seller_key = [2u8; 32];
    let treasury_key = if let Some(t) = &treasury {
        // Parse hex treasury key — simplified in v1
        if t.len() == 64 {
            hex::decode(t)?
        } else {
            vec![3u8; 32]
        }
    } else {
        vec![3u8; 32]
    };
    let treasury_arr: [u8; 32] = treasury_key[..32].try_into().unwrap_or([3u8; 32]);

    let result = assemble_create_escrow(
        &buyer_key,
        &seller_key,
        amount_sompi,
        timeout,
        &treasury_arr,
    )?;

    let fee_preview: serde_json::Value = reqwest::Client::new()
        .get(format!(
            "{}/v1/fees/estimate?amount_kas={}",
            api_url, amount_str
        ))
        .send()
        .await?
        .json()
        .await?;

    // Register with indexer
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/v1/escrows", api_url))
        .json(&serde_json::json!({
            "lock_tx_id": result.unsigned_tx_hex.chars().take(16).collect::<String>(),
            "lock_tx_output_index": 0,
            "buyer_address": counterparty,
            "amount_sompi": result.amount_sompi,
        }))
        .send()
        .await?;

    if resp.status().is_success() {
        let escrow: serde_json::Value = resp.json().await?;
        let fee_display = fee_preview["fee_kas"]
            .as_str()
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("{:.8}", result.fee_sompi as f64 / 100_000_000.0));
        println!("✅ Escrow created!");
        println!("   ID:       {}", escrow["id"].as_str().unwrap_or("?"));
        println!("   Amount:   {} KAS (fee: {} KAS)", amount_str, fee_display);
        println!("   Status:   pending_confirmation");
        println!();
        println!("📋 To broadcast this escrow, sign the transaction with:");
        println!(
            "   kaspawallet sign --transaction {}",
            result.unsigned_tx_hex
        );
        println!();
        println!("🔗 Trade link (send to counterparty):");
        println!(
            "   https://daglock.com/claim/{}",
            escrow["id"].as_str().unwrap_or(&result.escrow_id)
        );
    } else {
        let err: serde_json::Value = resp.json().await?;
        anyhow::bail!("Indexer error: {}", err);
    }

    Ok(())
}

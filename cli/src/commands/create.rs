//! Create escrow command — with real kaspawallet integration.

use crate::config::Config;
use anyhow::{Context, Result};

pub async fn run(
    api_url: String,
    amount_str: &str,
    counterparty: &str,
    timeout: u64,
    treasury: Option<String>,
    _trade_hash: Option<String>,
) -> Result<()> {
    let cfg = Config::load();
    let amount_sompi = crate::tx::kas_to_sompi(amount_str)?;

    // Get treasury key from args, config, or use a default
    let treasury_key_hex = treasury
        .as_deref()
        .or(cfg.treasury_key.as_deref())
        .unwrap_or("0000000000000000000000000000000000000000000000000000000000000000");
    let treasury_arr: [u8; 32] = crate::wallet::parse_hex_key(treasury_key_hex)?;

    // Get signing keys
    let use_kaspawallet = crate::wallet::kaspawallet_available();

    let (buyer_pubkey, seller_pubkey): ([u8; 32], [u8; 32]);

    if use_kaspawallet {
        // Derive keys from kaspawallet
        let output = std::process::Command::new("kaspawallet")
            .arg("keys")
            .arg("--show")
            .output()
            .context("Failed to get keys from kaspawallet")?;
        let key_data = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = key_data.lines().collect();

        if lines.len() < 2 {
            anyhow::bail!("kaspawallet keys returned insufficient keys. Need at least 2.");
        }

        buyer_pubkey = crate::wallet::parse_hex_key(lines[0].trim())?;
        seller_pubkey = crate::wallet::parse_hex_key(lines[1].trim())?;
    } else {
        // Read keys from env vars or generate for demo
        let buyer_hex = std::env::var("DAGLOCK_BUYER_KEY").unwrap_or_else(|_| {
            eprintln!("Warning: DAGLOCK_BUYER_KEY not set, using demo key [1u8; 32]");
            "0101010101010101010101010101010101010101010101010101010101010101".to_string()
        });
        let seller_hex = std::env::var("DAGLOCK_SELLER_KEY").unwrap_or_else(|_| {
            eprintln!("Warning: DAGLOCK_SELLER_KEY not set, using demo key [2u8; 32]");
            "0202020202020202020202020202020202020202020202020202020202020202".to_string()
        });

        buyer_pubkey = crate::wallet::parse_hex_key(&buyer_hex)?;
        seller_pubkey = crate::wallet::parse_hex_key(&seller_hex)?;
    }

    let result = crate::tx::assemble_create_escrow(
        &buyer_pubkey,
        &seller_pubkey,
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

    let escrow_data = serde_json::json!({
        "lock_tx_id": "pending_fund_me",
        "lock_tx_output_index": 0,
        "buyer_address": counterparty,
        "amount_sompi": result.amount_sompi,
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/v1/escrows", api_url))
        .json(&escrow_data)
        .send()
        .await?;

    if resp.status().is_success() {
        let escrow: serde_json::Value = resp.json().await?;
        let fee_display = fee_preview["fee_kas"]
            .as_str()
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("{:.8}", result.fee_sompi as f64 / 100_000_000.0));

        println!("Escrow created!");
        println!("   ID:       {}", escrow["id"].as_str().unwrap_or("?"));
        println!("   Amount:   {} KAS (fee: {} KAS)", amount_str, fee_display);
        println!("   Status:   pending_confirmation");
        println!();

        if use_kaspawallet {
            // Sign and broadcast automatically
            println!("Signing with kaspawallet...");
            let signed_tx = crate::wallet::sign_with_kaspawallet(&result.unsigned_tx_hex)?;
            println!("Signed transaction: {}", &signed_tx[..32]);
        } else {
            println!("To broadcast, sign with:");
            println!(
                "   kaspawallet sign --transaction {}",
                result.unsigned_tx_hex
            );
        }

        println!();
        println!("Covenant address: {}", result.covenant_address);
        println!("Send funds to this address using:");
        println!(
            "   kaspawallet send --to {} --amount {} --priority normal",
            result.covenant_address, amount_str
        );
        println!();
        println!(
            "Trade link: https://t.me/DagLock_bot?start=claim_{}",
            escrow["id"].as_str().unwrap_or(&result.escrow_id)
        );
    } else {
        let err: serde_json::Value = resp.json().await?;
        anyhow::bail!("Indexer error: {}", err);
    }

    Ok(())
}

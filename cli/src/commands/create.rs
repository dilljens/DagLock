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

    // Require kaspawallet — no dummy key fallback
    if !crate::wallet::kaspawallet_available() {
        anyhow::bail!(
            "kaspawallet is required for creating escrows.\n\
             Install it from: https://kaspa.org/wallets\n\
             Then run: kaspawallet keys --show"
        );
    }

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

    let buyer_pubkey = crate::wallet::parse_hex_key(lines[0].trim())?;
    let seller_pubkey = crate::wallet::parse_hex_key(lines[1].trim())?;

    // Compile covenant to get the covenant address
    let result = crate::tx::assemble_create_escrow(
        &buyer_pubkey,
        &seller_pubkey,
        amount_sompi,
        timeout,
        &treasury_arr,
    )?;

    println!("Covenant address: {}", result.covenant_address);
    println!();
    println!("Step 1: Send {} KAS to the covenant address:", amount_str);
    println!(
        "   kaspawallet send --to {} --amount {} --priority normal",
        result.covenant_address, amount_str
    );
    println!();
    println!("Step 2: Copy the transaction ID from kaspawallet output above.");
    println!();

    // Prompt for the lock transaction ID
    print!("Enter lock transaction ID: ");
    use std::io::Write;
    std::io::stdout().flush()?;
    let mut lock_tx_id = String::new();
    std::io::stdin().read_line(&mut lock_tx_id)?;
    let lock_tx_id = lock_tx_id.trim().to_string();

    if lock_tx_id.is_empty() {
        anyhow::bail!("Lock transaction ID is required.");
    }

    // Create escrow on indexer with the real lock_tx_id
    let escrow_data = serde_json::json!({
        "lock_tx_id": lock_tx_id,
        "lock_tx_output_index": 0,
        "buyer_address": counterparty,
        "amount_sompi": result.amount_sompi,
        "seller_address": lines[0].trim(),
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/v1/escrows", api_url))
        .json(&escrow_data)
        .send()
        .await
        .context("Failed to connect to indexer")?;

    if resp.status().is_success() {
        let escrow: serde_json::Value = resp.json().await?;
        let fee_display = format!("{:.8}", result.fee_sompi as f64 / 100_000_000.0);

        println!();
        println!("Escrow created!");
        println!("   ID:       {}", escrow["id"].as_str().unwrap_or("?"));
        println!("   Amount:   {} KAS (fee: {} KAS)", amount_str, fee_display);
        println!("   Status:   pending_confirmation");
        println!("   Lock TX:  {}", lock_tx_id);
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

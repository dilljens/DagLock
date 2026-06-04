//! DagLock CLI — Trustless escrow & atomic swaps from the terminal.
//!
//! Connects to the DagLock indexer API for queries and assembles
//! unsigned transactions for signing with kaspawallet or KasWare.

mod config;
mod tx;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "daglock-cli", version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Indexer API URL
    #[arg(long, default_value = "http://localhost:8443")]
    api_url: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new escrow proposal
    Create {
        /// Amount in KAS (e.g. "5000" or "5000.5")
        #[arg(long)]
        amount: String,

        /// Counterparty Kaspa address
        #[arg(long)]
        counterparty: String,

        /// Timeout in seconds from now (default: 86400 = 24h)
        #[arg(long, default_value_t = 86400)]
        timeout: u64,

        /// Treasury address for fees
        #[arg(long)]
        treasury: Option<String>,

        /// Escrow ID (for atomic swap pairing)
        #[arg(long)]
        trade_hash: Option<String>,
    },
    /// Claim/release an escrow as the seller
    Claim {
        /// Escrow ID
        id: String,
    },
    /// Refund an escrow as the buyer (after timeout)
    Refund {
        /// Escrow ID
        id: String,
    },
    /// Dispute an escrow
    Dispute {
        /// Escrow ID
        id: String,
        /// Reason for dispute
        #[arg(long)]
        reason: String,
    },
    /// Cancel an escrow before completion
    Cancel {
        /// Escrow ID
        id: String,
    },
    /// Offer board management
    #[command(subcommand)]
    Offer(OfferCommands),
    /// Check escrow status
    Status {
        /// Escrow ID
        id: String,
    },
    /// Check counterparty reputation
    Reputation {
        /// Kaspa address
        address: String,
    },
    /// Fetch a settlement receipt
    Receipt {
        /// Escrow ID
        id: String,
    },
    /// Configure DagLock CLI settings
    Config {
        /// Set indexer API URL
        #[arg(long)]
        api_url: Option<String>,
    },
    /// Send a message on an escrow thread
    Msg {
        /// Escrow ID
        id: String,
        /// Message text
        #[arg(long)]
        text: String,
        /// Your Kaspa address
        #[arg(long)]
        address: String,
        /// Hex signature
        #[arg(long)]
        signature: String,
    },
    /// List messages on an escrow thread
    Messages {
        /// Escrow ID
        id: String,
        /// Your Kaspa address
        #[arg(long)]
        address: String,
        /// Hex signature
        #[arg(long)]
        signature: String,
    },
}

#[derive(Subcommand)]
enum OfferCommands {
    /// List open offers
    List,
    /// Create a new offer
    Create {
        /// Side: buy or sell
        #[arg(long)]
        side: String,

        /// Base asset (e.g. KAS)
        #[arg(long)]
        base: String,

        /// Quote asset (e.g. KRC20:NACHO)
        #[arg(long)]
        quote: String,

        /// Amount in base asset units (e.g. "5000" or "5000.5")
        #[arg(long)]
        amount: String,
    },
    /// Accept an offer
    Accept {
        /// Offer ID
        id: String,

        /// Your Kaspa address
        #[arg(long)]
        address: String,
    },
    /// Cancel an offer
    Cancel {
        /// Offer ID
        id: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    let api_url = cli.api_url;

    match cli.command {
        Commands::Create {
            amount,
            counterparty,
            timeout,
            treasury,
            trade_hash,
        } => {
            commands::create::run(
                api_url,
                &amount,
                &counterparty,
                timeout,
                treasury,
                trade_hash,
            )
            .await?;
        }
        Commands::Claim { id } => {
            commands::claim::run(api_url, &id).await?;
        }
        Commands::Refund { id } => {
            commands::claim::run_refund(api_url, &id).await?;
        }
        Commands::Dispute { id, reason } => {
            commands::claim::run_dispute(api_url, &id, &reason).await?;
        }
        Commands::Cancel { id } => {
            commands::claim::run_cancel(api_url, &id).await?;
        }
        Commands::Offer(cmd) => match cmd {
            OfferCommands::List => commands::offer::list(api_url).await?,
            OfferCommands::Create {
                side,
                base,
                quote,
                amount,
            } => {
                commands::offer::create(api_url, &side, &base, &quote, &amount).await?;
            }
            OfferCommands::Accept { id, address } => {
                commands::offer::accept(api_url, &id, &address).await?;
            }
            OfferCommands::Cancel { id } => {
                commands::offer::cancel(api_url, &id).await?;
            }
        },
        Commands::Status { id } => {
            commands::status::run(api_url, &id).await?;
        }
        Commands::Reputation { address } => {
            commands::reputation::run(api_url, &address).await?;
        }
        Commands::Receipt { id } => {
            commands::receipt::run(api_url, &id).await?;
        }
        Commands::Msg {
            id,
            text,
            address,
            signature,
        } => {
            commands::message::send(api_url, &id, &text, &address, &signature).await?;
        }
        Commands::Messages {
            id,
            address,
            signature,
        } => {
            commands::message::list(api_url, &id, &address, &signature).await?;
        }
        Commands::Config { api_url: new_url } => {
            config::handle_config(new_url).await?;
        }
    }

    Ok(())
}

mod commands {
    pub mod claim;
    pub mod create;
    pub mod message;
    pub mod offer;
    pub mod receipt;
    pub mod reputation;
    pub mod status;
}

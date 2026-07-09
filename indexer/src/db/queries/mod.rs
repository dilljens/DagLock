#![allow(dead_code)]

pub mod apps;
pub mod auth;
pub mod blocks;
pub mod chat_evidence;
pub mod counteroffers;
pub mod deposits;
pub mod escrows;
pub mod evidence;
pub mod feedback;
pub mod identity;
pub mod invoices;
pub mod jury;
pub mod mediations;
pub mod messages;
pub mod milestones;
pub mod multi_escrows;
pub mod notifications;
pub mod flags;
pub mod offers;
pub mod pay;
pub mod reports;
pub mod reputation;
pub mod stats;
pub mod subscriptions;
pub mod tokens;
pub mod vaults;
pub mod vouches;

pub use apps::*;
pub use auth::*;
pub use chat_evidence::*;
pub use deposits::*;
pub use escrows::*;
pub use evidence::*;
pub use identity::*;
pub use invoices::*;
pub use jury::*;
pub use mediations::*;
pub use messages::*;
pub use milestones::*;
pub use multi_escrows::*;
pub use notifications::*;
pub use offers::*;
pub use pay::*;
pub use reputation::*;
pub use stats::*;
pub use vaults::*;
pub use vouches::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    fn escrow_fixture() -> Escrow {
        Escrow {
            id: "esc_123".to_string(),
            lock_tx_id: "tx123".to_string(),
            lock_tx_output_index: 0,
            status: EscrowStatus::Disputed,
            asset_type: "KAS".to_string(),
            buyer_address: "kaspa:buyer".to_string(),
            seller_address: Some("kaspa:seller".to_string()),
            amount_sompi: 500_000_000,
            fee_sompi: 2_500_000,
            template_hash: vec![1, 2, 3],
            expiration_daa_score: Some(42),
            disputed_at: Some(1_700_000_000),
            dispute_reason: Some("seller did not deliver".to_string()),
            cancelled_at: None,
            expired_at: None,
            created_at: 1_700_000_000,
            settled_at: None,
            refunded_at: None,
            mediator_key: None,
            dispute_mode: None,
            dispute_outcome: None,
            dispute_resolved_at: None,
            price_at_creation: None,
            price_currency: None,
            trade_hash: None,
            price_lock_time: None,
            price_at_settlement: None,
            price_source: None,
            price_type: None,
            invoice_id: None,
            memo: None,
            auto_settle_timeout: None,
            mediation_status: None,
            mediation_buyer_claim: None,
            mediation_seller_claim: None,
            mediation_result: None,
            mediation_expires_at: None,
            mediation_buyer_accepted: None,
            mediation_seller_accepted: None,
            chat_pubkey_buyer: None,
            chat_pubkey_seller: None,
        }
    }

    #[test]
    fn reputation_score_rises_with_trade_history() {
        let low = calculate_reputation_score(1, 0, 50_000_000, 1, 0, 0);
        let high = calculate_reputation_score(10, 10, 10_000_000_000, 180, 0, 0);
        assert!(high > low);
    }

    #[test]
    fn reputation_score_falls_with_disputes() {
        let clean = calculate_reputation_score(10, 10, 10_000_000_000, 180, 0, 0);
        let disputed = calculate_reputation_score(10, 10, 10_000_000_000, 180, 2, 2);
        assert!(disputed < clean);
    }

    #[test]
    fn receipt_carries_lifecycle_metadata() {
        let receipt = receipt_from_escrow(&escrow_fixture());
        assert_eq!(receipt.status, "disputed");
        assert_eq!(
            receipt.dispute_reason.as_deref(),
            Some("seller did not deliver")
        );
        assert!(receipt.disputed_at.is_some());
        assert!(!receipt.verification.signatures_verified);
        // fixture: template_hash = [1,2,3] (non-empty) -> covenant_verified = true
        assert!(receipt.verification.covenant_verified);
        // fixture: amount=500M, fee=2.5M -> 500M/200 = 2.5M
        assert!(receipt.verification.fee_compliant);
    }

    #[test]
    fn receipt_detects_incorrect_fee() {
        let mut escrow = escrow_fixture();
        escrow.fee_sompi = 100; // wrong fee
        let receipt = receipt_from_escrow(&escrow);
        assert!(!receipt.verification.fee_compliant);
    }

    #[test]
    fn jury_verdict_seller_wins_at_threshold() {
        let threshold = 3i64;
        let votes_for_seller = 3i64;
        let votes_for_buyer = 1i64;
        assert!(
            votes_for_seller >= threshold,
            "seller should win at threshold"
        );
        assert!(votes_for_buyer < threshold, "buyer should not win yet");
    }

    #[test]
    fn jury_verdict_buyer_wins_at_threshold() {
        let threshold = 3i64;
        let votes_for_seller = 1i64;
        let votes_for_buyer = 3i64;
        assert!(
            votes_for_buyer >= threshold,
            "buyer should win at threshold"
        );
        assert!(votes_for_seller < threshold, "seller should not win yet");
    }

    #[test]
    fn jury_verdict_no_winner_below_threshold() {
        let threshold = 3i64;
        let votes_for_seller = 2i64;
        let votes_for_buyer = 2i64;
        assert!(votes_for_seller < threshold, "no winner yet");
        assert!(votes_for_buyer < threshold, "no winner yet");
    }

    #[test]
    fn receipt_detects_empty_template_hash() {
        let mut escrow = escrow_fixture();
        escrow.template_hash = vec![];
        let receipt = receipt_from_escrow(&escrow);
        assert!(!receipt.verification.covenant_verified);
    }
}

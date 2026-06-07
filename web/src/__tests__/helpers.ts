import { vi } from "vitest";

export function mockApi() {
	return {
		// Health / network
		health: vi.fn().mockResolvedValue({ status: "ok", version: "0.1.0" }),
		network: vi.fn().mockResolvedValue({ network: "testnet-12", daa_score: 100 }),
		networkPrice: vi.fn().mockResolvedValue({ kas_usd: 0.15, updated_at: Date.now() }),
		stats: vi.fn().mockResolvedValue({ total_escrows: 10, active_escrows: 2, settled_escrows: 8 }),
		compile: vi.fn().mockResolvedValue({ script: "deadbeef", template_hash: "aa", abi: [] }),

		// Offers
		offers: vi.fn().mockResolvedValue({ offers: [], total: 0 }),
		createOffer: vi.fn().mockResolvedValue({ id: "offer_1", status: "proposed" }),
		acceptOffer: vi.fn().mockResolvedValue({ status: "accepted", offer_id: "offer_1" }),
		cancelOffer: vi.fn().mockResolvedValue({ status: "cancelled", offer_id: "offer_1" }),

		// Escrows
		escrows: vi.fn().mockResolvedValue({ escrows: [], total: 0 }),
		escrow: vi.fn().mockResolvedValue({ id: "esc_1", status: "active", amount_sompi: 100_000_000 }),
		createEscrow: vi.fn().mockResolvedValue({ id: "esc_1", amount_sompi: 100_000_000 }),
		settleEscrow: vi.fn().mockResolvedValue({ status: "settled", escrow_id: "esc_1" }),
		refundEscrow: vi.fn().mockResolvedValue({ status: "refunded", escrow_id: "esc_1" }),
		disputeEscrow: vi.fn().mockResolvedValue({ status: "disputed", escrow_id: "esc_1" }),
		cancelEscrow: vi.fn().mockResolvedValue({ status: "cancelled", escrow_id: "esc_1" }),
		swapEscrow: vi
			.fn()
			.mockResolvedValue({
				status: "settled",
				escrow_id: "esc_1",
				method: "swap",
				preimage_hash: "abc123",
			}),
		generateSwap: vi.fn().mockResolvedValue({ secret: "secret123", hash: "hash456" }),

		// Messages
		sendMessage: vi.fn().mockResolvedValue({ status: "sent" }),
		listMessages: vi.fn().mockResolvedValue({ messages: [], total: 0 }),

		// Evidence
		submitEvidence: vi.fn().mockResolvedValue({ id: "ev_1", content: "test" }),
		listEvidence: vi.fn().mockResolvedValue({ evidence: [] }),

		// Jury
		juryRegister: vi.fn().mockResolvedValue({ status: "registered" }),
		juryUnregister: vi.fn().mockResolvedValue({ status: "unregistered" }),
		juryCases: vi.fn().mockResolvedValue({ cases: [], total: 0 }),
		juryVote: vi.fn().mockResolvedValue({ status: "voted", vote: "seller_wins" }),

		// Reputation / receipt
		reputation: vi.fn().mockResolvedValue({ score: 5.0, trade_count: 0 }),
		receipt: vi.fn().mockResolvedValue({ receipt_id: "r_1", status: "settled" }),

		// Vaults
		vaults: vi.fn().mockResolvedValue({ vaults: [], total: 0 }),
		vault: vi.fn().mockResolvedValue({ id: "v_1", status: "locked" }),
		createVault: vi.fn().mockResolvedValue({ id: "v_1" }),
		withdrawVault: vi.fn().mockResolvedValue({ status: "withdrawn" }),

		// Identity
		createIdentity: vi.fn().mockResolvedValue({ status: "linked" }),

		// Vouch
		vouch: vi.fn().mockResolvedValue({ status: "vouched" }),
	};
}

export type MockApi = ReturnType<typeof mockApi>;

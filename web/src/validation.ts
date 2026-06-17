import { z } from "zod";

/** Valid Kaspa address starts with 'kaspa:' and is at least 15 chars. */
const kaspaAddress = z.string().refine(
	(addr) => addr.startsWith("kaspa:") && addr.length >= 15,
	{ message: "Must be a valid Kaspa address starting with 'kaspa:'" },
);

/** 64-char hex string for trade hash. */
const tradeHash = z
	.string()
	.regex(/^[a-fA-F0-9]{64}$/, "Must be 64 hex characters")
	.optional();

/** Amount in sompi: positive integer, max 1M KAS. */
const sompiAmount = z
	.number()
	.int()
	.positive("Amount must be positive")
	.max(100_000_000_000_000, "Amount exceeds maximum (1M KAS)");

/** Asset pair for offers. */
const assetName = z.string().min(1, "Asset is required").max(20);

/** Valid side for offers. */
const side = z.enum(["buy", "sell"]);

/** Valid dispute modes. */
const disputeMode = z.enum(["standard", "mediator", "jury"]);

/** Valid price types. */
const priceType = z.enum(["fixed", "market"]).default("fixed");

// ── Form Schemas ───────────────────────────────────────────────

export const CreateOfferSchema = z.object({
	side,
	base_asset: assetName,
	quote_asset: assetName,
	amount_sompi: sompiAmount,
	price_type: priceType,
	price_offset: z.number().optional(),
	min_price: z.number().optional(),
	max_price: z.number().optional(),
});

export const CreateEscrowSchema = z.object({
	amount_sompi: sompiAmount,
	buyer_address: kaspaAddress,
	seller_address: z.string().optional().or(z.literal("")),
	dispute_mode: disputeMode.optional(),
	trade_hash: tradeHash,
	mediator_key: z.string().optional().or(z.literal("")),
});

export const AcceptOfferSchema = z.object({
	counterparty_address: kaspaAddress,
});

export const CreateVaultSchema = z.object({
	amount_sompi: sompiAmount,
	owner_address: kaspaAddress,
	timeout_days: z.number().int().min(1).max(365),
});

export const DisputeSchema = z.object({
	reason: z.string().min(1, "Reason is required").max(500, "Reason too long"),
	content: z.string().max(10000).optional(),
});

export const MessageSchema = z.object({
	content: z.string().min(1, "Message is required").max(1024, "Message too long"),
});

export type CreateOfferFormData = z.infer<typeof CreateOfferSchema>;
export type CreateEscrowFormData = z.infer<typeof CreateEscrowSchema>;
export type CreateVaultFormData = z.infer<typeof CreateVaultSchema>;
export type DisputeFormData = z.infer<typeof DisputeSchema>;

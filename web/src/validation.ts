import { z } from "zod";

/** Valid Kaspa address starts with 'kaspa:' and is at least 15 chars. */
const kaspaAddress = z.string().refine(
	(addr) => addr.startsWith("kaspa:") && addr.length >= 15,
	{ message: "Must be a valid Kaspa address starting with 'kaspa:'" },
);

/** Amount in sompi: positive integer, max 1M KAS. */
const sompiAmount = z
	.number()
	.int()
	.positive("Amount must be positive")
	.max(100_000_000_000_000, "Amount exceeds maximum (1M KAS)");

export const CreateVaultSchema = z.object({
	amount_sompi: sompiAmount,
	owner_address: kaspaAddress,
	timeout_days: z.number().int().min(1).max(365),
});

export type CreateVaultFormData = z.infer<typeof CreateVaultSchema>;

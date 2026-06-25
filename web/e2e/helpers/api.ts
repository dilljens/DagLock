import type { Page } from "@playwright/test";

export async function setupApiMocks(page: Page): Promise<void> {
	await page.route("**/v1/health", async (route) => {
		await route.fulfill({
			status: 200,
			contentType: "application/json",
			body: JSON.stringify({
				status: "ok",
				version: "0.1.0",
				node_synced: true,
				node_daa_score: 123456,
				uptime_seconds: 3600,
			}),
		});
	});

	await page.route("**/v1/stats", async (route) => {
		await route.fulfill({
			status: 200,
			contentType: "application/json",
			body: JSON.stringify({
				total_escrows: 42,
				active_escrows: 12,
				disputed_escrows: 2,
				settled_escrows: 25,
				refunded_escrows: 3,
				cancelled_escrows: 0,
				total_volume_kas: "1500000000000",
				total_fees_collected_kas: "7500000000",
				unique_buyers: 30,
				unique_sellers: 28,
			}),
		});
	});

	await page.route("**/v1/network", async (route) => {
		await route.fulfill({
			status: 200,
			contentType: "application/json",
			body: JSON.stringify({
				network: "testnet-12",
				daa_score: 123456,
				block_count: 50000,
				difficulty: 12345.67,
				bps: 1.0,
			}),
		});
	});

	await page.route("**/v1/network/price", async (route) => {
		await route.fulfill({
			status: 200,
			contentType: "application/json",
			body: JSON.stringify({
				kas_usd: 0.15,
				updated_at: Date.now() / 1000,
			}),
		});
	});

	await page.route("**/v1/offers*", async (route) => {
		await route.fulfill({
			status: 200,
			contentType: "application/json",
			body: JSON.stringify({
				offers: [
					{
						id: "offer_test_1",
						creator_address: "kaspa:testcreatoraddress1234567890abcdef",
						side: "buy",
						base_asset: "KAS",
						quote_asset: "KAS",
						amount_sompi: 100_000_000,
						status: "proposed",
						created_at: Date.now() / 1000 - 3600,
						price_type: "market",
						price_currency: "USD",
					},
				],
				total: 1,
			}),
		});
	});

	await page.route("**/v1/compile", async (route) => {
		if (route.request().method() === "POST") {
			await route.fulfill({
				status: 200,
				contentType: "application/json",
				body: JSON.stringify({
					covenant_address: "kaspatest:covenantaddressformocktestingpurposesonly",
					script: "mock_script_hex",
					template_hash: "30876e3ea42d0e23bb0980f3fd97ae8807e9c70f",
					abi: [{ name: "buyer_key" }, { name: "seller_key" }],
				}),
			});
		} else {
			await route.continue();
		}
	});

	await page.route("**/v1/escrows", async (route) => {
		const url = route.request().url();
		if (route.request().method() === "POST") {
			await route.fulfill({
				status: 200,
				contentType: "application/json",
				body: JSON.stringify({
					id: "esc_test",
					lock_tx_id: "mock_tx_id_abcdef1234567890",
					lock_tx_output_index: 0,
					status: "pending_confirmation",
					asset_type: "KAS",
					buyer_address: "kaspa:qzmockbuyeraddress1234567890abcdef",
					amount_sompi: 100_000_000,
					fee_sompi: 500_000,
					template_hash: [48, 135, 110, 62],
					created_at: Date.now() / 1000,
				}),
			});
		} else if (url.includes("?address=")) {
			await route.fulfill({
				status: 200,
				contentType: "application/json",
				body: JSON.stringify({
					escrows: [
						{
							id: "esc_test_1",
							lock_tx_id: "tx_id_1",
							lock_tx_output_index: 0,
							status: "active",
							asset_type: "KAS",
							buyer_address: "kaspa:qzmockbuyeraddress1234567890abcdef",
							amount_sompi: 100_000_000,
							fee_sompi: 500_000,
							template_hash: [48, 135, 110, 62],
							created_at: Date.now() / 1000 - 86400,
						},
					],
					total: 1,
				}),
			});
		} else {
			await route.continue();
		}
	});

	await page.route("**/v1/escrows/*", async (route) => {
		const url = route.request().url();
		if (url.includes("/settle") || url.includes("/refund") || url.includes("/cancel") || url.includes("/dispute") || url.includes("/evidence") || url.includes("/messages") || url.includes("/swap") || url.includes("/resolve-dispute")) {
			await route.continue();
			return;
		}
		await route.fulfill({
			status: 200,
			contentType: "application/json",
			body: JSON.stringify({
				id: url.split("/").pop() || "esc_test",
				lock_tx_id: "mock_tx_id_abcdef1234567890",
				lock_tx_output_index: 0,
				status: "active",
				asset_type: "KAS",
				buyer_address: "kaspa:qzmockbuyeraddress1234567890abcdef",
				amount_sompi: 100_000_000,
				fee_sompi: 500_000,
				template_hash: [48, 135, 110, 62],
				created_at: Date.now() / 1000 - 86400,
			}),
		});
	});

	await page.route("**/v1/escrows/*/settle", async (route) => {
		await route.fulfill({
			status: 200,
			contentType: "application/json",
			body: JSON.stringify({
				status: "settled",
				escrow_id: "esc_test",
			}),
		});
	});

	await page.route("**/v1/swap/generate", async (route) => {
		await route.fulfill({
			status: 200,
			contentType: "application/json",
			body: JSON.stringify({
				secret: "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
				hash: "fedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321",
			}),
		});
	});
}

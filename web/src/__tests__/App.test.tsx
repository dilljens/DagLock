import { describe, it, expect, beforeAll, afterAll, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import App from "../App";

beforeAll(() => {
	vi.spyOn(globalThis, "fetch").mockImplementation(() =>
		Promise.resolve(
			new Response(JSON.stringify({ status: "ok" }), {
				status: 200,
				headers: { "Content-Type": "application/json" },
			}),
		),
	);
});

afterAll(() => {
	vi.restoreAllMocks();
});

describe("App", () => {
	it("renders the sidebar brand name", () => {
		render(<App />);
		const all = screen.getAllByText("DagLock");
		expect(all.length).toBeGreaterThanOrEqual(1);
	});

	it("renders sidebar navigation items", () => {
		render(<App />);
		expect(screen.getByText("Dashboard")).toBeInTheDocument();
		expect(screen.getByText("Offers")).toBeInTheDocument();
		expect(screen.getByText("Escrows")).toBeInTheDocument();
	});

	it("renders the testnet banner", () => {
		render(<App />);
		expect(screen.getByText(/TESTNET/)).toBeInTheDocument();
	});

	it("renders quick action cards on dashboard", async () => {
		render(<App />);
		await waitFor(() => {
			expect(screen.getByText("Browse Offers")).toBeInTheDocument();
		});
		expect(screen.getByText("Atomic Swap")).toBeInTheDocument();
	});

	it("renders feature cards explaining DagLock", async () => {
		render(<App />);
		await waitFor(() => {
			expect(screen.getByText("Time-Locked Vaults")).toBeInTheDocument();
		});
		expect(screen.getByText("Atomic Swaps")).toBeInTheDocument();
		expect(screen.getByText("AI Mediation")).toBeInTheDocument();
		expect(screen.getByText("E2E On-Chain Chat")).toBeInTheDocument();
	});

	it("renders How It Works section", async () => {
		render(<App />);
		await waitFor(() => {
			expect(screen.getByText("How It Works")).toBeInTheDocument();
		});
		expect(screen.getByText("Create or Accept")).toBeInTheDocument();
		expect(screen.getByText("Lock Funds")).toBeInTheDocument();
		expect(screen.getByText("Settle or Refund")).toBeInTheDocument();
	});

	it("renders footer with fee info", () => {
		render(<App />);
		const feeElements = screen.getAllByText(/0.5% escrow fee/);
		expect(feeElements.length).toBeGreaterThanOrEqual(1);
	});

	it("renders sidebar Docs link", () => {
		render(<App />);
		expect(screen.getAllByText("Docs").length).toBeGreaterThanOrEqual(1);
	});
});

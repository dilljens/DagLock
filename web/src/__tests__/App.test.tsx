import { describe, it, expect, beforeAll, afterAll, vi } from "vitest";
import { render, screen } from "@testing-library/react";
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

	it("renders quick action cards on dashboard", () => {
		render(<App />);
		expect(screen.getByText("Browse Offers")).toBeInTheDocument();
		expect(screen.getByText("Check Reputation")).toBeInTheDocument();
	});
});

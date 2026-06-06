import { describe, it, expect, beforeAll, afterAll, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import App from "../App";

beforeAll(() => {
	// Mock fetch for API calls that happen on mount
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
	it("renders the hero heading", () => {
		render(<App />);
		expect(screen.getByText("Trustless escrow and atomic swaps on Kaspa.")).toBeInTheDocument();
	});

	it("renders the brand name", () => {
		render(<App />);
		expect(screen.getByText("Kaspa Escrow")).toBeInTheDocument();
	});

	it("renders wallet status area", () => {
		render(<App />);
		expect(screen.getByText("No wallet")).toBeInTheDocument();
	});
});

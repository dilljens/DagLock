import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { mockApi } from "./helpers";

vi.mock("../api", () => ({ api: mockApi() }));
import { api } from "../api";
import { CreateEscrowForm } from "../components/escrows";

describe("CreateEscrowForm", () => {
	const onDone = vi.fn();
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it("renders all form fields", () => {
		render(<CreateEscrowForm onDone={onDone} />);
		expect(screen.getByDisplayValue("KAS")).toBeInTheDocument();
		expect(screen.getByPlaceholderText("100")).toBeInTheDocument();
		expect(screen.getByLabelText("Buyer address")).toBeInTheDocument();
		expect(screen.getByLabelText("Seller address (optional)")).toBeInTheDocument();
		expect(screen.getByDisplayValue("Standard (timeout refund)")).toBeInTheDocument();
		expect(screen.getByDisplayValue("Market price (locked at creation)")).toBeInTheDocument();
	});

	it("shows success with escrow ID on submit", async () => {
		const user = userEvent.setup();
		(api.createEscrow as ReturnType<typeof vi.fn>).mockResolvedValue({
			id: "esc_abc",
			amount_sompi: 100_000_000,
			lock_tx_id: "abc123def456_tx_id",
			status: "pending_confirmation",
		});

		(window as any).kasware = {
			sendKaspa: vi.fn().mockResolvedValue("abc123def456_tx_id"),
		};

		render(<CreateEscrowForm onDone={onDone} />);

		const amountInput = screen.getByPlaceholderText("100");
		await user.type(amountInput, "100");

		const buyerInput = screen.getByLabelText("Buyer address");
		await user.type(buyerInput, "kaspa:qr6g5fsvq5h4c56j8w6q8w6q8w6q8w6q8w6q8w6q");

		const submitBtn = screen.getByRole("button", { name: /create escrow/i });
		await user.click(submitBtn);

		await waitFor(() => {
			expect(screen.getByText("Escrow created!")).toBeInTheDocument();
		});
		expect(screen.getByText("esc_abc")).toBeInTheDocument();
	});

	it("shows error on invalid buyer address", async () => {
		const user = userEvent.setup();
		(window as any).kasware = {
			sendKaspa: vi.fn().mockResolvedValue("abc123def456_tx_id"),
		};

		render(<CreateEscrowForm onDone={onDone} />);

		const amountInput = screen.getByPlaceholderText("100");
		await user.type(amountInput, "100");

		const buyerInput = screen.getByLabelText("Buyer address");
		await user.type(buyerInput, "invalid");

		const submitBtn = screen.getByRole("button", { name: /create escrow/i });
		await user.click(submitBtn);

		await waitFor(() => {
			expect(screen.getByText(/Invalid buyer address/)).toBeInTheDocument();
		});
		expect(onDone).not.toHaveBeenCalled();
	});

	it("shows error when API fails", async () => {
		const user = userEvent.setup();
		(api.createEscrow as ReturnType<typeof vi.fn>).mockRejectedValue(new Error("Server error"));

		(window as any).kasware = {
			sendKaspa: vi.fn().mockResolvedValue("abc123def456_tx_id"),
		};

		render(<CreateEscrowForm onDone={onDone} />);

		const amountInput = screen.getByPlaceholderText("100");
		await user.type(amountInput, "100");

		const buyerInput = screen.getByLabelText("Buyer address");
		await user.type(buyerInput, "kaspa:qr6g5fsvq5h4c56j8w6q8w6q8w6q8w6q8w6q8w6q");

		const submitBtn = screen.getByRole("button", { name: /create escrow/i });
		await user.click(submitBtn);

		await waitFor(() => {
			expect(screen.getByText("Server error")).toBeInTheDocument();
		});
	});
});

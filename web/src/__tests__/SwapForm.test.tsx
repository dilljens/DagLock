import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { mockApi } from "./helpers";

vi.mock("../api", () => ({ api: mockApi() }));
import { api } from "../api";
import { SwapForm } from "../components/escrows";

describe("SwapForm", () => {
	const onDone = vi.fn();
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it("renders escrow ID and preimage fields", () => {
		render(<SwapForm onDone={onDone} />);
		expect(screen.getByPlaceholderText("esc_...")).toBeInTheDocument();
		expect(screen.getByPlaceholderText(/hex encoded secret/)).toBeInTheDocument();
	});

	it("shows success on valid input", async () => {
		const user = userEvent.setup();
		(api.swapEscrow as ReturnType<typeof vi.fn>).mockResolvedValue({
			status: "settled",
			escrow_id: "esc_1",
			method: "swap",
			preimage_hash: "abc123",
		});

		render(<SwapForm onDone={onDone} />);

		const escrowInput = screen.getByPlaceholderText("esc_...");
		await user.type(escrowInput, "esc_1");

		const preimageInput = screen.getByPlaceholderText(/hex encoded secret/);
		await user.type(preimageInput, "abc123");

		const submitBtn = screen.getByRole("button", { name: /submit preimage/i });
		await user.click(submitBtn);

		await waitFor(() => {
			expect(screen.getByText(/Swap settled!/)).toBeInTheDocument();
		});
	});

	it("shows error on API failure", async () => {
		const user = userEvent.setup();
		(api.swapEscrow as ReturnType<typeof vi.fn>).mockRejectedValue(new Error("Invalid preimage"));

		render(<SwapForm onDone={onDone} />);

		const escrowInput = screen.getByPlaceholderText("esc_...");
		await user.type(escrowInput, "esc_1");

		const preimageInput = screen.getByPlaceholderText(/hex encoded secret/);
		await user.type(preimageInput, "wrong");

		const submitBtn = screen.getByRole("button", { name: /submit preimage/i });
		await user.click(submitBtn);

		await waitFor(() => {
			expect(screen.getByText("Invalid preimage")).toBeInTheDocument();
		});
	});
});

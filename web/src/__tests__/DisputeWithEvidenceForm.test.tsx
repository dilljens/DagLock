import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { mockApi } from "./helpers";

vi.mock("../api", () => ({ api: mockApi() }));
import { api } from "../api";
import { DisputeWithEvidenceForm } from "../components/escrows";

const VALID_ADDR = "kaspa:qr6g5fsvq5h4c56j8w6q8w6q8w6q8w6q8w6q8w6q";

describe("DisputeWithEvidenceForm", () => {
	const onDone = vi.fn();
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it("renders all form fields", () => {
		render(<DisputeWithEvidenceForm onDone={onDone} />);
		expect(screen.getByPlaceholderText("esc_...")).toBeInTheDocument();
		expect(screen.getByPlaceholderText("Why are you disputing?")).toBeInTheDocument();
		expect(screen.getByPlaceholderText(/Describe what happened/)).toBeInTheDocument();
		expect(screen.getByPlaceholderText("kaspa:...")).toBeInTheDocument();
	});

	it("shows Disputed on success", async () => {
		const user = userEvent.setup();
		(api.disputeEscrow as ReturnType<typeof vi.fn>).mockResolvedValue({
			status: "disputed",
			escrow_id: "esc_1",
		});

		render(<DisputeWithEvidenceForm onDone={onDone} />);

		const escrowInput = screen.getByPlaceholderText("esc_...");
		await user.type(escrowInput, "esc_1");

		const reasonInput = screen.getByPlaceholderText("Why are you disputing?");
		await user.type(reasonInput, "Seller didn't deliver");

		const addressInput = screen.getByPlaceholderText("kaspa:...");
		await user.type(addressInput, VALID_ADDR);

		const sigInput = screen.getByPlaceholderText("auto-filled when signing");
		await user.type(sigInput, "abc123");

		const submitBtn = screen.getByRole("button", { name: /submit dispute/i });
		await user.click(submitBtn);

		await waitFor(() => {
			expect(screen.getByText(/Disputed — esc_1/)).toBeInTheDocument();
		});
	});

	it("shows error when auth missing", async () => {
		const user = userEvent.setup();
		render(<DisputeWithEvidenceForm onDone={onDone} />);

		const escrowInput = screen.getByPlaceholderText("esc_...");
		await user.type(escrowInput, "esc_1");

		const reasonInput = screen.getByPlaceholderText("Why are you disputing?");
		await user.type(reasonInput, "Some reason");

		const submitBtn = screen.getByRole("button", { name: /submit dispute/i });
		await user.click(submitBtn);

		await waitFor(() => {
			expect(screen.getByText(/Authentication required/)).toBeInTheDocument();
		});
	});

	it("shows error when reason missing", async () => {
		const user = userEvent.setup();
		render(<DisputeWithEvidenceForm onDone={onDone} />);

		const escrowInput = screen.getByPlaceholderText("esc_...");
		await user.type(escrowInput, "esc_1");

		const submitBtn = screen.getByRole("button", { name: /submit dispute/i });
		await user.click(submitBtn);

		// Should not proceed — no API call
		await waitFor(() => {
			expect(api.disputeEscrow).not.toHaveBeenCalled();
		});
	});
});

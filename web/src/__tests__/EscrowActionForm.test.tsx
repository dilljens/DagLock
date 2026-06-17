import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { mockApi } from "./helpers";

vi.mock("../api", () => ({ api: mockApi() }));
import { api } from "../api";
import { EscrowActionForm } from "../components/escrows";

const VALID_ADDR = "kaspa:qr6g5fsvq5h4c56j8w6q8w6q8w6q8w6q8w6q8w6q";

describe("EscrowActionForm", () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	describe("settle", () => {
		it("renders auth fields for settle", () => {
			render(<EscrowActionForm action="settle" />);
			expect(screen.getByPlaceholderText("esc_...")).toBeInTheDocument();
			expect(screen.getByPlaceholderText("kaspa:...")).toBeInTheDocument();
		});

		it("shows success on valid auth", async () => {
			const user = userEvent.setup();
			(api.settleEscrow as ReturnType<typeof vi.fn>).mockResolvedValue({
				status: "settled",
				escrow_id: "esc_1",
			});

			render(<EscrowActionForm action="settle" />);

			const escrowInput = screen.getByPlaceholderText("esc_...");
			await user.type(escrowInput, "esc_1");

			const addressInput = screen.getByPlaceholderText("kaspa:...");
			await user.type(addressInput, VALID_ADDR);

			const sigInput = screen.getByPlaceholderText("auto-filled when signing");
			await user.type(sigInput, "abc123");

			const submitBtn = screen.getByRole("button", { name: /settle/i });
			await user.click(submitBtn);

			await waitFor(() => {
				expect(screen.getByText(/settled/)).toBeInTheDocument();
			});
		});

		it("shows error when auth missing", async () => {
			const user = userEvent.setup();
			render(<EscrowActionForm action="settle" />);

			const escrowInput = screen.getByPlaceholderText("esc_...");
			await user.type(escrowInput, "esc_1");

			const submitBtn = screen.getByRole("button", { name: /settle/i });
			await user.click(submitBtn);

			await waitFor(() => {
				expect(screen.getByText(/Authentication required/)).toBeInTheDocument();
			});
		});
	});

	describe("cancel", () => {
		it("shows ConfirmDialog on submit", async () => {
			const user = userEvent.setup();
			render(<EscrowActionForm action="cancel" />);

			const escrowInput = screen.getByPlaceholderText("esc_...");
			await user.type(escrowInput, "esc_1");

			const submitBtn = screen.getByRole("button", { name: /cancel/i });
			await user.click(submitBtn);

			await waitFor(() => {
				expect(screen.getByText(/Are you sure you want to cancel/)).toBeInTheDocument();
			});

			// ConfirmDialog should be visible
			expect(screen.getByText("Cancel escrow")).toBeInTheDocument();
			expect(screen.getByText(/Are you sure you want to cancel escrow esc_1/)).toBeInTheDocument();
		});

		it("confirms cancel and submits", async () => {
			const user = userEvent.setup();
			(api.cancelEscrow as ReturnType<typeof vi.fn>).mockResolvedValue({
				status: "cancelled",
				escrow_id: "esc_1",
			});

			render(<EscrowActionForm action="cancel" />);

			const escrowInput = screen.getByPlaceholderText("esc_...");
			await user.type(escrowInput, "esc_1");

			const submitBtn = screen.getByRole("button", { name: /cancel/i });
			await user.click(submitBtn);

			await waitFor(() => {
				expect(screen.getByText(/Are you sure you want to cancel/)).toBeInTheDocument();
			});

			// The dialog has two "Cancel" buttons: dismiss + confirm.
			// Find all "Cancel" buttons and click the LAST one (confirm, primary style).
			const allCancelBtns = screen.getAllByRole("button", { name: "Cancel" });
			await user.click(allCancelBtns[allCancelBtns.length - 1]);

			await waitFor(() => {
				expect(api.cancelEscrow).toHaveBeenCalledWith("esc_1");
			});
		});

		it("dismisses ConfirmDialog on cancel", async () => {
			const user = userEvent.setup();
			const { container } = render(<EscrowActionForm action="cancel" />);

			const escrowInput = screen.getByPlaceholderText("esc_...");
			await user.type(escrowInput, "esc_1");

			const submitBtn = screen.getByRole("button", { name: /cancel/i });
			await user.click(submitBtn);

			await waitFor(() => {
				expect(screen.getByText(/Are you sure you want to cancel/)).toBeInTheDocument();
			});

			// Find the dismiss button inside the Radix Dialog Portal (rendered in document.body).
			// The dismiss button is the one that is NOT of class "button primary".
			const dialog = document.querySelector('[role="dialog"]');
			expect(dialog).not.toBeNull();
			const dismissBtn = dialog!.querySelector(".button:not(.primary)");
			expect(dismissBtn).not.toBeNull();
			await user.click(dismissBtn!);

			await waitFor(() => {
				expect(screen.queryByText(/Are you sure you want to cancel/)).not.toBeInTheDocument();
			});
			expect(api.cancelEscrow).not.toHaveBeenCalled();
		});
	});

	describe("refund", () => {
		it("shows ConfirmDialog for refund", async () => {
			const user = userEvent.setup();
			render(<EscrowActionForm action="refund" />);

			const escrowInput = screen.getByPlaceholderText("esc_...");
			await user.type(escrowInput, "esc_1");

			const submitBtn = screen.getByRole("button", { name: /refund/i });
			await user.click(submitBtn);

			await waitFor(() => {
				expect(screen.getByText(/Are you sure you want to refund/)).toBeInTheDocument();
			});
		});
	});

	describe("dispute", () => {
		it("shows reason field for dispute", () => {
			render(<EscrowActionForm action="dispute" />);
			expect(screen.getByPlaceholderText("Why are you disputing?")).toBeInTheDocument();
		});
	});
});

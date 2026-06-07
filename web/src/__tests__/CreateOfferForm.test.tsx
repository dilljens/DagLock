import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { mockApi } from "./helpers";

vi.mock("../api", () => ({ api: mockApi() }));
import { api } from "../api";
import { CreateOfferForm } from "../components/offers";

describe("CreateOfferForm", () => {
	const onDone = vi.fn();
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it("renders all form fields", () => {
		render(<CreateOfferForm onDone={onDone} />);
		expect(screen.getByDisplayValue("Sell")).toBeInTheDocument();
		expect(screen.getByDisplayValue("KAS")).toBeInTheDocument();
		expect(screen.getByDisplayValue("USDC")).toBeInTheDocument();
		expect(screen.getByPlaceholderText("100")).toBeInTheDocument();
		expect(screen.getByDisplayValue("Fixed price")).toBeInTheDocument();
	});

	it("shows success on submit with valid address", async () => {
		const user = userEvent.setup();
		(api.createOffer as ReturnType<typeof vi.fn>).mockResolvedValue({
			id: "offer_1",
			status: "proposed",
		});

		render(<CreateOfferForm onDone={onDone} />);

		const amountInput = screen.getByPlaceholderText("100");
		await user.type(amountInput, "100");

		const addressInput = screen.getByLabelText("Your address");
		await user.type(addressInput, "kaspa:qr6g5fsvq5h4c56j8w6q8w6q8w6q8w6q8w6q8w6q");

		const submitBtn = screen.getByRole("button", { name: /create offer/i });
		await user.click(submitBtn);

		await waitFor(() => {
			expect(screen.getByText("Offer created!")).toBeInTheDocument();
		});
		expect(onDone).toHaveBeenCalled();
	});

	it("shows error on invalid address", async () => {
		const user = userEvent.setup();
		render(<CreateOfferForm onDone={onDone} />);

		const amountInput = screen.getByPlaceholderText("100");
		await user.type(amountInput, "100");

		const addressInput = screen.getByLabelText("Your address");
		await user.type(addressInput, "invalid");

		const submitBtn = screen.getByRole("button", { name: /create offer/i });
		await user.click(submitBtn);

		await waitFor(() => {
			expect(screen.getByText(/Invalid address format/)).toBeInTheDocument();
		});
		expect(onDone).not.toHaveBeenCalled();
	});

	it("shows error on invalid amount", async () => {
		const user = userEvent.setup();
		render(<CreateOfferForm onDone={onDone} />);

		const submitBtn = screen.getByRole("button", { name: /create offer/i });
		await user.click(submitBtn);

		await waitFor(() => {
			expect(screen.getByText(/Invalid amount/)).toBeInTheDocument();
		});
	});
});

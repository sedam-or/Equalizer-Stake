import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { HomePage } from "./index";

describe("home route", () => {
  it("renders product name", () => {
    render(<HomePage />);

    expect(screen.getByRole("heading", { name: "Ritma" })).toBeDefined();
  });
});

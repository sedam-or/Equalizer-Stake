import { createFileRoute } from "@tanstack/react-router";
import { BRAND } from "../brand";

export function HomePage() {
  return (
    <main className="grid min-h-screen place-items-center bg-bg p-6 text-text">
      <section className="rounded-md border border-surface bg-surface p-8">
        <h1 className="text-3xl font-bold">{BRAND.name}</h1>
        <p>{BRAND.tagline}</p>
      </section>
    </main>
  );
}

export const Route = createFileRoute("/")({ component: HomePage });

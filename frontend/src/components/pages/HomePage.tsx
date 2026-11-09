import { Link } from "react-router-dom";

export function HomePage() {
  return (
    <div className="max-w-3xl mx-auto px-6 pt-20 pb-12">
      <h1 className="text-4xl font-bold text-text text-center mb-4">
        Template
      </h1>
      <p className="text-lg text-text-secondary text-center mb-16">
        A full-stack webapp template powered by Rust (Axum) + React + TypeScript + Tailwind
      </p>

      <div className="grid gap-6 md:grid-cols-3">
        <FeatureCard
          to="/docs/backend"
          title="Axum Backend"
          description="Type-safe Rust API with Axum 0.8, SQLx, JWT auth, rate limiting, and security headers."
        />
        <FeatureCard
          to="/docs/frontend"
          title="React Frontend"
          description="React 19 + TypeScript + Vite + Tailwind CSS + Zustand for state management."
        />
        <FeatureCard
          to="/docs/deployment"
          title="Production Ready"
          description="Docker multi-stage build, PostgreSQL migrations, graceful shutdown, health checks."
        />
      </div>
    </div>
  );
}

function FeatureCard({
  to,
  title,
  description,
}: {
  to: string;
  title: string;
  description: string;
}) {
  return (
    <Link
      to={to}
      className="block p-5 rounded-lg bg-surface border border-border-subtle hover:border-accent/40 hover:bg-surface-hover transition-colors"
    >
      <h3 className="text-base font-semibold text-text mb-2">{title}</h3>
      <p className="text-sm text-text-secondary">{description}</p>
      <span className="inline-block mt-3 text-xs text-accent font-medium">
        Read docs &rarr;
      </span>
    </Link>
  );
}

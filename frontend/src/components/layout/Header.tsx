import { Link } from "react-router-dom";

export function Header() {
  return (
    <header className="flex items-center justify-between px-6 py-3 border-b border-border-subtle bg-surface">
      <Link to="/" className="text-lg font-semibold text-accent hover:text-accent-hover">
        Template
      </Link>
    </header>
  );
}

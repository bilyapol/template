import { useState } from "react";
import { useAuthStore } from "@/stores/auth.store";
import { logout, devLogin, fetchMe } from "@/api/auth";
import { Link } from "react-router-dom";
import { useAuthMode } from "@/stores/authmode.store";

export function Header() {
  const user = useAuthStore((s) => s.user);
  const setUser = useAuthStore((s) => s.setUser);
  const authMode = useAuthMode((s) => s.mode);
  const [showDevLogin, setShowDevLogin] = useState(false);
  const [email, setEmail] = useState("");

  const handleLogout = async () => {
    await logout();
    setUser(null);
  };

  const handleDevLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!email.trim()) return;
    await devLogin(email.trim());
    const me = await fetchMe();
    setUser(me);
    setShowDevLogin(false);
    setEmail("");
  };

  return (
    <header className="flex items-center justify-between px-6 py-3 border-b border-border-subtle bg-surface">
      <Link to="/" className="text-lg font-semibold text-accent hover:text-accent-hover">
        Template
      </Link>
      <nav className="flex items-center gap-4">
        {user ? (
          <>
            <Link
              to="/dashboard"
              className="text-sm text-text-secondary hover:text-text"
            >
              Dashboard
            </Link>
            <div className="flex items-center gap-3">
              {user.avatar_url && (
                <img
                  src={user.avatar_url}
                  alt={user.name}
                  className="w-7 h-7 rounded-full"
                />
              )}
              <span className="text-sm text-text-secondary">{user.name}</span>
              <button
                onClick={handleLogout}
                className="text-sm text-text-muted hover:text-text"
              >
                Logout
              </button>
            </div>
          </>
        ) : authMode === "google" ? (
          <a
            href="/api/auth/google"
            className="px-4 py-1.5 text-sm font-medium rounded-md bg-btn-bg text-btn-text hover:bg-btn-hover"
          >
            Sign in with Google
          </a>
        ) : authMode === "dev" ? (
          <div className="relative">
            <button
              onClick={() => setShowDevLogin(!showDevLogin)}
              className="px-4 py-1.5 text-sm font-medium rounded-md bg-btn-bg text-btn-text hover:bg-btn-hover"
            >
              Dev Login
            </button>
            {showDevLogin && (
              <form
                onSubmit={handleDevLogin}
                className="absolute right-0 top-full mt-2 p-3 bg-surface border border-border rounded-lg shadow-lg z-50 w-64"
              >
                <input
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder="you@example.com"
                  autoFocus
                  className="w-full mb-2 px-3 py-1.5 rounded-md bg-bg border border-border text-text text-sm placeholder:text-text-muted focus:outline-none focus:border-accent"
                />
                <button
                  type="submit"
                  className="w-full px-3 py-1.5 text-sm font-medium rounded-md bg-btn-bg text-btn-text hover:bg-btn-hover"
                >
                  Log in
                </button>
              </form>
            )}
          </div>
        ) : null}
      </nav>
    </header>
  );
}

import { useEffect, type ReactNode } from "react";
import { useAuthStore } from "@/stores/auth.store";
import { useAuthMode } from "@/stores/authmode.store";
import { fetchMe } from "@/api/auth";
import { Header } from "./Header";
import { Spinner } from "@/components/ui/Spinner";

export function AppShell({ children }: { children: ReactNode }) {
  const { isLoading, setUser } = useAuthStore();
  const loadAuthMode = useAuthMode((s) => s.load);

  useEffect(() => {
    loadAuthMode();
    fetchMe()
      .then((user) => setUser(user))
      .catch(() => setUser(null));
  }, [setUser, loadAuthMode]);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-screen">
        <Spinner className="w-8 h-8 text-accent" />
      </div>
    );
  }

  return (
    <div className="flex flex-col h-screen">
      <Header />
      <main className="flex-1 overflow-auto">{children}</main>
    </div>
  );
}

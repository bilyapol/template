import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import { AppShell } from "@/components/layout/AppShell";
import { useAuthStore } from "@/stores/auth.store";
import { HomePage } from "@/components/pages/HomePage";
import { DashboardPage } from "@/components/pages/DashboardPage";
import { DocPage } from "@/components/pages/DocPage";
import type { ReactNode } from "react";

function RequireAuth({ children }: { children: ReactNode }) {
  const user = useAuthStore((s) => s.user);
  if (!user) return <Navigate to="/" replace />;
  return <>{children}</>;
}

export default function App() {
  return (
    <BrowserRouter>
      <AppShell>
        <Routes>
          <Route path="/" element={<HomePage />} />
          <Route path="/docs/:slug" element={<DocPage />} />
          <Route
            path="/dashboard"
            element={
              <RequireAuth>
                <DashboardPage />
              </RequireAuth>
            }
          />
        </Routes>
      </AppShell>
    </BrowserRouter>
  );
}

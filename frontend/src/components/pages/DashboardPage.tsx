import { useAuthStore } from "@/stores/auth.store";

export function DashboardPage() {
  const user = useAuthStore((s) => s.user);

  return (
    <div className="max-w-2xl mx-auto px-6 pt-10 pb-12">
      <h1 className="text-2xl font-semibold text-text mb-1">Dashboard</h1>
      <p className="text-sm text-text-secondary">Welcome back, {user?.name}</p>
    </div>
  );
}

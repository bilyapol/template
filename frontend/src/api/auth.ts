import { apiFetch } from "./client";
import type { User } from "@/types/api";

export function fetchMe(): Promise<User> {
  return apiFetch<User>("/api/user/me");
}

export async function logout(): Promise<void> {
  await apiFetch("/api/auth/logout", { method: "POST" });
}

export function fetchAuthMode(): Promise<{ mode: "google" | "dev" | "none" }> {
  return apiFetch("/api/auth/mode");
}

export async function devLogin(email: string, name?: string): Promise<void> {
  await apiFetch("/api/auth/dev-login", {
    method: "POST",
    body: JSON.stringify({ email, name }),
  });
}

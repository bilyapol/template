import { create } from "zustand";
import { fetchAuthMode } from "@/api/auth";

interface AuthModeState {
  mode: "google" | "dev" | "none" | null;
  load: () => Promise<void>;
}

export const useAuthMode = create<AuthModeState>((set) => ({
  mode: null,
  load: async () => {
    const { mode } = await fetchAuthMode();
    set({ mode });
  },
}));

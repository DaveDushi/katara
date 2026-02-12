import { create } from "zustand";

interface AppState {
  activeTab: string;
  runtimeUrl: string | null;
  wsPort: number;
  axumPort: number;
  setActiveTab: (tab: string) => void;
  setRuntimeUrl: (url: string) => void;
  setPorts: (ws: number, axum: number) => void;
}

export const useAppStore = create<AppState>((set) => ({
  activeTab: "chat",
  runtimeUrl: null,
  wsPort: 0,
  axumPort: 0,
  setActiveTab: (tab) => set({ activeTab: tab }),
  setRuntimeUrl: (url) => set({ runtimeUrl: url }),
  setPorts: (ws, axum) =>
    set({
      wsPort: ws,
      axumPort: axum,
      runtimeUrl: axum > 0 ? `http://127.0.0.1:${axum}` : null,
    }),
}));

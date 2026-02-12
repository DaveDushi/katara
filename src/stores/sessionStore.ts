import { create } from "zustand";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

export interface SessionInfo {
  id: string;
  status: string;
  working_dir: string;
}

export interface ClaudeMessage {
  session_id: string;
  message: {
    type: string;
    [key: string]: unknown;
  };
}

interface SessionState {
  sessions: SessionInfo[];
  activeSessionId: string | null;
  messages: ClaudeMessage[];
  initialized: boolean;
  setSessions: (sessions: SessionInfo[]) => void;
  setActiveSession: (id: string | null) => void;
  addSession: (session: SessionInfo) => void;
  removeSession: (id: string) => void;
  updateSessionStatus: (id: string, status: string) => void;
  addMessage: (msg: ClaudeMessage) => void;
  clearMessages: () => void;
  spawnSession: (workingDir: string, initialPrompt?: string) => Promise<string>;
  killSession: (sessionId: string) => Promise<void>;
  sendMessage: (sessionId: string, content: string) => Promise<void>;
  approveTool: (
    sessionId: string,
    requestId: string,
    approved: boolean,
    updatedInput?: unknown
  ) => Promise<void>;
  interruptSession: (sessionId: string) => Promise<void>;
  getMessageHistory: (sessionId: string) => Promise<ClaudeMessage[]>;
  initListeners: () => void;
}

export const useSessionStore = create<SessionState>((set, get) => ({
  sessions: [],
  activeSessionId: null,
  messages: [],
  initialized: false,
  setSessions: (sessions) => set({ sessions }),
  setActiveSession: (id) => set({ activeSessionId: id }),
  addSession: (session) =>
    set((state) => ({
      sessions: [...state.sessions, session],
      activeSessionId: session.id,
    })),
  removeSession: (id) =>
    set((state) => ({
      sessions: state.sessions.filter((s) => s.id !== id),
      activeSessionId:
        state.activeSessionId === id ? null : state.activeSessionId,
    })),
  updateSessionStatus: (id, status) =>
    set((state) => ({
      sessions: state.sessions.map((s) =>
        s.id === id ? { ...s, status } : s
      ),
    })),
  addMessage: (msg) =>
    set((state) => ({
      messages: [...state.messages, msg],
    })),
  clearMessages: () => set({ messages: [] }),

  spawnSession: async (workingDir, initialPrompt) => {
    const sessionId = await invoke<string>("spawn_session", {
      workingDir,
      initialPrompt: initialPrompt ?? null,
    });
    get().addSession({
      id: sessionId,
      status: "Starting",
      working_dir: workingDir,
    });
    return sessionId;
  },

  killSession: async (sessionId) => {
    await invoke("kill_session", { sessionId });
    get().removeSession(sessionId);
  },

  sendMessage: async (sessionId, content) => {
    await invoke("send_message", { sessionId, content });
  },

  approveTool: async (sessionId, requestId, approved, updatedInput) => {
    await invoke("approve_tool", {
      sessionId,
      requestId,
      approved,
      updatedInput: updatedInput ?? null,
    });
  },

  interruptSession: async (sessionId) => {
    await invoke("interrupt_session", { sessionId });
  },

  getMessageHistory: async (sessionId) => {
    return await invoke<ClaudeMessage[]>("get_message_history", { sessionId });
  },

  initListeners: () => {
    if (get().initialized) return;
    set({ initialized: true });

    // Listen for session status changes from Rust
    listen<{ session_id: string; status: string }>("claude:status", (event) => {
      const { session_id, status } = event.payload;
      get().updateSessionStatus(session_id, status);
    });

    // Listen for Claude messages from Rust
    listen<ClaudeMessage>("claude:message", (event) => {
      get().addMessage(event.payload);
    });
  },
}));

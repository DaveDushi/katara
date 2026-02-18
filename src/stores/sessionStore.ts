import { create } from "zustand";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

export interface SessionInfo {
  id: string;
  status: string;
  working_dir: string;
  model?: string;
  permission_mode?: string;
}

export interface UsageTotals {
  input_tokens: number;
  output_tokens: number;
  cache_creation_input_tokens: number;
  cache_read_input_tokens: number;
}

export interface PendingApproval {
  requestId: string;
  toolName: string;
  toolInput: Record<string, unknown>;
  toolUseId: string;
  sessionId: string;
  timestamp: number;
}

export interface SessionCost {
  session_id: string;
  model: string | null;
  input_tokens: number;
  output_tokens: number;
  cache_creation_input_tokens: number;
  cache_read_input_tokens: number;
  estimated_cost_usd: number;
}

export interface ClaudeMessage {
  session_id: string;
  message: {
    type: string;
    request?: {
      subtype?: string;
      request_id?: string;
      tool_name?: string;
      tool_use_id?: string;
      input?: Record<string, unknown>;
    };
    [key: string]: unknown;
  };
}

interface SessionState {
  sessions: SessionInfo[];
  activeSessionId: string | null;
  messages: ClaudeMessage[];
  initialized: boolean;
  pendingApprovals: PendingApproval[];
  sessionUsage: Record<string, UsageTotals>;

  setSessions: (sessions: SessionInfo[]) => void;
  setActiveSession: (id: string | null) => void;
  addSession: (session: SessionInfo) => void;
  removeSession: (id: string) => void;
  updateSessionStatus: (id: string, status: string) => void;
  addMessage: (msg: ClaudeMessage) => void;
  clearMessages: () => void;

  spawnSession: (
    workingDir: string,
    initialPrompt?: string,
    model?: string,
    permissionMode?: string
  ) => Promise<string>;
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
  setPermissionMode: (sessionId: string, mode: string) => Promise<void>;
  getSessionCost: (sessionId: string) => Promise<SessionCost>;

  addPendingApproval: (approval: PendingApproval) => void;
  removePendingApproval: (requestId: string) => void;
  clearPendingApprovals: (sessionId: string) => void;

  initListeners: () => void;
}

export const useSessionStore = create<SessionState>((set, get) => ({
  sessions: [],
  activeSessionId: null,
  messages: [],
  initialized: false,
  pendingApprovals: [],
  sessionUsage: {},

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
      pendingApprovals: state.pendingApprovals.filter(
        (a) => a.sessionId !== id
      ),
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

  addPendingApproval: (approval) =>
    set((state) => ({
      pendingApprovals: [...state.pendingApprovals, approval],
    })),
  removePendingApproval: (requestId) =>
    set((state) => ({
      pendingApprovals: state.pendingApprovals.filter(
        (a) => a.requestId !== requestId
      ),
    })),
  clearPendingApprovals: (sessionId) =>
    set((state) => ({
      pendingApprovals: state.pendingApprovals.filter(
        (a) => a.sessionId !== sessionId
      ),
    })),

  spawnSession: async (workingDir, initialPrompt, model, permissionMode) => {
    const sessionId = await invoke<string>("spawn_session", {
      workingDir,
      initialPrompt: initialPrompt ?? null,
      model: model ?? null,
      permissionMode: permissionMode ?? null,
    });
    get().addSession({
      id: sessionId,
      status: "Starting",
      working_dir: workingDir,
      model,
      permission_mode: permissionMode ?? "default",
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

  setPermissionMode: async (sessionId, mode) => {
    await invoke("set_permission_mode", {
      sessionId,
      permissionMode: mode,
    });
    set((state) => ({
      sessions: state.sessions.map((s) =>
        s.id === sessionId ? { ...s, permission_mode: mode } : s
      ),
    }));
  },

  getSessionCost: async (sessionId) => {
    return await invoke<SessionCost>("get_session_cost", { sessionId });
  },

  initListeners: () => {
    if (get().initialized) return;
    set({ initialized: true });

    // Listen for session status changes from Rust
    listen<{ session_id: string; status: string }>(
      "claude:status",
      (event) => {
        const { session_id, status } = event.payload;
        get().updateSessionStatus(session_id, status);
      }
    );

    // Listen for Claude messages from Rust
    listen<ClaudeMessage>("claude:message", (event) => {
      get().addMessage(event.payload);

      // Detect tool approval requests (control_request with can_use_tool)
      const msg = event.payload.message;
      if (
        msg.type === "control_request" &&
        msg.request?.subtype === "can_use_tool" &&
        msg.request?.request_id
      ) {
        get().addPendingApproval({
          requestId: msg.request.request_id,
          toolName: msg.request.tool_name ?? "unknown",
          toolInput: (msg.request.input as Record<string, unknown>) ?? {},
          toolUseId: msg.request.tool_use_id ?? "",
          sessionId: event.payload.session_id,
          timestamp: Date.now(),
        });
      }
    });

    // Listen for usage updates from Rust
    listen<{ session_id: string; usage_totals: UsageTotals }>(
      "claude:usage",
      (event) => {
        const { session_id, usage_totals } = event.payload;
        set((state) => ({
          sessionUsage: {
            ...state.sessionUsage,
            [session_id]: usage_totals,
          },
        }));
      }
    );
  },
}));

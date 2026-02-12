import { useEffect, useMemo } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { CopilotKit } from "@copilotkit/react-core";
import { HttpAgent } from "@ag-ui/client";
import "@copilotkit/react-ui/styles.css";
import { useAppStore } from "./stores/appStore";
import { useSessionStore } from "./stores/sessionStore";
import {
  MessageSquare,
  Zap,
  FileText,
  Settings,
  Layers,
  LayoutDashboard,
  Info,
} from "lucide-react";

import AgentChat from "./tabs/AgentChat";
import Skills from "./tabs/Skills";
import ClaudeMd from "./tabs/ClaudeMd";
import SettingsTab from "./tabs/Settings";
import Sessions from "./tabs/Sessions";
import Dashboard from "./tabs/Dashboard";
import About from "./tabs/About";

const TABS = [
  { id: "chat", label: "Agent Chat", icon: MessageSquare, component: AgentChat },
  { id: "skills", label: "Skills", icon: Zap, component: Skills },
  { id: "claude-md", label: "CLAUDE.md", icon: FileText, component: ClaudeMd },
  { id: "settings", label: "Settings", icon: Settings, component: SettingsTab },
  { id: "sessions", label: "Sessions", icon: Layers, component: Sessions },
  { id: "dashboard", label: "Dashboard", icon: LayoutDashboard, component: Dashboard },
  { id: "about", label: "About", icon: Info, component: About },
] as const;

export default function App() {
  const { activeTab, setActiveTab, setPorts, runtimeUrl } = useAppStore();

  // Initialize session event listeners (Tauri events from Rust)
  useEffect(() => {
    useSessionStore.getState().initListeners();
  }, []);

  // Fetch ports from Rust on mount (servers start before frontend loads)
  useEffect(() => {
    invoke<{ ws_port: number; axum_port: number }>("get_ports").then((ports) => {
      if (ports.ws_port > 0 || ports.axum_port > 0) {
        setPorts(ports.ws_port, ports.axum_port);
      }
    });
  }, [setPorts]);

  // Also listen for port events (in case servers restart or start late)
  useEffect(() => {
    const unlistenAgui = listen<number>("agui:port", (event) => {
      const axumPort = event.payload;
      setPorts(useAppStore.getState().wsPort, axumPort);
    });

    const unlistenWs = listen<number>("ws:port", (event) => {
      setPorts(event.payload, useAppStore.getState().axumPort);
    });

    return () => {
      unlistenAgui.then((fn) => fn());
      unlistenWs.then((fn) => fn());
    };
  }, [setPorts]);

  // Create a local AG-UI agent pointing at our Axum backend.
  // HttpAgent.run() POSTs directly to the `url` — it does NOT append /run.
  // So we must set the full run endpoint URL.
  // Type assertion needed: top-level @ag-ui/client and the one bundled
  // inside @copilotkitnext/react are duplicate packages with identical APIs.
  const agents = useMemo(() => {
    if (!runtimeUrl) return undefined;
    return {
      default: new HttpAgent({
        url: `${runtimeUrl}/agent/default/run`,
        agentId: "default",
        description: "Claude Code AI agent",
      }),
    } as Record<string, any>;
  }, [runtimeUrl]);

  const ActiveComponent =
    TABS.find((t) => t.id === activeTab)?.component ?? AgentChat;

  const layout = (
    <div className="flex h-screen bg-slate-950 text-slate-200">
      {/* Sidebar */}
      <nav className="w-14 flex flex-col items-center py-3 bg-slate-900 border-r border-slate-800 shrink-0">
        {TABS.map((tab) => {
          const Icon = tab.icon;
          const isActive = activeTab === tab.id;
          return (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`w-10 h-10 mb-1 rounded-lg flex items-center justify-center transition-colors
                ${isActive
                  ? "bg-katara-600 text-white"
                  : "text-slate-500 hover:text-slate-300 hover:bg-slate-800"
                }`}
              title={tab.label}
            >
              <Icon size={20} />
            </button>
          );
        })}
      </nav>

      {/* Main content */}
      <main className="flex-1 overflow-hidden">
        <ActiveComponent />
      </main>
    </div>
  );

  // Wrap with CopilotKit when AG-UI backend is ready
  if (runtimeUrl) {
    return (
      <CopilotKit
        runtimeUrl={runtimeUrl}
        agent="default"
        showDevConsole={false}
        agents__unsafe_dev_only={agents}
      >
        {layout}
      </CopilotKit>
    );
  }

  return layout;
}

import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { LayoutDashboard } from "lucide-react";
import { useAppStore } from "../stores/appStore";
import { useSessionStore } from "../stores/sessionStore";

export default function Dashboard() {
  const { wsPort, axumPort, setPorts } = useAppStore();
  const sessions = useSessionStore((s) => s.sessions);

  // Fetch ports from Rust on mount (events may have fired before listeners registered)
  useEffect(() => {
    invoke<{ ws_port: number; axum_port: number }>("get_ports").then((ports) => {
      if (ports.ws_port > 0 || ports.axum_port > 0) {
        setPorts(ports.ws_port, ports.axum_port);
      }
    });
  }, [setPorts]);

  return (
    <div className="h-full flex flex-col">
      <header className="flex items-center gap-2 px-4 py-3 border-b border-slate-800 bg-slate-900">
        <LayoutDashboard size={18} className="text-emerald-400" />
        <h1 className="text-sm font-semibold">Dashboard</h1>
      </header>
      <div className="flex-1 p-6 overflow-y-auto">
        <h2 className="text-lg font-semibold mb-4">System Status</h2>

        <div className="grid grid-cols-2 gap-4 max-w-2xl">
          <div className="bg-slate-900 rounded-lg p-4 border border-slate-800">
            <div className="text-xs text-slate-500 uppercase tracking-wide">
              WebSocket Server
            </div>
            <div className="mt-2 flex items-center gap-2">
              <div
                className={`w-2 h-2 rounded-full ${
                  wsPort > 0 ? "bg-emerald-400" : "bg-slate-600"
                }`}
              />
              <span className="text-sm">
                {wsPort > 0 ? `Port ${wsPort}` : "Starting..."}
              </span>
            </div>
          </div>

          <div className="bg-slate-900 rounded-lg p-4 border border-slate-800">
            <div className="text-xs text-slate-500 uppercase tracking-wide">
              AG-UI Server
            </div>
            <div className="mt-2 flex items-center gap-2">
              <div
                className={`w-2 h-2 rounded-full ${
                  axumPort > 0 ? "bg-emerald-400" : "bg-slate-600"
                }`}
              />
              <span className="text-sm">
                {axumPort > 0 ? `Port ${axumPort}` : "Starting..."}
              </span>
            </div>
          </div>

          <div className="bg-slate-900 rounded-lg p-4 border border-slate-800">
            <div className="text-xs text-slate-500 uppercase tracking-wide">
              Active Sessions
            </div>
            <div className="mt-2 text-2xl font-bold text-katara-400">{sessions.length}</div>
          </div>

          <div className="bg-slate-900 rounded-lg p-4 border border-slate-800">
            <div className="text-xs text-slate-500 uppercase tracking-wide">
              Platform
            </div>
            <div className="mt-2 text-sm">Tauri + Rust</div>
          </div>
        </div>
      </div>
    </div>
  );
}

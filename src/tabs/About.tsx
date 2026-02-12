import { Info } from "lucide-react";

export default function About() {
  return (
    <div className="h-full flex flex-col">
      <header className="flex items-center gap-2 px-4 py-3 border-b border-slate-800 bg-slate-900">
        <Info size={18} className="text-slate-400" />
        <h1 className="text-sm font-semibold">About</h1>
      </header>
      <div className="flex-1 flex items-center justify-center">
        <div className="text-center max-w-md">
          <h2 className="text-3xl font-bold bg-gradient-to-r from-katara-400 to-blue-400 bg-clip-text text-transparent">
            Katara
          </h2>
          <p className="mt-2 text-slate-400">
            Self-Healing Workflow Platform for AI Coding Agents
          </p>
          <div className="mt-6 text-sm text-slate-500 space-y-1">
            <p>Version 0.1.0</p>
            <p>Tauri 2 + React 18 + Rust</p>
          </div>
          <div className="mt-6 text-xs text-slate-600 space-y-1">
            <p>CopilotKit for conversational UI</p>
            <p>AG-UI protocol for agent communication</p>
            <p>Claude Code CLI via WebSocket bridge</p>
          </div>
        </div>
      </div>
    </div>
  );
}

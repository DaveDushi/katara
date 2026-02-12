import { useState } from "react";
import {
  Layers,
  Plus,
  Trash2,
  Circle,
  MessageSquare,
  FolderOpen,
} from "lucide-react";
import { useSessionStore } from "../stores/sessionStore";
import { useAppStore } from "../stores/appStore";

export default function Sessions() {
  const {
    sessions,
    activeSessionId,
    setActiveSession,
    spawnSession,
    killSession,
  } = useSessionStore();
  const setActiveTab = useAppStore((s) => s.setActiveTab);
  const [workingDir, setWorkingDir] = useState("");
  const [spawning, setSpawning] = useState(false);

  const handleSpawn = async () => {
    const dir = workingDir.trim() || ".";
    setSpawning(true);
    try {
      await spawnSession(dir);
    } catch (e) {
      console.error("Failed to spawn session:", e);
    } finally {
      setSpawning(false);
    }
  };

  const handleActivate = (id: string) => {
    setActiveSession(id);
    setActiveTab("chat");
  };

  const statusColor = (status: string) => {
    switch (status) {
      case "Active":
        return "text-emerald-400";
      case "Connected":
      case "Idle":
        return "text-blue-400";
      case "Starting":
        return "text-yellow-400";
      case "Disconnected":
      case "Terminated":
        return "text-slate-500";
      default:
        return "text-red-400";
    }
  };

  return (
    <div className="h-full flex flex-col">
      <header className="flex items-center gap-2 px-4 py-3 border-b border-slate-800 bg-slate-900">
        <Layers size={18} className="text-purple-400" />
        <h1 className="text-sm font-semibold">Sessions</h1>
        <span className="ml-auto text-xs text-slate-500">
          {sessions.length} session{sessions.length !== 1 ? "s" : ""}
        </span>
      </header>

      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {/* New session form */}
        <div className="bg-slate-900 rounded-lg p-4 border border-slate-800">
          <h3 className="text-sm font-medium mb-3">New Session</h3>
          <div className="flex gap-2">
            <div className="flex-1 flex items-center gap-2 bg-slate-800 border border-slate-700 rounded-lg px-3">
              <FolderOpen size={14} className="text-slate-500 shrink-0" />
              <input
                value={workingDir}
                onChange={(e) => setWorkingDir(e.target.value)}
                placeholder="Working directory (default: current)"
                className="flex-1 bg-transparent py-2 text-sm text-slate-200 placeholder-slate-500 focus:outline-none"
                onKeyDown={(e) => e.key === "Enter" && handleSpawn()}
              />
            </div>
            <button
              onClick={handleSpawn}
              disabled={spawning}
              className="px-4 py-2 bg-katara-600 hover:bg-katara-500 disabled:bg-slate-700 text-white rounded-lg flex items-center gap-2 text-sm transition-colors"
            >
              <Plus size={14} />
              Spawn
            </button>
          </div>
        </div>

        {/* Session list */}
        {sessions.length === 0 ? (
          <div className="text-center text-slate-500 mt-8">
            <Layers size={36} className="mx-auto mb-3 text-slate-700" />
            <p className="text-sm">No active sessions</p>
            <p className="text-xs mt-1 text-slate-600">
              Spawn a session to get started
            </p>
          </div>
        ) : (
          <div className="space-y-2">
            {sessions.map((session) => (
              <div
                key={session.id}
                className={`bg-slate-900 rounded-lg p-4 border transition-colors cursor-pointer ${
                  session.id === activeSessionId
                    ? "border-katara-500"
                    : "border-slate-800 hover:border-slate-700"
                }`}
                onClick={() => handleActivate(session.id)}
              >
                <div className="flex items-center gap-3">
                  <Circle
                    size={8}
                    className={`fill-current shrink-0 ${statusColor(session.status)}`}
                  />
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <span className="text-sm font-mono text-slate-300">
                        {session.id.slice(0, 8)}...
                      </span>
                      <span
                        className={`text-xs ${statusColor(session.status)}`}
                      >
                        {session.status}
                      </span>
                      {session.id === activeSessionId && (
                        <span className="text-xs bg-katara-600/30 text-katara-400 px-2 py-0.5 rounded">
                          active
                        </span>
                      )}
                    </div>
                    <div className="text-xs text-slate-500 mt-1 truncate">
                      {session.working_dir}
                    </div>
                  </div>
                  <div className="flex items-center gap-1 shrink-0">
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        handleActivate(session.id);
                      }}
                      className="p-1.5 text-slate-500 hover:text-slate-300 hover:bg-slate-800 rounded"
                      title="Open chat"
                    >
                      <MessageSquare size={14} />
                    </button>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        killSession(session.id);
                      }}
                      className="p-1.5 text-slate-500 hover:text-red-400 hover:bg-slate-800 rounded"
                      title="Kill session"
                    >
                      <Trash2 size={14} />
                    </button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

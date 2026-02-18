import { Shield, ShieldCheck, ShieldOff, Eye } from "lucide-react";
import { useSessionStore } from "../stores/sessionStore";

const MODES = [
  {
    value: "default",
    label: "Default",
    icon: Shield,
    description: "Ask for all tool permissions",
    color: "text-slate-400",
  },
  {
    value: "plan",
    label: "Plan",
    icon: Eye,
    description: "Read-only, deny all tool use",
    color: "text-blue-400",
  },
  {
    value: "acceptEdits",
    label: "Accept Edits",
    icon: ShieldCheck,
    description: "Auto-approve file edits, ask for others",
    color: "text-amber-400",
  },
  {
    value: "bypassPermissions",
    label: "Bypass",
    icon: ShieldOff,
    description: "Auto-approve everything (use with caution)",
    color: "text-red-400",
  },
] as const;

export default function PermissionModeSelector() {
  const activeSessionId = useSessionStore((s) => s.activeSessionId);
  const sessions = useSessionStore((s) => s.sessions);
  const setPermissionMode = useSessionStore((s) => s.setPermissionMode);

  const session = sessions.find((s) => s.id === activeSessionId);
  const currentMode = session?.permission_mode ?? "default";

  if (!activeSessionId) return null;

  return (
    <div className="flex items-center gap-1">
      {MODES.map((mode) => {
        const Icon = mode.icon;
        const isActive = currentMode === mode.value;
        return (
          <button
            key={mode.value}
            onClick={() => setPermissionMode(activeSessionId, mode.value)}
            className={`flex items-center gap-1 px-2 py-1 rounded text-xs transition-colors ${
              isActive
                ? `bg-slate-700 ${mode.color} font-medium`
                : "text-slate-500 hover:text-slate-300 hover:bg-slate-800"
            }`}
            title={mode.description}
          >
            <Icon size={12} />
            {mode.label}
          </button>
        );
      })}
    </div>
  );
}

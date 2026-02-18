import { useState } from "react";
import {
  Shield,
  Check,
  X,
  ChevronDown,
  ChevronUp,
  Pencil,
} from "lucide-react";
import {
  useSessionStore,
  type PendingApproval,
} from "../stores/sessionStore";

export default function ToolApprovalBanner() {
  const pendingApprovals = useSessionStore((s) => s.pendingApprovals);
  const activeSessionId = useSessionStore((s) => s.activeSessionId);
  const approveTool = useSessionStore((s) => s.approveTool);
  const removePendingApproval = useSessionStore(
    (s) => s.removePendingApproval
  );

  const approval = pendingApprovals.find(
    (a) => a.sessionId === activeSessionId
  );

  if (!approval) return null;

  return (
    <ToolApprovalCard
      key={approval.requestId}
      approval={approval}
      onApprove={async (updatedInput) => {
        await approveTool(
          approval.sessionId,
          approval.requestId,
          true,
          updatedInput
        );
        removePendingApproval(approval.requestId);
      }}
      onDeny={async () => {
        await approveTool(
          approval.sessionId,
          approval.requestId,
          false
        );
        removePendingApproval(approval.requestId);
      }}
    />
  );
}

function ToolApprovalCard({
  approval,
  onApprove,
  onDeny,
}: {
  approval: PendingApproval;
  onApprove: (updatedInput?: Record<string, unknown>) => Promise<void>;
  onDeny: () => Promise<void>;
}) {
  const [expanded, setExpanded] = useState(false);
  const [editing, setEditing] = useState(false);
  const [editedInput, setEditedInput] = useState(
    JSON.stringify(approval.toolInput, null, 2)
  );
  const [loading, setLoading] = useState(false);

  const handleApprove = async () => {
    setLoading(true);
    try {
      if (editing) {
        const parsed = JSON.parse(editedInput);
        await onApprove(parsed);
      } else {
        await onApprove();
      }
    } catch (e) {
      console.error("Failed to approve tool:", e);
    }
    setLoading(false);
  };

  const handleDeny = async () => {
    setLoading(true);
    try {
      await onDeny();
    } catch (e) {
      console.error("Failed to deny tool:", e);
    }
    setLoading(false);
  };

  const inputStr = JSON.stringify(approval.toolInput, null, 2);
  const isLong = inputStr.length > 200;

  return (
    <div className="mx-2 mb-2 rounded-lg border border-amber-700/50 bg-amber-950/30 overflow-hidden">
      {/* Header */}
      <div className="flex items-center gap-2 px-3 py-2 bg-amber-900/20">
        <Shield size={14} className="text-amber-400 shrink-0" />
        <span className="text-xs font-medium text-amber-300">
          Tool Approval Required
        </span>
        <span className="ml-auto text-xs text-amber-500 font-mono">
          {approval.toolName}
        </span>
      </div>

      {/* Input preview */}
      <div className="px-3 py-2 border-t border-amber-800/30">
        {editing ? (
          <textarea
            className="w-full px-2 py-1.5 bg-slate-800/80 rounded text-xs text-slate-200 font-mono border border-slate-700 focus:border-amber-500/50 outline-none resize-y"
            style={{ minHeight: "80px" }}
            value={editedInput}
            onChange={(e) => setEditedInput(e.target.value)}
          />
        ) : (
          <pre
            className={`text-xs text-slate-400 font-mono overflow-x-auto ${
              !expanded && isLong ? "max-h-[60px] overflow-hidden" : ""
            }`}
          >
            {inputStr}
          </pre>
        )}
        {isLong && !editing && (
          <button
            onClick={() => setExpanded(!expanded)}
            className="flex items-center gap-1 text-xs text-amber-500 hover:text-amber-400 mt-1"
          >
            {expanded ? <ChevronUp size={12} /> : <ChevronDown size={12} />}
            {expanded ? "Collapse" : "Show all"}
          </button>
        )}
      </div>

      {/* Action buttons */}
      <div className="flex items-center gap-2 px-3 py-2 border-t border-amber-800/30 bg-amber-950/20">
        <button
          onClick={handleApprove}
          disabled={loading}
          className="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-medium bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50 text-white transition-colors"
        >
          <Check size={12} />
          Approve
        </button>
        <button
          onClick={handleDeny}
          disabled={loading}
          className="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-medium bg-red-700 hover:bg-red-600 disabled:opacity-50 text-white transition-colors"
        >
          <X size={12} />
          Deny
        </button>
        <button
          onClick={() => setEditing(!editing)}
          className="flex items-center gap-1.5 px-2 py-1.5 rounded-md text-xs text-slate-400 hover:text-slate-200 hover:bg-slate-700/50 transition-colors ml-auto"
        >
          <Pencil size={12} />
          {editing ? "Preview" : "Edit"}
        </button>
      </div>
    </div>
  );
}

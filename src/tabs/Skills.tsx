import { Zap } from "lucide-react";

export default function Skills() {
  return (
    <div className="h-full flex flex-col">
      <header className="flex items-center gap-2 px-4 py-3 border-b border-slate-800 bg-slate-900">
        <Zap size={18} className="text-yellow-400" />
        <h1 className="text-sm font-semibold">Skills</h1>
      </header>
      <div className="flex-1 flex items-center justify-center text-slate-500">
        <div className="text-center">
          <Zap size={48} className="mx-auto mb-4 text-slate-700" />
          <p className="text-lg font-medium">Skills</p>
          <p className="text-sm mt-1">
            Self-healing workflow skills with dynamic UI generation
          </p>
          <p className="text-xs mt-3 text-slate-600">
            Phase 5 implementation
          </p>
        </div>
      </div>
    </div>
  );
}

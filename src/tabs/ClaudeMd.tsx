import { FileText } from "lucide-react";

export default function ClaudeMd() {
  return (
    <div className="h-full flex flex-col">
      <header className="flex items-center gap-2 px-4 py-3 border-b border-slate-800 bg-slate-900">
        <FileText size={18} className="text-blue-400" />
        <h1 className="text-sm font-semibold">CLAUDE.md</h1>
      </header>
      <div className="flex-1 flex items-center justify-center text-slate-500">
        <div className="text-center">
          <FileText size={48} className="mx-auto mb-4 text-slate-700" />
          <p className="text-lg font-medium">CLAUDE.md Editor</p>
          <p className="text-sm mt-1">
            4-level configuration editor with Monaco
          </p>
          <p className="text-xs mt-3 text-slate-600">
            Phase 6 implementation
          </p>
        </div>
      </div>
    </div>
  );
}

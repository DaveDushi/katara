import { TerminalSquare } from "lucide-react";

export default function Terminal() {
  return (
    <div className="h-full flex flex-col">
      <header className="flex items-center gap-2 px-4 py-3 border-b border-slate-800 bg-slate-900">
        <TerminalSquare size={18} className="text-green-400" />
        <h1 className="text-sm font-semibold">Terminal</h1>
      </header>
      <div className="flex-1 flex items-center justify-center text-slate-500">
        <div className="text-center">
          <TerminalSquare size={48} className="mx-auto mb-4 text-slate-700" />
          <p className="text-lg font-medium">Terminal</p>
          <p className="text-sm mt-1">
            Embedded terminal via portable-pty + xterm.js
          </p>
          <p className="text-xs mt-3 text-slate-600">
            Phase 3 implementation
          </p>
        </div>
      </div>
    </div>
  );
}

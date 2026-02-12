import { Settings as SettingsIcon } from "lucide-react";

export default function SettingsTab() {
  return (
    <div className="h-full flex flex-col">
      <header className="flex items-center gap-2 px-4 py-3 border-b border-slate-800 bg-slate-900">
        <SettingsIcon size={18} className="text-slate-400" />
        <h1 className="text-sm font-semibold">Settings</h1>
      </header>
      <div className="flex-1 flex items-center justify-center text-slate-500">
        <div className="text-center">
          <SettingsIcon size={48} className="mx-auto mb-4 text-slate-700" />
          <p className="text-lg font-medium">Settings</p>
          <p className="text-sm mt-1">
            Visual configuration editor
          </p>
          <p className="text-xs mt-3 text-slate-600">
            Phase 6 implementation
          </p>
        </div>
      </div>
    </div>
  );
}

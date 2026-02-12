import { useCallback } from "react";
import { MessageSquare, Plus, Circle } from "lucide-react";
import { CopilotChat } from "@copilotkit/react-ui";
import { useCopilotAction } from "@copilotkit/react-core";
import { useSessionStore } from "../stores/sessionStore";
import { useAppStore } from "../stores/appStore";

/**
 * Catch-all Gen-UI renderer: renders ANY tool call from Claude as a
 * collapsible card inline in the chat. This is the baseline — specific
 * domain actions (registered separately) take priority over this.
 */
function useCatchAllToolRenderer() {
  useCopilotAction({
    name: "*",
    render: ({ name, args, status }: { name: string; args: Record<string, unknown>; status: string }) => {
      const isRunning = status === "inProgress" || status === "executing";
      return (
        <div className="my-2 rounded-lg border border-slate-700 bg-slate-800/80 overflow-hidden text-sm">
          <div className="flex items-center gap-2 px-3 py-2 bg-slate-900/50 border-b border-slate-700">
            <div
              className={`w-2 h-2 rounded-full ${isRunning ? "bg-amber-400 animate-pulse" : "bg-emerald-400"}`}
            />
            <span className="font-mono text-xs text-slate-300">{name}</span>
            <span className="ml-auto text-xs text-slate-500">
              {isRunning ? "running..." : "done"}
            </span>
          </div>
          {args && Object.keys(args).length > 0 && (
            <pre className="px-3 py-2 text-xs text-slate-400 overflow-x-auto max-h-48 overflow-y-auto">
              {JSON.stringify(args, null, 2)}
            </pre>
          )}
        </div>
      );
    },
  });
}

/**
 * Example domain Gen-UI actions for an Amazon listing agent.
 * Claude can invoke these to render rich UI inline in the chat.
 */
function useAmazonListingActions() {
  useCopilotAction({
    name: "render_product_input",
    description:
      "Render a product input form for the user to fill in product details like title, brand, features, and upload images",
    parameters: [
      { name: "title", type: "string", description: "Suggested product title", required: false },
      { name: "brand", type: "string", description: "Product brand", required: false },
      { name: "category", type: "string", description: "Product category", required: false },
      {
        name: "features",
        type: "string[]",
        description: "Key product features/bullet points",
        required: false,
      },
    ],
    render: ({ args, status }) => {
      const isComplete = status === "complete";
      return (
        <div className="my-3 rounded-xl border border-blue-800/50 bg-gradient-to-b from-blue-950/40 to-slate-900/60 overflow-hidden">
          <div className="px-4 py-3 border-b border-blue-800/30 bg-blue-950/30">
            <h3 className="text-sm font-semibold text-blue-300">
              Product Input
            </h3>
            <p className="text-xs text-slate-400 mt-0.5">
              {isComplete ? "Product details captured" : "Gathering product info..."}
            </p>
          </div>
          <div className="p-4 space-y-3">
            {args.title && (
              <div>
                <label className="text-xs text-slate-500 uppercase tracking-wide">
                  Title
                </label>
                <div className="mt-1 px-3 py-2 bg-slate-800 rounded-lg text-sm text-slate-200 border border-slate-700">
                  {args.title}
                </div>
              </div>
            )}
            {args.brand && (
              <div>
                <label className="text-xs text-slate-500 uppercase tracking-wide">
                  Brand
                </label>
                <div className="mt-1 px-3 py-2 bg-slate-800 rounded-lg text-sm text-slate-200 border border-slate-700">
                  {args.brand}
                </div>
              </div>
            )}
            {args.category && (
              <div>
                <label className="text-xs text-slate-500 uppercase tracking-wide">
                  Category
                </label>
                <div className="mt-1 px-3 py-2 bg-slate-800 rounded-lg text-sm text-slate-200 border border-slate-700">
                  {args.category}
                </div>
              </div>
            )}
            {args.features && args.features.length > 0 && (
              <div>
                <label className="text-xs text-slate-500 uppercase tracking-wide">
                  Key Features
                </label>
                <ul className="mt-1 space-y-1">
                  {(args.features as string[]).map((f: string, i: number) => (
                    <li
                      key={i}
                      className="flex items-start gap-2 px-3 py-1.5 bg-slate-800 rounded-lg text-sm text-slate-200 border border-slate-700"
                    >
                      <span className="text-blue-400 mt-0.5">•</span>
                      {f}
                    </li>
                  ))}
                </ul>
              </div>
            )}
          </div>
        </div>
      );
    },
    handler: async ({ title, brand, category, features }) => {
      return JSON.stringify({ title, brand, category, features });
    },
  });

  useCopilotAction({
    name: "render_listing_output",
    description:
      "Render a polished Amazon-style product listing with title, bullet points, description, and pricing",
    parameters: [
      { name: "title", type: "string", description: "Listing title" },
      {
        name: "bullets",
        type: "string[]",
        description: "Product bullet points (5 recommended)",
      },
      {
        name: "description",
        type: "string",
        description: "Product description (HTML or plain text)",
      },
      { name: "price", type: "string", description: "Product price", required: false },
      { name: "rating", type: "number", description: "Star rating 1-5", required: false },
    ],
    render: ({ args, status }) => {
      const isComplete = status === "complete";
      return (
        <div className="my-3 rounded-xl border border-emerald-800/50 bg-gradient-to-b from-emerald-950/30 to-slate-900/60 overflow-hidden">
          <div className="px-4 py-3 border-b border-emerald-800/30 bg-emerald-950/30">
            <h3 className="text-sm font-semibold text-emerald-300">
              Amazon Listing Preview
            </h3>
            <p className="text-xs text-slate-400 mt-0.5">
              {isComplete ? "Listing generated" : "Generating listing..."}
            </p>
          </div>
          <div className="p-4 space-y-4">
            {args.title && (
              <h2 className="text-base font-medium text-slate-100 leading-snug">
                {args.title}
              </h2>
            )}

            {(args.rating || args.price) && (
              <div className="flex items-center gap-4">
                {args.rating && (
                  <div className="flex items-center gap-1">
                    {[1, 2, 3, 4, 5].map((star) => (
                      <span
                        key={star}
                        className={`text-sm ${
                          star <= (args.rating ?? 0)
                            ? "text-amber-400"
                            : "text-slate-600"
                        }`}
                      >
                        ★
                      </span>
                    ))}
                    <span className="text-xs text-slate-400 ml-1">
                      {args.rating}/5
                    </span>
                  </div>
                )}
                {args.price && (
                  <span className="text-lg font-bold text-emerald-400">
                    {args.price}
                  </span>
                )}
              </div>
            )}

            {args.bullets && (args.bullets as string[]).length > 0 && (
              <div className="space-y-1.5">
                <label className="text-xs text-slate-500 uppercase tracking-wide">
                  Key Features
                </label>
                {(args.bullets as string[]).map((b: string, i: number) => (
                  <div
                    key={i}
                    className="flex items-start gap-2 text-sm text-slate-200"
                  >
                    <span className="text-emerald-400 mt-0.5 shrink-0">
                      •
                    </span>
                    <span>{b}</span>
                  </div>
                ))}
              </div>
            )}

            {args.description && (
              <div>
                <label className="text-xs text-slate-500 uppercase tracking-wide">
                  Description
                </label>
                <div className="mt-1 text-sm text-slate-300 leading-relaxed whitespace-pre-wrap">
                  {args.description}
                </div>
              </div>
            )}
          </div>
        </div>
      );
    },
    handler: async ({ title, bullets, description, price, rating }) => {
      return JSON.stringify({ title, bullets, description, price, rating });
    },
  });
}

/**
 * Inner chat component — only rendered when CopilotKit provider is active.
 * All useCopilotAction hooks live here so they're safely inside the provider.
 */
function AgentChatInner() {
  const { activeSessionId, sessions, interruptSession } = useSessionStore();

  const activeSession = sessions.find((s) => s.id === activeSessionId);
  const statusColor =
    activeSession?.status === "Active"
      ? "text-emerald-400"
      : activeSession?.status === "Connected" ||
          activeSession?.status === "Idle"
        ? "text-blue-400"
        : activeSession?.status === "Starting"
          ? "text-yellow-400"
          : "text-slate-500";

  // Register Gen-UI action hooks (safe: we're inside CopilotKit provider)
  useCatchAllToolRenderer();
  useAmazonListingActions();

  const handleStopGeneration = useCallback(async () => {
    if (!activeSessionId) return;
    try {
      await interruptSession(activeSessionId);
    } catch (e: any) {
      console.error("Failed to interrupt:", e);
    }
  }, [activeSessionId, interruptSession]);

  return (
    <div className="h-full flex flex-col">
      <header className="flex items-center gap-2 px-4 py-3 border-b border-slate-800 bg-slate-900">
        <MessageSquare size={18} className="text-katara-400" />
        <h1 className="text-sm font-semibold">Agent Chat</h1>
        <div className="ml-auto flex items-center gap-2 text-xs">
          <Circle size={8} className={`fill-current ${statusColor}`} />
          <span className="text-slate-400">
            {activeSession?.status || "Unknown"}
          </span>
          <span className="text-slate-600">
            {activeSessionId!.slice(0, 8)}...
          </span>
        </div>
      </header>

      <div className="flex-1 overflow-hidden copilotkit-chat-container">
        <CopilotChat
          className="h-full"
          instructions="You are Claude Code running inside Katara. You have access to tool calls that can render rich UI components inline in the chat. When appropriate, use render_product_input to show product input forms and render_listing_output to show generated listings."
          onStopGeneration={handleStopGeneration}
          labels={{
            initial: "How can I help you today?",
            placeholder: "Send a message...",
          }}
        />
      </div>
    </div>
  );
}

/**
 * Outer wrapper — handles the "no session" / "no CopilotKit" states
 * without calling any CopilotKit hooks.
 */
export default function AgentChat() {
  const { activeSessionId, spawnSession } = useSessionStore();
  const runtimeUrl = useAppStore((s) => s.runtimeUrl);

  const handleSpawnSession = useCallback(async () => {
    try {
      await spawnSession(".");
    } catch (e: any) {
      console.error("Failed to spawn session:", e);
    }
  }, [spawnSession]);

  // No CopilotKit provider or no session — show start screen
  if (!runtimeUrl || !activeSessionId) {
    return (
      <div className="h-full flex flex-col">
        <header className="flex items-center gap-2 px-4 py-3 border-b border-slate-800 bg-slate-900">
          <MessageSquare size={18} className="text-katara-400" />
          <h1 className="text-sm font-semibold">Agent Chat</h1>
        </header>
        <div className="flex-1 flex items-center justify-center">
          <div className="text-center">
            <MessageSquare
              size={48}
              className="mx-auto mb-4 text-slate-700"
            />
            <p className="text-lg font-medium text-slate-300">
              Start a Claude Code Session
            </p>
            <p className="text-sm mt-1 text-slate-500 max-w-sm">
              Spawn a new session to start chatting with Claude Code.
              {!runtimeUrl && " Waiting for backend to start..."}
            </p>
            <button
              onClick={handleSpawnSession}
              disabled={!runtimeUrl}
              className="mt-6 px-4 py-2 bg-katara-600 hover:bg-katara-500 disabled:bg-slate-700 disabled:text-slate-500 text-white rounded-lg flex items-center gap-2 mx-auto transition-colors"
            >
              <Plus size={16} />
              New Session
            </button>
          </div>
        </div>
      </div>
    );
  }

  // runtimeUrl exists + activeSession exists = CopilotKit provider is active
  return <AgentChatInner />;
}

import { useCallback, useState } from "react";
import { MessageSquare, Plus, Circle, Package, FileText, Sparkles } from "lucide-react";
import { CopilotChat } from "@copilotkit/react-ui";
import { useFrontendTool } from "@copilotkit/react-core";
import { useSessionStore } from "../stores/sessionStore";
import { useAppStore } from "../stores/appStore";

// ─── Workspace state types ─────────────────────────────────────────────

interface ProductData {
  title?: string;
  brand?: string;
  category?: string;
  features?: string[];
}

interface ListingData {
  title?: string;
  bullets?: string[];
  description?: string;
  price?: string;
  rating?: number;
}

// ─── Safe array helper ─────────────────────────────────────────────────

function toStringArray(val: unknown): string[] {
  if (Array.isArray(val)) return val.map(String);
  if (typeof val === "string") {
    try {
      const parsed = JSON.parse(val);
      if (Array.isArray(parsed)) return parsed.map(String);
    } catch {
      // comma-separated fallback
      return val.split(",").map((s) => s.trim()).filter(Boolean);
    }
  }
  return [];
}

// ─── Workspace panels ──────────────────────────────────────────────────

function ProductInputPanel({ data, status }: { data: ProductData | null; status: string }) {
  const features = toStringArray(data?.features);
  const isLoading = status === "loading";

  return (
    <div className="rounded-xl border border-slate-700/50 bg-slate-900/50 overflow-hidden">
      <div className="flex items-center gap-2 px-4 py-3 border-b border-slate-700/50 bg-slate-800/30">
        <Package size={16} className="text-blue-400" />
        <h3 className="text-sm font-semibold text-slate-200">Product Input</h3>
        {isLoading && (
          <div className="ml-auto flex items-center gap-1.5">
            <div className="w-1.5 h-1.5 rounded-full bg-blue-400 animate-pulse" />
            <span className="text-xs text-slate-500">Gathering info...</span>
          </div>
        )}
      </div>
      <div className="p-4 space-y-3">
        {!data ? (
          <p className="text-sm text-slate-500 text-center py-6">
            Ask the agent to gather product details. They'll appear here.
          </p>
        ) : (
          <>
            {data.title && (
              <Field label="Title" value={data.title} />
            )}
            {data.brand && (
              <Field label="Brand" value={data.brand} />
            )}
            {data.category && (
              <Field label="Category" value={data.category} />
            )}
            {features.length > 0 && (
              <div>
                <label className="text-xs text-slate-500 uppercase tracking-wide">Key Features</label>
                <ul className="mt-1 space-y-1">
                  {features.map((f, i) => (
                    <li key={i} className="flex items-start gap-2 px-3 py-1.5 bg-slate-800/60 rounded-lg text-sm text-slate-200 border border-slate-700/50">
                      <span className="text-blue-400 mt-0.5">•</span>
                      {f}
                    </li>
                  ))}
                </ul>
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
}

function ListingOutputPanel({ data, status }: { data: ListingData | null; status: string }) {
  const bullets = toStringArray(data?.bullets);
  const isLoading = status === "loading";

  return (
    <div className="rounded-xl border border-slate-700/50 bg-slate-900/50 overflow-hidden">
      <div className="flex items-center gap-2 px-4 py-3 border-b border-slate-700/50 bg-slate-800/30">
        <FileText size={16} className="text-emerald-400" />
        <h3 className="text-sm font-semibold text-slate-200">Listing Preview</h3>
        {isLoading && (
          <div className="ml-auto flex items-center gap-1.5">
            <div className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" />
            <span className="text-xs text-slate-500">Generating...</span>
          </div>
        )}
      </div>
      <div className="p-4 space-y-4">
        {!data ? (
          <p className="text-sm text-slate-500 text-center py-6">
            The generated listing will appear here once the agent creates it.
          </p>
        ) : (
          <>
            {data.title && (
              <h2 className="text-base font-medium text-slate-100 leading-snug">
                {data.title}
              </h2>
            )}

            {(data.rating || data.price) && (
              <div className="flex items-center gap-4">
                {data.rating && (
                  <div className="flex items-center gap-1">
                    {[1, 2, 3, 4, 5].map((star) => (
                      <span key={star} className={`text-sm ${star <= (data.rating ?? 0) ? "text-amber-400" : "text-slate-600"}`}>
                        ★
                      </span>
                    ))}
                    <span className="text-xs text-slate-400 ml-1">{data.rating}/5</span>
                  </div>
                )}
                {data.price && (
                  <span className="text-lg font-bold text-emerald-400">{data.price}</span>
                )}
              </div>
            )}

            {bullets.length > 0 && (
              <div className="space-y-1.5">
                <label className="text-xs text-slate-500 uppercase tracking-wide">Key Features</label>
                {bullets.map((b, i) => (
                  <div key={i} className="flex items-start gap-2 text-sm text-slate-200">
                    <span className="text-emerald-400 mt-0.5 shrink-0">•</span>
                    <span>{b}</span>
                  </div>
                ))}
              </div>
            )}

            {data.description && (
              <div>
                <label className="text-xs text-slate-500 uppercase tracking-wide">Description</label>
                <div className="mt-1 text-sm text-slate-300 leading-relaxed whitespace-pre-wrap">
                  {data.description}
                </div>
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
}

function Field({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <label className="text-xs text-slate-500 uppercase tracking-wide">{label}</label>
      <div className="mt-1 px-3 py-2 bg-slate-800/60 rounded-lg text-sm text-slate-200 border border-slate-700/50">
        {value}
      </div>
    </div>
  );
}

// ─── Inner component (inside CopilotKit provider) ──────────────────────

function AgentChatInner() {
  const { activeSessionId, sessions, interruptSession } = useSessionStore();

  const activeSession = sessions.find((s) => s.id === activeSessionId);
  const statusColor =
    activeSession?.status === "Active"
      ? "text-emerald-400"
      : activeSession?.status === "Connected" || activeSession?.status === "Idle"
        ? "text-blue-400"
        : activeSession?.status === "Starting"
          ? "text-yellow-400"
          : "text-slate-500";

  // Workspace state — updated by Gen-UI tools
  const [productData, setProductData] = useState<ProductData | null>(null);
  const [productStatus, setProductStatus] = useState("idle");
  const [listingData, setListingData] = useState<ListingData | null>(null);
  const [listingStatus, setListingStatus] = useState("idle");

  // ─── Gen-UI tools (useFrontendTool) ───────────────────────────────

  useFrontendTool({
    name: "render_product_input",
    description:
      "Render a product input form showing product details like title, brand, features, and category. Call this to display gathered product information in the workspace.",
    parameters: [
      { name: "title", type: "string" as const, description: "Suggested product title", required: false },
      { name: "brand", type: "string" as const, description: "Product brand", required: false },
      { name: "category", type: "string" as const, description: "Product category", required: false },
      { name: "features", type: "string[]" as const, description: "Key product features/bullet points", required: false },
    ],
    handler: async ({ title, brand, category, features }) => {
      const data: ProductData = {
        title: title as string | undefined,
        brand: brand as string | undefined,
        category: category as string | undefined,
        features: toStringArray(features),
      };
      setProductData(data);
      setProductStatus("complete");
      return data;
    },
    render: ({ status, args }) => {
      // Update workspace on inProgress
      if (status === "inProgress" || status === "executing") {
        setProductStatus("loading");
        if (args) {
          setProductData({
            title: args.title as string | undefined,
            brand: args.brand as string | undefined,
            category: args.category as string | undefined,
            features: toStringArray(args.features),
          });
        }
      }
      // Inline chat indicator (minimal — main view is in workspace)
      const isRunning = status === "inProgress" || status === "executing";
      return (
        <div className="my-1 px-3 py-2 rounded-lg bg-blue-950/30 border border-blue-800/30 text-xs text-blue-300 flex items-center gap-2">
          <Package size={14} />
          {isRunning ? "Updating product input..." : "Product input updated"}
          {isRunning && <div className="w-1.5 h-1.5 rounded-full bg-blue-400 animate-pulse ml-auto" />}
        </div>
      );
    },
  });

  useFrontendTool({
    name: "render_listing_output",
    description:
      "Render a polished Amazon-style product listing with title, bullet points, description, and pricing in the workspace.",
    parameters: [
      { name: "title", type: "string" as const, description: "Listing title" },
      { name: "bullets", type: "string[]" as const, description: "Product bullet points (5 recommended)" },
      { name: "description", type: "string" as const, description: "Product description" },
      { name: "price", type: "string" as const, description: "Product price", required: false },
      { name: "rating", type: "number" as const, description: "Star rating 1-5", required: false },
    ],
    handler: async ({ title, bullets, description, price, rating }) => {
      const data: ListingData = {
        title: title as string | undefined,
        bullets: toStringArray(bullets),
        description: description as string | undefined,
        price: price as string | undefined,
        rating: rating as number | undefined,
      };
      setListingData(data);
      setListingStatus("complete");
      return data;
    },
    render: ({ status, args }) => {
      if (status === "inProgress" || status === "executing") {
        setListingStatus("loading");
        if (args) {
          setListingData({
            title: args.title as string | undefined,
            bullets: toStringArray(args.bullets),
            description: args.description as string | undefined,
            price: args.price as string | undefined,
            rating: args.rating as number | undefined,
          });
        }
      }
      const isRunning = status === "inProgress" || status === "executing";
      return (
        <div className="my-1 px-3 py-2 rounded-lg bg-emerald-950/30 border border-emerald-800/30 text-xs text-emerald-300 flex items-center gap-2">
          <FileText size={14} />
          {isRunning ? "Generating listing..." : "Listing generated"}
          {isRunning && <div className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse ml-auto" />}
        </div>
      );
    },
  });

  // Catch-all: render any other tool as a compact card in chat
  useFrontendTool({
    name: "*",
    render: (props: any) => {
      const { name, args, status } = props;
      const isRunning = status === "inProgress" || status === "executing";
      return (
        <div className="my-2 rounded-lg border border-slate-700 bg-slate-800/80 overflow-hidden text-sm">
          <div className="flex items-center gap-2 px-3 py-2 bg-slate-900/50 border-b border-slate-700">
            <div className={`w-2 h-2 rounded-full ${isRunning ? "bg-amber-400 animate-pulse" : "bg-emerald-400"}`} />
            <span className="font-mono text-xs text-slate-300">{name ?? "tool"}</span>
            <span className="ml-auto text-xs text-slate-500">{isRunning ? "running..." : "done"}</span>
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

  const handleStopGeneration = useCallback(async () => {
    if (!activeSessionId) return;
    try {
      await interruptSession(activeSessionId);
    } catch (e: any) {
      console.error("Failed to interrupt:", e);
    }
  }, [activeSessionId, interruptSession]);

  return (
    <div className="h-full flex">
      {/* Left: Gen-UI Workspace */}
      <div className="flex-1 flex flex-col overflow-hidden">
        <header className="flex items-center gap-2 px-4 py-3 border-b border-slate-800 bg-slate-900 shrink-0">
          <Sparkles size={18} className="text-katara-400" />
          <h1 className="text-sm font-semibold">Agent Workspace</h1>
          <div className="ml-auto flex items-center gap-2 text-xs">
            <Circle size={8} className={`fill-current ${statusColor}`} />
            <span className="text-slate-400">{activeSession?.status || "Unknown"}</span>
            <span className="text-slate-600">{activeSessionId!.slice(0, 8)}...</span>
          </div>
        </header>

        <div className="flex-1 overflow-y-auto p-4 space-y-4">
          <ProductInputPanel data={productData} status={productStatus} />
          <ListingOutputPanel data={listingData} status={listingStatus} />
        </div>
      </div>

      {/* Right: Chat panel */}
      <div className="w-[380px] border-l border-slate-800 flex flex-col shrink-0 copilotkit-chat-container">
        <div className="flex items-center gap-2 px-4 py-3 border-b border-slate-800 bg-slate-900 shrink-0">
          <MessageSquare size={16} className="text-slate-400" />
          <span className="text-xs font-medium text-slate-300">Chat</span>
        </div>
        <div className="flex-1 overflow-hidden">
          <CopilotChat
            className="h-full"
            instructions="You are Claude Code running inside Katara. You can render rich UI in the workspace using these tools: render_product_input (to show product details) and render_listing_output (to show a generated listing). Use these tools whenever the user asks about products or listings."
            onStopGeneration={handleStopGeneration}
            labels={{
              initial: "Describe a product and I'll create a listing for it.",
              placeholder: "Send a message...",
            }}
          />
        </div>
      </div>
    </div>
  );
}

// ─── Outer wrapper ─────────────────────────────────────────────────────

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

  if (!runtimeUrl || !activeSessionId) {
    return (
      <div className="h-full flex flex-col">
        <header className="flex items-center gap-2 px-4 py-3 border-b border-slate-800 bg-slate-900">
          <Sparkles size={18} className="text-katara-400" />
          <h1 className="text-sm font-semibold">Agent Workspace</h1>
        </header>
        <div className="flex-1 flex items-center justify-center">
          <div className="text-center">
            <Sparkles size={48} className="mx-auto mb-4 text-slate-700" />
            <p className="text-lg font-medium text-slate-300">Start a Claude Code Session</p>
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

  return <AgentChatInner />;
}

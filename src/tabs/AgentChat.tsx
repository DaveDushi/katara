import { useCallback, useState } from "react";
import { MessageSquare, Plus, Circle, Package, FileText, Sparkles, X, PlusCircle, DollarSign } from "lucide-react";
import { CopilotChat } from "@copilotkit/react-ui";
import { useFrontendTool, useCopilotReadable } from "@copilotkit/react-core";
import { useSessionStore } from "../stores/sessionStore";
import { useAppStore } from "../stores/appStore";
import ToolApprovalBanner from "../components/ToolApprovalBanner";
import PermissionModeSelector from "../components/PermissionModeSelector";

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

// ─── Shared input styles ───────────────────────────────────────────────

const inputClass =
  "w-full mt-1 px-3 py-2 bg-slate-800/60 rounded-lg text-sm text-slate-200 border border-slate-700/50 focus:border-blue-500/50 focus:ring-1 focus:ring-blue-500/20 outline-none placeholder:text-slate-600 transition-colors";

const textareaClass =
  "w-full mt-1 px-3 py-2 bg-slate-800/60 rounded-lg text-sm text-slate-200 border border-slate-700/50 focus:border-emerald-500/50 focus:ring-1 focus:ring-emerald-500/20 outline-none placeholder:text-slate-600 resize-y min-h-[80px] transition-colors";

// ─── Workspace panels ──────────────────────────────────────────────────

function ProductInputPanel({
  data,
  status,
  onChange,
}: {
  data: ProductData | null;
  status: string;
  onChange: (updated: ProductData) => void;
}) {
  const features = toStringArray(data?.features);
  const isLoading = status === "loading";

  const update = (field: keyof ProductData, value: string | string[]) => {
    onChange({ ...data, [field]: value });
  };

  const updateFeature = (index: number, value: string) => {
    const next = [...features];
    next[index] = value;
    update("features", next);
  };

  const removeFeature = (index: number) => {
    update("features", features.filter((_, i) => i !== index));
  };

  const addFeature = () => {
    update("features", [...features, ""]);
  };

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
        <div>
          <label className="text-xs text-slate-500 uppercase tracking-wide">Title</label>
          <input
            className={inputClass}
            placeholder="Product title"
            value={data?.title ?? ""}
            onChange={(e) => update("title", e.target.value)}
          />
        </div>
        <div className="grid grid-cols-2 gap-3">
          <div>
            <label className="text-xs text-slate-500 uppercase tracking-wide">Brand</label>
            <input
              className={inputClass}
              placeholder="Brand name"
              value={data?.brand ?? ""}
              onChange={(e) => update("brand", e.target.value)}
            />
          </div>
          <div>
            <label className="text-xs text-slate-500 uppercase tracking-wide">Category</label>
            <input
              className={inputClass}
              placeholder="Product category"
              value={data?.category ?? ""}
              onChange={(e) => update("category", e.target.value)}
            />
          </div>
        </div>
        <div>
          <div className="flex items-center justify-between">
            <label className="text-xs text-slate-500 uppercase tracking-wide">Key Features</label>
            <button
              onClick={addFeature}
              className="flex items-center gap-1 text-xs text-blue-400 hover:text-blue-300 transition-colors"
            >
              <PlusCircle size={12} />
              Add
            </button>
          </div>
          <div className="mt-1 space-y-1.5">
            {features.length === 0 && (
              <p className="text-xs text-slate-600 py-2">No features yet — add manually or let the agent fill them in.</p>
            )}
            {features.map((f, i) => (
              <div key={i} className="flex items-center gap-1.5">
                <span className="text-blue-400 text-xs shrink-0">•</span>
                <input
                  className="flex-1 px-2 py-1.5 bg-slate-800/60 rounded-lg text-sm text-slate-200 border border-slate-700/50 focus:border-blue-500/50 focus:ring-1 focus:ring-blue-500/20 outline-none placeholder:text-slate-600 transition-colors"
                  value={f}
                  placeholder="Feature description"
                  onChange={(e) => updateFeature(i, e.target.value)}
                />
                <button
                  onClick={() => removeFeature(i)}
                  className="p-1 text-slate-600 hover:text-red-400 transition-colors shrink-0"
                >
                  <X size={14} />
                </button>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

function ListingOutputPanel({
  data,
  status,
  onChange,
}: {
  data: ListingData | null;
  status: string;
  onChange: (updated: ListingData) => void;
}) {
  const bullets = toStringArray(data?.bullets);
  const isLoading = status === "loading";

  const update = (field: keyof ListingData, value: string | string[] | number) => {
    onChange({ ...data, [field]: value });
  };

  const updateBullet = (index: number, value: string) => {
    const next = [...bullets];
    next[index] = value;
    update("bullets", next);
  };

  const removeBullet = (index: number) => {
    update("bullets", bullets.filter((_, i) => i !== index));
  };

  const addBullet = () => {
    update("bullets", [...bullets, ""]);
  };

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
      <div className="p-4 space-y-3">
        <div>
          <label className="text-xs text-slate-500 uppercase tracking-wide">Title</label>
          <input
            className={inputClass}
            placeholder="Listing title"
            value={data?.title ?? ""}
            onChange={(e) => update("title", e.target.value)}
          />
        </div>

        <div className="grid grid-cols-2 gap-3">
          <div>
            <label className="text-xs text-slate-500 uppercase tracking-wide">Price</label>
            <input
              className={inputClass}
              placeholder="$0.00"
              value={data?.price ?? ""}
              onChange={(e) => update("price", e.target.value)}
            />
          </div>
          <div>
            <label className="text-xs text-slate-500 uppercase tracking-wide">Rating</label>
            <div className="flex items-center gap-1 mt-1 h-[38px]">
              {[1, 2, 3, 4, 5].map((star) => (
                <button
                  key={star}
                  onClick={() => update("rating", star)}
                  className={`text-lg transition-colors ${
                    star <= (data?.rating ?? 0) ? "text-amber-400" : "text-slate-600 hover:text-slate-400"
                  }`}
                >
                  ★
                </button>
              ))}
              {data?.rating && (
                <span className="text-xs text-slate-500 ml-1">{data.rating}/5</span>
              )}
            </div>
          </div>
        </div>

        <div>
          <div className="flex items-center justify-between">
            <label className="text-xs text-slate-500 uppercase tracking-wide">Bullet Points</label>
            <button
              onClick={addBullet}
              className="flex items-center gap-1 text-xs text-emerald-400 hover:text-emerald-300 transition-colors"
            >
              <PlusCircle size={12} />
              Add
            </button>
          </div>
          <div className="mt-1 space-y-1.5">
            {bullets.length === 0 && (
              <p className="text-xs text-slate-600 py-2">No bullet points yet.</p>
            )}
            {bullets.map((b, i) => (
              <div key={i} className="flex items-center gap-1.5">
                <span className="text-emerald-400 text-xs shrink-0">•</span>
                <input
                  className="flex-1 px-2 py-1.5 bg-slate-800/60 rounded-lg text-sm text-slate-200 border border-slate-700/50 focus:border-emerald-500/50 focus:ring-1 focus:ring-emerald-500/20 outline-none placeholder:text-slate-600 transition-colors"
                  value={b}
                  placeholder="Bullet point"
                  onChange={(e) => updateBullet(i, e.target.value)}
                />
                <button
                  onClick={() => removeBullet(i)}
                  className="p-1 text-slate-600 hover:text-red-400 transition-colors shrink-0"
                >
                  <X size={14} />
                </button>
              </div>
            ))}
          </div>
        </div>

        <div>
          <label className="text-xs text-slate-500 uppercase tracking-wide">Description</label>
          <textarea
            className={textareaClass}
            placeholder="Product description"
            value={data?.description ?? ""}
            onChange={(e) => update("description", e.target.value)}
          />
        </div>
      </div>
    </div>
  );
}

// ─── Inner component (inside CopilotKit provider) ──────────────────────

function formatModelShort(model?: string): string {
  if (!model) return "Sonnet";
  if (model.includes("opus")) return "Opus";
  if (model.includes("haiku")) return "Haiku";
  return "Sonnet";
}

function formatCost(usd: number): string {
  if (usd < 0.01) return "$0.00";
  return `$${usd.toFixed(4)}`;
}

function AgentChatInner() {
  const { activeSessionId, sessions, interruptSession, sessionUsage } =
    useSessionStore();

  const activeSession = sessions.find((s) => s.id === activeSessionId);
  const usage = activeSessionId ? sessionUsage[activeSessionId] : undefined;
  const statusColor =
    activeSession?.status === "Active"
      ? "text-emerald-400"
      : activeSession?.status === "Connected" || activeSession?.status === "Idle"
        ? "text-blue-400"
        : activeSession?.status === "Starting"
          ? "text-yellow-400"
          : "text-slate-500";

  // Workspace state — updated by Gen-UI tools and user edits
  const [productData, setProductData] = useState<ProductData | null>(null);
  const [productStatus, setProductStatus] = useState("idle");
  const [listingData, setListingData] = useState<ListingData | null>(null);
  const [listingStatus, setListingStatus] = useState("idle");

  // ─── Share workspace state with the agent ─────────────────────────
  // This lets the agent read current field values, including user edits.

  useCopilotReadable({
    description: "Current product input form data in the workspace. The user can edit these fields directly.",
    value: productData,
  });

  useCopilotReadable({
    description: "Current listing output data in the workspace. The user can edit these fields directly.",
    value: listingData,
  });

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
          <div className="ml-auto flex items-center gap-3 text-xs">
            <PermissionModeSelector />
            <span className="px-1.5 py-0.5 rounded bg-slate-800 text-slate-400 font-mono">
              {formatModelShort(activeSession?.model)}
            </span>
            {usage && (usage.input_tokens > 0 || usage.output_tokens > 0) && (
              <span
                className="flex items-center gap-1 text-slate-500"
                title={`In: ${usage.input_tokens.toLocaleString()} | Out: ${usage.output_tokens.toLocaleString()} | Cache write: ${usage.cache_creation_input_tokens.toLocaleString()} | Cache read: ${usage.cache_read_input_tokens.toLocaleString()}`}
              >
                <DollarSign size={10} />
                {formatCost(
                  ((usage.input_tokens * 3 +
                    usage.output_tokens * 15 +
                    usage.cache_creation_input_tokens * 3.75 +
                    usage.cache_read_input_tokens * 0.3) /
                    1_000_000)
                )}
              </span>
            )}
            <div className="flex items-center gap-2">
              <Circle size={8} className={`fill-current ${statusColor}`} />
              <span className="text-slate-400">{activeSession?.status || "Unknown"}</span>
              <span className="text-slate-600">{activeSessionId!.slice(0, 8)}...</span>
            </div>
          </div>
        </header>

        <div className="flex-1 overflow-y-auto p-4 space-y-4">
          <ProductInputPanel data={productData} status={productStatus} onChange={setProductData} />
          <ListingOutputPanel data={listingData} status={listingStatus} onChange={setListingData} />
        </div>
      </div>

      {/* Right: Chat panel */}
      <div className="w-[380px] border-l border-slate-800 flex flex-col shrink-0 copilotkit-chat-container">
        <div className="flex items-center gap-2 px-4 py-3 border-b border-slate-800 bg-slate-900 shrink-0">
          <MessageSquare size={16} className="text-slate-400" />
          <span className="text-xs font-medium text-slate-300">Chat</span>
        </div>
        <div className="flex-1 overflow-hidden flex flex-col">
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
          <ToolApprovalBanner />
        </div>
      </div>
    </div>
  );
}

// ─── Model options ────────────────────────────────────────────────────

const MODEL_OPTIONS = [
  {
    id: "claude-sonnet-4-5-20250929",
    label: "Sonnet 4.5",
    description: "Fast and capable",
    color: "text-blue-400",
    bgActive: "bg-blue-500/10 border-blue-500/40",
  },
  {
    id: "claude-opus-4-5-20250918",
    label: "Opus 4.5",
    description: "Most powerful",
    color: "text-purple-400",
    bgActive: "bg-purple-500/10 border-purple-500/40",
  },
  {
    id: "claude-haiku-4-5-20251001",
    label: "Haiku 4.5",
    description: "Fastest, lowest cost",
    color: "text-emerald-400",
    bgActive: "bg-emerald-500/10 border-emerald-500/40",
  },
] as const;

// ─── Outer wrapper ─────────────────────────────────────────────────────

export default function AgentChat() {
  const { activeSessionId, spawnSession } = useSessionStore();
  const runtimeUrl = useAppStore((s) => s.runtimeUrl);
  const [selectedModel, setSelectedModel] = useState<string>(MODEL_OPTIONS[0].id);
  const [spawning, setSpawning] = useState(false);

  const handleSpawnSession = useCallback(async () => {
    setSpawning(true);
    try {
      await spawnSession(".", undefined, selectedModel);
    } catch (e: any) {
      console.error("Failed to spawn session:", e);
    }
    setSpawning(false);
  }, [spawnSession, selectedModel]);

  if (!runtimeUrl || !activeSessionId) {
    return (
      <div className="h-full flex flex-col">
        <header className="flex items-center gap-2 px-4 py-3 border-b border-slate-800 bg-slate-900">
          <Sparkles size={18} className="text-katara-400" />
          <h1 className="text-sm font-semibold">Agent Workspace</h1>
        </header>
        <div className="flex-1 flex items-center justify-center">
          <div className="text-center max-w-md">
            <Sparkles size={48} className="mx-auto mb-4 text-slate-700" />
            <p className="text-lg font-medium text-slate-300">Start a Claude Code Session</p>
            <p className="text-sm mt-1 text-slate-500">
              Spawn a new session to start chatting with Claude Code.
              {!runtimeUrl && " Waiting for backend to start..."}
            </p>

            {/* Model selector */}
            <div className="mt-6 flex gap-2 justify-center">
              {MODEL_OPTIONS.map((model) => {
                const isSelected = selectedModel === model.id;
                return (
                  <button
                    key={model.id}
                    onClick={() => setSelectedModel(model.id)}
                    className={`px-3 py-2 rounded-lg border text-left transition-all ${
                      isSelected
                        ? `${model.bgActive} border`
                        : "bg-slate-800/50 border-slate-700/50 hover:border-slate-600"
                    }`}
                  >
                    <div className={`text-sm font-medium ${isSelected ? model.color : "text-slate-300"}`}>
                      {model.label}
                    </div>
                    <div className="text-xs text-slate-500 mt-0.5">
                      {model.description}
                    </div>
                  </button>
                );
              })}
            </div>

            <button
              onClick={handleSpawnSession}
              disabled={!runtimeUrl || spawning}
              className="mt-4 px-4 py-2 bg-katara-600 hover:bg-katara-500 disabled:bg-slate-700 disabled:text-slate-500 text-white rounded-lg flex items-center gap-2 mx-auto transition-colors"
            >
              <Plus size={16} />
              {spawning ? "Starting..." : "New Session"}
            </button>
          </div>
        </div>
      </div>
    );
  }

  return <AgentChatInner />;
}

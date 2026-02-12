# CopilotKit Gen-UI Integration Plan

## Goal
Add CopilotKit Gen-UI to the existing AgentChat tab so Claude's tool calls render as rich, interactive React components inline in the chat. The flagship use case: an Amazon listing agent with structured input forms and beautiful output displays.

## Architecture Overview

```
                    CopilotKit Provider (runtimeUrl -> our Axum server)
                           │
          ┌────────────────┼────────────────┐
          │                │                │
   CopilotChat UI    useCopilotAction   Catch-all "*" renderer
   (replaces our     (domain-specific    (renders ANY tool call
    custom chat)      Gen-UI actions)     Claude makes as UI)
          │                │                │
          └────────────────┼────────────────┘
                           │
              Axum AG-UI Backend (SSE)
                           │
              Claude CLI via WebSocket
```

## Key Insight: Two Layers of Tool Calls

1. **Claude CLI's internal tools** (Read, Edit, Bash, Glob, etc.) — These are the tools Claude Code uses to operate. Our backend already translates these into AG-UI `ToolCallStart/Args/End` events. The catch-all `name: "*"` renderer can display these as collapsible tool-activity UI.

2. **Custom Gen-UI actions** (domain-specific) — These are `useCopilotAction` hooks registered on the frontend that Claude can invoke. For the Amazon listing agent: `showProductForm`, `displayListing`, `showImageGallery`, etc. CopilotKit sends these to the backend in the `tools` field, and our backend must forward them to Claude's system prompt so Claude knows they're available.

## Implementation Steps

### Step 1: Re-add CopilotKit Provider to App.tsx
- Wrap the app (or just AgentChat) with `<CopilotKit runtimeUrl={runtimeUrl} agent="default">`
- Import CopilotKit styles
- The provider will call our `/api/copilotkit/info` endpoint to discover the "default" agent

### Step 2: Replace custom chat UI with CopilotChat
- Replace our hand-rolled chat messages/input with `<CopilotChat>` from `@copilotkit/react-ui`
- This gives us: markdown rendering, streaming text, action execution rendering, stop button, etc.
- Keep our session management (spawn/kill/interrupt) alongside CopilotChat
- CopilotChat will send messages via the CopilotKit runtime (POST to our Axum endpoint) which then forwards to Claude CLI

### Step 3: Fix the Axum AG-UI backend to handle CopilotKit v1.51's protocol
- CopilotKit v1.51 uses **both** GraphQL (legacy) and AG-UI (new). It tries AG-UI first.
- Our SSE endpoint at `/api/copilotkit` needs to correctly handle the AG-UI POST format
- The `tools` array in `RunAgentInput` contains frontend-registered actions — we need to forward these tool definitions to Claude so it knows it can call them
- Session management: CopilotKit uses `threadId` — we map this to our session IDs

### Step 4: Add catch-all Gen-UI renderer for Claude's tools
- Register `useCopilotAction({ name: "*", render: ... })` to render ANY tool call
- This renders Claude's Read/Edit/Bash/Glob etc. as collapsible cards in the chat
- Shows tool name, input args, and status (in-progress/complete)

### Step 5: Add domain-specific Gen-UI actions
- Create reusable Gen-UI components in `src/genui/` directory
- Start with an example: Amazon listing agent actions
  - `renderProductInput`: File upload form, image preview
  - `renderListing`: Rich listing display with title, bullets, description, images
  - `renderComparisonTable`: Side-by-side product comparison
- These use `useCopilotAction` with `render` functions
- Claude invokes them by name when it decides the UI is appropriate

### Step 6: Wire tool definitions to Claude's context
- When CopilotKit sends `tools` in RunAgentInput, our backend extracts the action schemas
- Forward these as part of Claude's system prompt or as tool definitions
- When Claude calls one of these tools, our bridge emits ToolCallStart/Args/End
- CopilotKit matches the tool name to the registered action and renders the component

## Files to Create/Modify

### Modified:
- `src/App.tsx` — Re-add CopilotKit provider with runtimeUrl
- `src/tabs/AgentChat.tsx` — Replace custom chat with CopilotChat + session controls + Gen-UI hooks
- `src-tauri/src/agui/server.rs` — Fix session routing, forward tools to Claude
- `src-tauri/src/agui/bridge.rs` — Improve tool call event translation
- `src-tauri/src/agui/events.rs` — Add ToolCallResult event type

### Created:
- `src/genui/CatchAllToolRenderer.tsx` — Catch-all renderer for Claude's internal tools
- `src/genui/GenUIActions.tsx` — Domain-specific Gen-UI action hooks (Amazon listing example)

## Risk: CopilotKit v1.51 GraphQL vs AG-UI
CopilotKit v1.51 internally uses `@copilotkitnext` which supports AG-UI via `AbstractAgent`. When `runtimeUrl` is set, it fetches `/info`, creates a `ProxiedCopilotRuntimeAgent`, and that agent uses AG-UI SSE transport. Our existing SSE endpoint should work — but we may need to handle the exact AG-UI event format CopilotKit expects (field names, event types, etc.).

## Iteration Plan
1. Get CopilotChat rendering with basic text streaming (Steps 1-3)
2. Add catch-all tool renderer so Claude's tools show as UI (Step 4)
3. Add first domain-specific Gen-UI action as proof-of-concept (Step 5)
4. Refine and add more Gen-UI components (Step 6)

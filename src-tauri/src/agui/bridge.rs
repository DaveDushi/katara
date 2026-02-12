use crate::agui::events::AguiEvent;
use crate::websocket::protocol::{ClaudeMessage, ContentBlock};

/// Tracks state across streaming events within a single run.
/// Created once per AG-UI request in the handler loop.
#[derive(Debug, Default)]
pub struct BridgeState {
    /// Maps content_block index to block type ("text" or "tool_use")
    block_types: std::collections::HashMap<u64, String>,
    /// Maps content_block index to tool_use ID (for tool blocks)
    block_tool_ids: std::collections::HashMap<u64, String>,
    /// Whether we've received any streaming text events
    has_streamed_text: bool,
    /// Tool IDs that were already streamed
    streamed_tool_ids: std::collections::HashSet<String>,
}

impl BridgeState {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Translates a Claude CLI NDJSON message into zero or more AG-UI events.
///
/// This is the central translation layer between Claude Code's protocol
/// and the AG-UI protocol that CopilotKit's frontend understands.
///
/// Message lifecycle from Claude CLI:
///   1. stream_event(content_block_start)  — new text or tool_use block
///   2. stream_event(content_block_delta)  — token-by-token text or partial JSON
///   3. stream_event(content_block_stop)   — block finished
///   4. assistant                          — final complete message (all blocks)
///   5. result                             — turn complete
///
/// We emit AG-UI events from streaming events for real-time display.
/// The final `assistant` message is used only for tool_use blocks
/// that weren't already streamed.
pub fn translate_claude_message(
    msg: &ClaudeMessage,
    thread_id: &str,
    run_id: &str,
    bridge: &mut BridgeState,
) -> Vec<AguiEvent> {
    let mut events = Vec::new();

    match msg {
        ClaudeMessage::System(sys) if sys.subtype == "init" => {
            events.push(AguiEvent::StateSnapshot {
                snapshot: serde_json::json!({
                    "model": sys.model,
                    "tools": sys.tools,
                    "sessionId": sys.session_id,
                    "cwd": sys.cwd,
                }),
            });
        }

        ClaudeMessage::StreamEvent(stream) => {
            match stream.event.event_type.as_str() {
                "content_block_start" => {
                    let block_type = stream
                        .event
                        .extra
                        .get("content_block")
                        .and_then(|cb| cb.get("type"))
                        .and_then(|t| t.as_str())
                        .unwrap_or("text");

                    let index = stream.event.index.unwrap_or(0);
                    bridge.block_types.insert(index, block_type.to_string());

                    if block_type == "text" {
                        let msg_id = format!("{}-msg-{}", run_id, index);
                        events.push(AguiEvent::TextMessageStart {
                            message_id: msg_id,
                            role: "assistant".into(),
                        });
                        bridge.has_streamed_text = true;
                    } else if block_type == "tool_use" {
                        let cb = stream.event.extra.get("content_block");
                        let tool_id = cb
                            .and_then(|c| c.get("id"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string();
                        let tool_name = cb
                            .and_then(|c| c.get("name"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string();

                        bridge.block_tool_ids.insert(index, tool_id.clone());
                        bridge.streamed_tool_ids.insert(tool_id.clone());

                        events.push(AguiEvent::ToolCallStart {
                            tool_call_id: tool_id,
                            tool_call_name: tool_name,
                            parent_message_id: None,
                        });
                    }
                }

                "content_block_delta" => {
                    let index = stream.event.index.unwrap_or(0);

                    if let Some(ref delta) = stream.event.delta {
                        if delta.delta_type == "text_delta" {
                            if let Some(ref text) = delta.text {
                                let msg_id = format!("{}-msg-{}", run_id, index);
                                events.push(AguiEvent::TextMessageContent {
                                    message_id: msg_id,
                                    delta: text.clone(),
                                });
                            }
                        } else if delta.delta_type == "input_json_delta" {
                            if let Some(ref partial) = delta.partial_json {
                                let tool_id = bridge
                                    .block_tool_ids
                                    .get(&index)
                                    .cloned()
                                    .unwrap_or_else(|| format!("{}-tool-{}", run_id, index));
                                events.push(AguiEvent::ToolCallArgs {
                                    tool_call_id: tool_id,
                                    delta: partial.clone(),
                                });
                            }
                        }
                    }
                }

                "content_block_stop" => {
                    let index = stream.event.index.unwrap_or(0);
                    let block_type = bridge.block_types.get(&index).map(|s| s.as_str());

                    match block_type {
                        Some("text") => {
                            let msg_id = format!("{}-msg-{}", run_id, index);
                            events.push(AguiEvent::TextMessageEnd {
                                message_id: msg_id,
                            });
                        }
                        Some("tool_use") => {
                            let tool_id = bridge
                                .block_tool_ids
                                .get(&index)
                                .cloned()
                                .unwrap_or_else(|| format!("{}-tool-{}", run_id, index));
                            events.push(AguiEvent::ToolCallEnd {
                                tool_call_id: tool_id,
                            });
                        }
                        _ => {
                            // Unknown block type, emit text end as safe fallback
                            let msg_id = format!("{}-msg-{}", run_id, index);
                            events.push(AguiEvent::TextMessageEnd {
                                message_id: msg_id,
                            });
                        }
                    }
                }

                _ => {
                    // message_start, message_stop, message_delta, etc.
                }
            }
        }

        ClaudeMessage::Assistant(assistant) => {
            // Final assistant message: skip blocks that were already streamed.
            for block in &assistant.message.content {
                match block {
                    ContentBlock::Text { text } => {
                        if !bridge.has_streamed_text {
                            // No streaming happened — emit full text as single message
                            let msg_id = assistant.message.id.clone();
                            events.push(AguiEvent::TextMessageStart {
                                message_id: msg_id.clone(),
                                role: "assistant".into(),
                            });
                            events.push(AguiEvent::TextMessageContent {
                                message_id: msg_id.clone(),
                                delta: text.clone(),
                            });
                            events.push(AguiEvent::TextMessageEnd {
                                message_id: msg_id,
                            });
                        }
                    }
                    ContentBlock::ToolUse { id, name, input } => {
                        if !bridge.streamed_tool_ids.contains(id) {
                            // Tool wasn't streamed — emit complete tool call
                            events.push(AguiEvent::ToolCallStart {
                                tool_call_id: id.clone(),
                                tool_call_name: name.clone(),
                                parent_message_id: Some(assistant.message.id.clone()),
                            });
                            events.push(AguiEvent::ToolCallArgs {
                                tool_call_id: id.clone(),
                                delta: serde_json::to_string(input).unwrap_or_default(),
                            });
                            events.push(AguiEvent::ToolCallEnd {
                                tool_call_id: id.clone(),
                            });
                        }
                    }
                    ContentBlock::ToolResult { .. } => {}
                }
            }
        }

        ClaudeMessage::ControlRequest(ctrl) => {
            if ctrl.request.subtype == "can_use_tool" {
                events.push(AguiEvent::Custom {
                    name: "tool_approval_request".into(),
                    value: serde_json::json!({
                        "requestId": ctrl.request.request_id,
                        "toolName": ctrl.request.tool_name,
                        "toolInput": ctrl.request.input,
                        "toolUseId": ctrl.request.tool_use_id,
                    }),
                });
            }
        }

        ClaudeMessage::Result(_result) => {
            events.push(AguiEvent::RunFinished {
                thread_id: thread_id.to_string(),
                run_id: run_id.to_string(),
            });
        }

        _ => {}
    }

    events
}

import { useEffect, useRef, useState } from "react";
import { useAuth } from "../context/AuthContext";
import { ChatMessage, ModelOption } from "../types/api";
import MessageBubble from "./MessageBubble";

export default function PromptTab() {
  const { creds } = useAuth();
  const [models, setModels] = useState<ModelOption[]>([]);
  const [selectedModel, setSelectedModel] = useState<string>("");
  const [temperature, setTemperature] = useState<number>(1.0);
  const [maxTokens, setMaxTokens] = useState<number>(1024);
  const [input, setInput] = useState<string>("");
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [streaming, setStreaming] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);
  const abortRef = useRef<AbortController | null>(null);
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!creds) return;
    fetch("/api/models", {
      headers: { Authorization: `Basic ${creds}` },
    })
      .then((r) => r.json())
      .then((data: ModelOption[]) => {
        setModels(data);
        if (data.length > 0) setSelectedModel(data[0].model);
      })
      .catch(() => setError("Failed to load models"));
  }, [creds]);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  async function sendMessage() {
    const text = input.trim();
    if (!text || streaming) return;
    setError(null);

    const userMsg: ChatMessage = { role: "user", content: text };
    const nextHistory = [...messages, userMsg];
    setMessages(nextHistory);
    setInput("");
    setStreaming(true);

    const controller = new AbortController();
    abortRef.current = controller;

    const placeholderIdx = nextHistory.length;
    setMessages([...nextHistory, { role: "assistant", content: "" }]);

    try {
      const res = await fetch("/v1/chat/completions", {
        method: "POST",
        signal: controller.signal,
        headers: {
          "Content-Type": "application/json",
          Authorization: `Basic ${creds}`,
          "x-provider": "anthropic",
        },
        body: JSON.stringify({
          model: selectedModel,
          messages: nextHistory
            .filter((m) => m.content.trim().length > 0)
            .map((m) => ({ role: m.role, content: m.content })),
          stream: true,
          temperature,
          max_tokens: maxTokens,
        }),
      });

      if (res.status === 401) {
        throw new Error("Unauthorized — check credentials");
      }
      if (!res.ok) {
        throw new Error(`Upstream error: HTTP ${res.status}`);
      }

      const reader = res.body!.getReader();
      const decoder = new TextDecoder();
      let sseBuffer = "";
      let accumulated = "";

      while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        sseBuffer += decoder.decode(value, { stream: true });

        const lines = sseBuffer.split("\n");
        sseBuffer = lines.pop() ?? "";

        for (const line of lines) {
          if (!line.startsWith("data: ")) continue;
          const json = line.slice(6).trim();
          if (json === "[DONE]") continue;
          try {
            const chunk = JSON.parse(json);
            // Anthropic SSE format: content_block_delta with text_delta
            if (
              chunk?.type === "content_block_delta" &&
              chunk?.delta?.type === "text_delta" &&
              chunk?.delta?.text
            ) {
              accumulated += chunk.delta.text;
              setMessages((prev) => {
                const updated = [...prev];
                updated[placeholderIdx] = { role: "assistant", content: accumulated };
                return updated;
              });
            }
          } catch {
            // malformed chunk — skip
          }
        }
      }
    } catch (err: unknown) {
      if (err instanceof Error && err.name === "AbortError") {
        setMessages((prev) => {
          const updated = [...prev];
          if (updated[placeholderIdx]) {
            updated[placeholderIdx] = { ...updated[placeholderIdx], cancelled: true };
          }
          return updated;
        });
      } else {
        setError(err instanceof Error ? err.message : "Unknown error");
        setMessages((prev) => prev.slice(0, placeholderIdx));
      }
    } finally {
      abortRef.current = null;
      setStreaming(false);
    }
  }

  function cancelStream() {
    abortRef.current?.abort();
  }

  function handleKeyDown(e: React.KeyboardEvent<HTMLTextAreaElement>) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  }

  return (
    <div className="flex flex-col h-full max-h-[calc(100vh-120px)]">
      {/* Controls */}
      <div className="flex flex-wrap gap-4 items-center p-4 border-b border-gray-200 bg-white">
        <div className="flex items-center gap-2">
          <label className="text-sm font-medium text-gray-700">Model</label>
          <select
            value={selectedModel}
            onChange={(e) => setSelectedModel(e.target.value)}
            className="text-sm border border-gray-300 rounded px-2 py-1"
            disabled={streaming}
          >
            {models.map((m) => (
              <option key={m.model} value={m.model}>
                {m.display_name}
              </option>
            ))}
          </select>
        </div>
        <div className="flex items-center gap-2">
          <label className="text-sm font-medium text-gray-700">Temp</label>
          <input
            type="number"
            min={0}
            max={2}
            step={0.1}
            value={temperature}
            onChange={(e) => setTemperature(parseFloat(e.target.value))}
            className="text-sm border border-gray-300 rounded px-2 py-1 w-20"
            disabled={streaming}
          />
        </div>
        <div className="flex items-center gap-2">
          <label className="text-sm font-medium text-gray-700">Max tokens</label>
          <input
            type="number"
            min={1}
            max={8192}
            step={256}
            value={maxTokens}
            onChange={(e) => setMaxTokens(parseInt(e.target.value, 10))}
            className="text-sm border border-gray-300 rounded px-2 py-1 w-24"
            disabled={streaming}
          />
        </div>
        <button
          onClick={() => setMessages([])}
          className="ml-auto text-sm text-gray-500 hover:text-red-600"
          disabled={streaming}
        >
          Clear
        </button>
      </div>

      {/* Message list */}
      <div className="flex-1 overflow-y-auto p-4">
        {messages.length === 0 && (
          <p className="text-center text-gray-400 text-sm mt-8">
            Send a message to start a conversation
          </p>
        )}
        {messages.map((msg, i) => (
          <MessageBubble key={i} message={msg} />
        ))}
        <div ref={bottomRef} />
      </div>

      {/* Error */}
      {error && (
        <div className="px-4 py-2 bg-red-50 border-t border-red-200 text-sm text-red-700">
          {error}
        </div>
      )}

      {/* Input area */}
      <div className="border-t border-gray-200 p-4 bg-white">
        <div className="flex gap-2">
          <textarea
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Type a message… (Enter to send, Shift+Enter for newline)"
            rows={3}
            className="flex-1 resize-none border border-gray-300 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
            disabled={streaming}
          />
          {streaming ? (
            <button
              onClick={cancelStream}
              className="self-end px-4 py-2 bg-red-500 hover:bg-red-600 text-white text-sm font-medium rounded-lg"
            >
              Stop
            </button>
          ) : (
            <button
              onClick={sendMessage}
              disabled={!input.trim()}
              className="self-end px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:opacity-50 text-white text-sm font-medium rounded-lg"
            >
              Send
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

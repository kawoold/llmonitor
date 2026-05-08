export type TimeWindow = "1h" | "24h" | "7d" | "30d";

export interface ProviderSummary {
  provider: string;
  model: string;
  request_count: number;
  prompt_tokens: number;
  completion_tokens: number;
  total_tokens: number;
}

export interface StatsSummary {
  window: string;
  total_requests: number;
  total_tokens: number;
  prompt_tokens: number;
  completion_tokens: number;
  by_provider: ProviderSummary[];
}

export interface UsageDataPoint {
  bucket: string;
  total_tokens: number;
  prompt_tokens: number;
  completion_tokens: number;
}

export interface CacheStats {
  total_requests: number;
  cache_read_tokens: number;
  cache_creation_tokens: number;
  cache_hit_rate: number;
}

export interface ConfigResponse {
  anthropic_api_key: string;
}

export interface ModelOption {
  provider: string;
  model: string;
  display_name: string;
}

export interface ChatMessage {
  role: "user" | "assistant";
  content: string;
  cancelled?: boolean;
}

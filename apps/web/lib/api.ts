/**
 * Typed client for the Synapse2 REST API.
 *
 * All actions are dispatched via POST /v1/synapse2 with:
 *   { "action": "<action>", "params": { ... } }
 *
 * The base URL is relative (empty string) so the same binary serves
 * both the API and the web UI without CORS configuration.
 */

import { endpoint, WEB_APP_CONFIG } from "@/lib/template";

export interface ApiResponse<T = unknown> {
  data?: T;
  error?: string;
  status?: number;
}

const BEARER_TOKEN_KEY = "synapse2.bearer-token";

export function getBearerToken(): string | null {
  if (typeof window === "undefined") return null;
  return window.sessionStorage.getItem(BEARER_TOKEN_KEY);
}

export function setBearerToken(token: string): void {
  if (typeof window === "undefined") return;
  const normalized = token.trim();
  if (normalized) window.sessionStorage.setItem(BEARER_TOKEN_KEY, normalized);
  else window.sessionStorage.removeItem(BEARER_TOKEN_KEY);
}

export function clearBearerToken(): void {
  if (typeof window !== "undefined") window.sessionStorage.removeItem(BEARER_TOKEN_KEY);
}

export interface StatusResult {
  status: string;
  note?: string;
  server?: string;
  version?: string;
  transport?: string;
}

export interface HealthResult {
  status: string;
}

/** Shared fetch helper — handles JSON parsing and error normalisation. */
export async function apiFetch<T>(url: string, options?: RequestInit): Promise<ApiResponse<T>> {
  try {
    const res = await fetch(url, options);
    const text = await res.text();
    const json = parseJsonBody(text);
    if (!res.ok) {
      const error =
        isRecord(json) && typeof json.error === "string" ? json.error : `HTTP ${res.status}`;
      return { error, status: res.status };
    }
    return { data: json as T };
  } catch (e) {
    return { error: e instanceof Error ? e.message : "Network error" };
  }
}

export function parseJsonBody(text: string): unknown {
  if (!text.trim()) return {};
  try {
    return JSON.parse(text);
  } catch {
    return text;
  }
}

export function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

/** POST /v1/synapse2 — dispatch an action */
export function callAction<T = unknown>(
  action: string,
  params: Record<string, unknown> = {},
): Promise<ApiResponse<T>> {
  const token = getBearerToken();
  return apiFetch<T>(endpoint(WEB_APP_CONFIG.restEndpoint), {
    method: "POST",
    credentials: "same-origin",
    headers: {
      "Content-Type": "application/json",
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
    },
    body: JSON.stringify({ action, params }),
  });
}

/** GET /health */
export function getHealth(signal?: AbortSignal): Promise<ApiResponse<HealthResult>> {
  return apiFetch<HealthResult>(endpoint(WEB_APP_CONFIG.healthEndpoint), { signal });
}

/** GET /status */
export function getStatus(signal?: AbortSignal): Promise<ApiResponse<StatusResult>> {
  return apiFetch<StatusResult>(endpoint(WEB_APP_CONFIG.statusEndpoint), { signal });
}

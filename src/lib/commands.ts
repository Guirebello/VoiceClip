import { invoke } from "@tauri-apps/api/core";

export interface Config {
  model_name: string;
  hotkey: string;
  badge_opacity: number;
  max_recording_duration: number;
  append_mode: boolean;
  microphone: string | null;
  always_on_top: boolean;
  badge_x: number | null;
  badge_y: number | null;
}

export interface StatsSummary {
  total_recordings: number;
  total_seconds: number;
  avg_words: number;
}

export interface SessionRow {
  id: number;
  started_at: number;
  duration_secs: number;
  word_count: number;
  transcription: string;
  latency_ms: number;
  error: string | null;
}

export type BadgeState = "idle" | "recording" | "processing" | "success" | "error";

export function toggleRecording(): Promise<void> {
  return invoke("toggle_recording");
}

export function getConfig(): Promise<Config> {
  return invoke("get_config");
}

export function saveConfig(config: Config): Promise<void> {
  return invoke("save_config", { newConfig: config });
}

export function listInputDevices(): Promise<string[]> {
  return invoke("list_input_devices");
}

export function getStatsSummary(): Promise<StatsSummary> {
  return invoke("get_stats_summary");
}

export function getRecentSessions(limit: number): Promise<SessionRow[]> {
  return invoke("get_recent_sessions", { limit });
}

export function openSettingsWindow(): Promise<void> {
  return invoke("open_settings_window");
}

export function openStatsWindow(): Promise<void> {
  return invoke("open_stats_window");
}

export function saveBadgePosition(x: number, y: number): Promise<void> {
  return invoke("save_badge_position", { x, y });
}

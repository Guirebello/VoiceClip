import { useState, useEffect } from "react";
import { getConfig, saveConfig, listInputDevices, type Config } from "../lib/commands";

export default function SettingsWindow() {
  const [config, setConfig] = useState<Config | null>(null);
  const [devices, setDevices] = useState<string[]>([]);
  const [status, setStatus] = useState("");

  useEffect(() => {
    getConfig().then(setConfig);
    listInputDevices().then(setDevices);
  }, []);

  if (!config) return <div style={styles.container}><p>Loading...</p></div>;

  const handleSave = async () => {
    try {
      await saveConfig(config);
      setStatus("Saved!");
      setTimeout(() => setStatus(""), 2000);
    } catch (e) {
      setStatus(`Error: ${e}`);
    }
  };

  const handleAlwaysOnTopChange = async (checked: boolean) => {
    setConfig({ ...config, always_on_top: checked });
    // Apply immediately to the badge window by saving config
    // The badge window's always_on_top is set at startup; for runtime changes,
    // we'll need to communicate via the backend on next restart or use window API
    try {
      // Try to set it on the badge window directly
      const { WebviewWindow } = await import("@tauri-apps/api/webviewWindow");
      const badge = await WebviewWindow.getByLabel("badge");
      if (badge) {
        await badge.setAlwaysOnTop(checked);
      }
    } catch {
      // Will apply on restart
    }
  };

  return (
    <div style={styles.container}>
      <h2 style={styles.title}>Settings</h2>

      <Section label="Global Hotkey">
        <input
          type="text"
          value={config.hotkey}
          onChange={(e) => setConfig({ ...config, hotkey: e.target.value })}
          style={styles.input}
        />
        <p style={styles.help}>Examples: Super+Alt+V, Ctrl+Shift+R, F9. Use "None" to disable.</p>
        <p style={styles.help}>Changes take effect on restart.</p>
      </Section>

      <Section label="Microphone">
        <select
          value={config.microphone ?? ""}
          onChange={(e) => setConfig({ ...config, microphone: e.target.value || null })}
          style={styles.input}
        >
          <option value="">System Default</option>
          {devices.map((d) => (
            <option key={d} value={d}>{d}</option>
          ))}
        </select>
      </Section>

      <Section label="Window">
        <label style={styles.checkboxLabel}>
          <input
            type="checkbox"
            checked={config.always_on_top}
            onChange={(e) => handleAlwaysOnTopChange(e.target.checked)}
          />
          Always on top
        </label>
      </Section>

      <Section label="General">
        <label style={styles.checkboxLabel}>
          <input
            type="checkbox"
            checked={config.append_mode}
            onChange={(e) => setConfig({ ...config, append_mode: e.target.checked })}
          />
          Append mode (add to existing clipboard)
        </label>
        <div style={{ marginTop: 10 }}>
          <label style={styles.label}>Badge Opacity: {config.badge_opacity.toFixed(1)}</label>
          <input
            type="range"
            min={0.1}
            max={1.0}
            step={0.1}
            value={config.badge_opacity}
            onChange={(e) => setConfig({ ...config, badge_opacity: parseFloat(e.target.value) })}
            style={{ width: "100%" }}
          />
        </div>
      </Section>

      <div style={{ display: "flex", alignItems: "center", gap: 12, marginTop: 16 }}>
        <button onClick={handleSave} style={styles.button}>Save</button>
        {status && <span style={{ color: status.startsWith("Error") ? "#dc143c" : "#50cd32", fontSize: 13 }}>{status}</span>}
      </div>
    </div>
  );
}

function Section({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div style={styles.section}>
      <h3 style={styles.sectionLabel}>{label}</h3>
      {children}
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    padding: 20,
    fontFamily: "system-ui, sans-serif",
    color: "#eee",
    background: "#1e1e1e",
    minHeight: "100vh",
    boxSizing: "border-box",
  },
  title: {
    margin: "0 0 16px 0",
    fontSize: 18,
    fontWeight: 600,
  },
  section: {
    marginBottom: 16,
  },
  sectionLabel: {
    fontSize: 13,
    fontWeight: 600,
    color: "#aaa",
    textTransform: "uppercase" as const,
    letterSpacing: 0.5,
    marginBottom: 6,
    marginTop: 0,
  },
  input: {
    width: "100%",
    padding: "8px 10px",
    fontSize: 14,
    background: "#2a2a2a",
    border: "1px solid #444",
    borderRadius: 4,
    color: "#eee",
    boxSizing: "border-box" as const,
  },
  help: {
    fontSize: 12,
    color: "#888",
    margin: "4px 0 0 0",
  },
  label: {
    fontSize: 13,
    color: "#ccc",
  },
  checkboxLabel: {
    display: "flex",
    alignItems: "center",
    gap: 8,
    fontSize: 14,
    color: "#eee",
    cursor: "pointer",
  },
  button: {
    padding: "8px 24px",
    fontSize: 14,
    background: "#3a7bd5",
    color: "#fff",
    border: "none",
    borderRadius: 4,
    cursor: "pointer",
    fontWeight: 500,
  },
};

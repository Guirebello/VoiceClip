import { useState, useEffect } from "react";
import { getStatsSummary, getRecentSessions, type StatsSummary, type SessionRow } from "../lib/commands";

export default function StatsWindow() {
  const [summary, setSummary] = useState<StatsSummary | null>(null);
  const [sessions, setSessions] = useState<SessionRow[]>([]);

  useEffect(() => {
    getStatsSummary().then(setSummary);
    getRecentSessions(50).then(setSessions);
  }, []);

  return (
    <div style={styles.container}>
      <h2 style={styles.title}>Stats</h2>

      {summary && (
        <div style={styles.summaryRow}>
          <SummaryCard label="Total Recordings" value={String(summary.total_recordings)} />
          <SummaryCard label="Total Minutes" value={(summary.total_seconds / 60).toFixed(1)} />
          <SummaryCard label="Avg Words" value={summary.avg_words.toFixed(1)} />
        </div>
      )}

      <h3 style={styles.sectionLabel}>Recent Sessions</h3>
      <div style={styles.sessionList}>
        {sessions.length === 0 && <p style={styles.empty}>No sessions yet.</p>}
        {sessions.map((s) => (
          <SessionItem key={s.id} session={s} />
        ))}
      </div>
    </div>
  );
}

function SummaryCard({ label, value }: { label: string; value: string }) {
  return (
    <div style={styles.card}>
      <div style={styles.cardValue}>{value}</div>
      <div style={styles.cardLabel}>{label}</div>
    </div>
  );
}

function SessionItem({ session }: { session: SessionRow }) {
  const isError = session.error !== null;
  const date = new Date(session.started_at * 1000);
  const dateStr = date.toLocaleString();
  const preview = isError
    ? session.error!
    : session.transcription.length > 80
      ? session.transcription.slice(0, 80) + "..."
      : session.transcription;

  return (
    <div style={{ ...styles.session, borderLeftColor: isError ? "#dc143c" : "#444" }}>
      <div style={styles.sessionHeader}>
        <span>{dateStr}</span>
        <span>{session.duration_secs}s</span>
        <span>{session.word_count} words</span>
        <span>{session.latency_ms}ms</span>
      </div>
      <div style={{ ...styles.sessionBody, color: isError ? "#dc143c" : "#ccc" }}>
        {preview}
      </div>
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
  summaryRow: {
    display: "flex",
    gap: 12,
    marginBottom: 20,
  },
  card: {
    flex: 1,
    background: "#2a2a2a",
    borderRadius: 8,
    padding: "14px 16px",
    textAlign: "center" as const,
  },
  cardValue: {
    fontSize: 24,
    fontWeight: 700,
    color: "#fff",
  },
  cardLabel: {
    fontSize: 11,
    color: "#888",
    textTransform: "uppercase" as const,
    letterSpacing: 0.5,
    marginTop: 4,
  },
  sectionLabel: {
    fontSize: 13,
    fontWeight: 600,
    color: "#aaa",
    textTransform: "uppercase" as const,
    letterSpacing: 0.5,
    marginBottom: 8,
  },
  sessionList: {
    maxHeight: "calc(100vh - 200px)",
    overflowY: "auto" as const,
  },
  session: {
    padding: "10px 12px",
    background: "#252525",
    borderRadius: 6,
    marginBottom: 8,
    borderLeft: "3px solid #444",
  },
  sessionHeader: {
    display: "flex",
    gap: 12,
    fontSize: 12,
    color: "#888",
    marginBottom: 4,
  },
  sessionBody: {
    fontSize: 13,
    lineHeight: 1.4,
    color: "#ccc",
  },
  empty: {
    color: "#666",
    fontSize: 14,
    textAlign: "center" as const,
    padding: 20,
  },
};

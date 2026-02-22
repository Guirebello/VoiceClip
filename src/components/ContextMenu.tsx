import { useEffect, useRef } from "react";
import { exit } from "@tauri-apps/plugin-process";
import { openSettingsWindow, openStatsWindow } from "../lib/commands";

interface ContextMenuProps {
  x: number;
  y: number;
  onClose: () => void;
}

export default function ContextMenu({ x, y, onClose }: ContextMenuProps) {
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClick = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        onClose();
      }
    };
    document.addEventListener("mousedown", handleClick);
    return () => document.removeEventListener("mousedown", handleClick);
  }, [onClose]);

  return (
    <div
      ref={menuRef}
      style={{
        position: "fixed",
        left: x,
        top: y,
        background: "#2a2a2a",
        border: "1px solid #444",
        borderRadius: 6,
        padding: "4px 0",
        zIndex: 1000,
        minWidth: 120,
        boxShadow: "0 4px 12px rgba(0,0,0,0.5)",
      }}
    >
      <MenuItem label="Stats" onClick={() => { openStatsWindow(); onClose(); }} />
      <MenuItem label="Settings" onClick={() => { openSettingsWindow(); onClose(); }} />
      <div style={{ borderTop: "1px solid #444", margin: "4px 0" }} />
      <MenuItem label="Quit" onClick={() => exit(0)} />
    </div>
  );
}

function MenuItem({ label, onClick }: { label: string; onClick: () => void }) {
  return (
    <div
      onClick={onClick}
      style={{
        padding: "6px 16px",
        color: "#eee",
        fontSize: 13,
        cursor: "pointer",
        fontFamily: "system-ui, sans-serif",
      }}
      onMouseEnter={(e) => { (e.target as HTMLDivElement).style.background = "#3a3a3a"; }}
      onMouseLeave={(e) => { (e.target as HTMLDivElement).style.background = "transparent"; }}
    >
      {label}
    </div>
  );
}

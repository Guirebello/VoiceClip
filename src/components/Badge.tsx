import { useState, useEffect, useRef, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { toggleRecording, saveBadgePosition, type BadgeState } from "../lib/commands";
import ContextMenu from "./ContextMenu";
import "./Badge.css";

const LEVEL_BUFFER_SIZE = 30;

export default function Badge() {
  const [state, setState] = useState<BadgeState>("idle");
  const [menuOpen, setMenuOpen] = useState(false);
  const [menuPos, setMenuPos] = useState({ x: 0, y: 0 });
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const levelsRef = useRef<number[]>(new Array(LEVEL_BUFFER_SIZE).fill(0));
  const animFrameRef = useRef<number>(0);

  // Listen for badge-state events
  useEffect(() => {
    const unlisten = listen<string>("badge-state", (event) => {
      setState(event.payload as BadgeState);
    });
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  // Listen for audio-level events and draw waveform
  useEffect(() => {
    if (state !== "recording") {
      levelsRef.current = new Array(LEVEL_BUFFER_SIZE).fill(0);
      const canvas = canvasRef.current;
      if (canvas) {
        const ctx = canvas.getContext("2d");
        if (ctx) ctx.clearRect(0, 0, 64, 64);
      }
      return;
    }

    const unlisten = listen<number>("audio-level", (event) => {
      levelsRef.current.push(event.payload);
      if (levelsRef.current.length > LEVEL_BUFFER_SIZE) {
        levelsRef.current.shift();
      }
    });

    const draw = () => {
      const canvas = canvasRef.current;
      if (!canvas) return;
      const ctx = canvas.getContext("2d");
      if (!ctx) return;

      ctx.clearRect(0, 0, 64, 64);
      const levels = levelsRef.current;
      const cx = 32;
      const cy = 32;
      const baseRadius = 26;
      const maxExtend = 6;
      const barCount = levels.length;

      ctx.strokeStyle = "rgba(255, 255, 255, 0.6)";
      ctx.lineWidth = 2;
      ctx.lineCap = "round";

      for (let i = 0; i < barCount; i++) {
        const angle = (i / barCount) * Math.PI * 2 - Math.PI / 2;
        const level = Math.min(levels[i] * 5, 1);
        const extend = level * maxExtend;

        const x1 = cx + Math.cos(angle) * baseRadius;
        const y1 = cy + Math.sin(angle) * baseRadius;
        const x2 = cx + Math.cos(angle) * (baseRadius + extend);
        const y2 = cy + Math.sin(angle) * (baseRadius + extend);

        ctx.beginPath();
        ctx.moveTo(x1, y1);
        ctx.lineTo(x2, y2);
        ctx.stroke();
      }

      animFrameRef.current = requestAnimationFrame(draw);
    };

    animFrameRef.current = requestAnimationFrame(draw);

    return () => {
      unlisten.then((fn) => fn());
      cancelAnimationFrame(animFrameRef.current);
    };
  }, [state]);

  const handleClick = useCallback(() => {
    toggleRecording();
  }, []);

  const handleContextMenu = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    setMenuPos({ x: e.clientX, y: e.clientY });
    setMenuOpen(true);
  }, []);

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    if (e.button === 0 && !menuOpen) {
      getCurrentWindow().startDragging();
    }
  }, [menuOpen]);

  // Save badge position on window move
  useEffect(() => {
    let timeout: ReturnType<typeof setTimeout>;
    const unlisten = getCurrentWindow().onMoved((pos) => {
      clearTimeout(timeout);
      timeout = setTimeout(() => {
        saveBadgePosition(pos.payload.x, pos.payload.y);
      }, 500);
    });
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  const icon = state === "recording" ? "\u23F9"
    : state === "processing" ? "\u2026"
    : state === "success" ? "\u2713"
    : state === "error" ? "\u2717"
    : "\uD83C\uDFA4";

  return (
    <div
      className="badge-container"
      onMouseDown={handleMouseDown}
      onClick={handleClick}
      onContextMenu={handleContextMenu}
    >
      <div className={`badge ${state}`}>
        <span className="badge-icon">{icon}</span>
      </div>
      <canvas ref={canvasRef} className="waveform-canvas" width={64} height={64} />
      {menuOpen && (
        <ContextMenu
          x={menuPos.x}
          y={menuPos.y}
          onClose={() => setMenuOpen(false)}
        />
      )}
    </div>
  );
}

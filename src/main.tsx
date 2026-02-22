import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import Badge from "./components/Badge";
import SettingsWindow from "./components/SettingsWindow";
import StatsWindow from "./components/StatsWindow";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <BrowserRouter>
      <Routes>
        <Route path="/badge" element={<Badge />} />
        <Route path="/settings" element={<SettingsWindow />} />
        <Route path="/stats" element={<StatsWindow />} />
        <Route path="/" element={<Badge />} />
      </Routes>
    </BrowserRouter>
  </StrictMode>
);

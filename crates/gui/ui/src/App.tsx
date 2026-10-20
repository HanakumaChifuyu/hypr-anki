import { Navigate, Route, Routes } from "react-router";
import AppLayout from "./components/layout/AppLayout";
import DeckPage from "./pages/DeckPage";
import DeckSettingsPage from "./pages/DeckSettingsPage";
import BrowserPage from "./pages/BrowserPage";
import StatusPage from "./pages/StatusPage";
import SettingPage from "./pages/SettingPage";

export default function App() {
  return (
    <Routes>
      <Route element={<AppLayout />}>
        <Route index element={<Navigate to="/deck" replace />} />
        <Route path="deck" element={<DeckPage />} />
        <Route path="deck/:deckId/settings" element={<DeckSettingsPage />} />
        <Route path="browser" element={<BrowserPage />} />
        <Route path="status" element={<StatusPage />} />
        <Route path="setting" element={<SettingPage />} />
        <Route path="*" element={<Navigate to="/deck" replace />} />
      </Route>
    </Routes>
  );
}

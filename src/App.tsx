import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import SetupPassword from "./features/password/components/SetupPassword";
import Dashboard from "./features/dashboard/components/Dashboard";
import "./App.css";

function App() {
  const [initialized, setInitialized] = useState<boolean | null>(null);
  const [isAuthenticated, setIsAuthenticated] = useState(false);

  const checkInitialized = async () => {
    try {
      const result = await invoke<boolean>("is_initialized");
      setInitialized(result);
    } catch (err) {
      console.error("Failed to check initialized", err);
      setInitialized(false);
    }
  };

  useEffect(() => {
    checkInitialized();
  }, []);

  const handleAuthSuccess = async () => {
    await checkInitialized(); // обновляем состояние на случай, если только что создали пароль
    setIsAuthenticated(true);
  };

  const handleLock = async () => {
    await invoke("lock");
    setIsAuthenticated(false);
    await checkInitialized(); // убеждаемся, что initialized остался true
  };

  if (initialized === null) {
    return (
      <div className="h-screen flex items-center justify-center text-white">
        Loading...
      </div>
    );
  }

  if (isAuthenticated) {
    return (
      <main className="h-screen bg-gray-900">
        <Dashboard onLock={handleLock} />
      </main>
    );
  }

  return (
    <main className="h-screen flex items-center justify-center bg-linear-to-r from-gray-500 to-gray-700">
      {!initialized ? (
        <SetupPassword mode="setup" onSuccess={handleAuthSuccess} />
      ) : (
        <SetupPassword mode="unlock" onSuccess={handleAuthSuccess} />
      )}
    </main>
  );
}

export default App;

import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface DashboardProps {
  onLock: () => void;
}

export default function Dashboard({ onLock }: DashboardProps) {
  const [privateKey, setPrivateKey] = useState("");
  const [hasSavedKey, setHasSavedKey] = useState(false);
  const [error, setError] = useState("");
  const [isChanging, setIsChanging] = useState(false);
  const [newPrivateKey, setNewPrivateKey] = useState("");

  useEffect(() => {
    invoke<string | null>("get_private_key")
      .then((key) => {
        if (key) setHasSavedKey(true);
      })
      .catch(console.error);
  }, []);

  const handleSavePrivateKey = async () => {
    setError("");
    if (!privateKey.trim()) {
      setError("Enter your private key");
      return;
    }
    try {
      await invoke("save_private_key", { privateKeyB58: privateKey });
      setHasSavedKey(true);
      setPrivateKey("");
      alert("Private key saved and encrypted");
    } catch (err: any) {
      setError(err.toString());
    }
  };

  const handleChangePrivateKey = async () => {
    setError("");
    if (!newPrivateKey.trim()) {
      setError("Enter your new private key");
      return;
    }
    try {
      await invoke("change_private_key", { newPrivateKeyB58: newPrivateKey });
      setHasSavedKey(true);
      setNewPrivateKey("");
      setIsChanging(false);
      alert("Private key changed");
    } catch (err: any) {
      setError(err.toString());
    }
  };

  const handleRemovePrivateKey = async () => {
    if (confirm("Are you sure you want to remove the saved privarte key?")) {
      try {
        await invoke("remove_private_key");
        setHasSavedKey(false);
        alert("Private key removed");
      } catch (err: any) {
        setError(err.toString());
      }
    }
  };

  const handleLock = async () => {
    await invoke("lock");
    onLock();
  };

  return (
    <div className="text-white p-4">
      <h1 className="text-2xl">Dashboard</h1>
      <p>Hello!</p>

      <div className="mt-4 p-4 bg-gray-800 rounded">
        <h2 className="text-xl">Solana Wallet</h2>
        {!hasSavedKey ? (
          <>
            <input
              type="text"
              placeholder="Private key (base58)"
              value={privateKey}
              onChange={(e) => setPrivateKey(e.target.value)}
              className="text-black p-2 rounded w-full mt-2"
            />
            <button
              onClick={handleSavePrivateKey}
              className="mt-2 bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded cursor-pointer"
            >
              Import Wallet
            </button>
          </>
        ) : (
          <div>
            <p className="text-green-400">Wallet imported (data encrypted)</p>
            <div className="flex gap-2 mt-2">
              <button
                onClick={() => setIsChanging(true)}
                className="bg-yellow-600 hover:bg-yellow-700 px-3 py-1 rounded cursor-pointer"
              >
                Change
              </button>
              <button
                onClick={handleRemovePrivateKey}
                className="bg-red-600 hover:bg-red-700 px-3 py-1 rounded cursor-pointer"
              >
                Remove
              </button>
            </div>
          </div>
        )}
        {isChanging && (
          <div className="mt-2">
            <input
              type="text"
              placeholder="New private key (base58)"
              value={newPrivateKey}
              onChange={(e) => setNewPrivateKey(e.target.value)}
              className="text-black p-2 rounded w-full"
            />
            <div className="flex gap-2 mt-2">
              <button
                onClick={handleChangePrivateKey}
                className="bg-green-600 hover:bg-green-700 px-3 py-1 rounded cursor-pointer"
              >
                Apply
              </button>
              <button
                onClick={() => setIsChanging(false)}
                className="bg-gray-600 hover:bg-gray-700 px-3 py-1 rounded cursor-pointer"
              >
                Cancel
              </button>
            </div>
          </div>
        )}
        {error && <p className="text-red-400 mt-2">{error}</p>}
      </div>

      <button
        onClick={handleLock}
        className="mt-4 bg-red-600 hover:bg-red-700 px-4 py-2 rounded cursor-pointer"
      >
        Lock
      </button>
    </div>
  );
}

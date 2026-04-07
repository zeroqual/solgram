import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";

interface SetupPasswordProps {
  onSuccess: () => void;
  mode: "setup" | "unlock";
}

export default function SetupPassword({ onSuccess, mode }: SetupPasswordProps) {
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [error, setError] = useState("");

  const handleSubmit = async () => {
    setError("");
    if (mode === "setup" && password !== confirmPassword) {
      setError("Passwords do not match");
      return;
    }
    try {
      if (mode === "setup") {
        await invoke("setup_password", { password });
      } else {
        const success = await invoke<boolean>("unlock", { password });
        if (!success) {
          setError("Bad password");
          return;
        }
      }
      onSuccess();
    } catch (error: any) {
      setError(error.toString());
    }
  };
  return (
    <div className="flex flex-col gap-2 text-white">
      <p className="text-2xl text-center">
        {mode === "setup" ? "Create Master Password" : "Enter Master Password"}
      </p>
      <div className="relative">
        <input
          type={showPassword ? "text" : "password"}
          placeholder="Master Password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          className="text-white text-center bg-gray-800 rounded p-2 w-full pr-13"
        />
        <button
          type="button"
          onClick={() => setShowPassword(!showPassword)}
          className="absolute right-2 top-1/2 transform -translate-y-1/2 text-gray-400"
        >
          {showPassword ? "Hide" : "Show"}
        </button>
      </div>
      {mode === "setup" && (
        <input
          type={showPassword ? "text" : "password"}
          placeholder="Confirm Password"
          value={confirmPassword}
          onChange={(e) => setConfirmPassword(e.target.value)}
          className="text-white text-center bg-gray-800 rounded p-2 w-full"
        />
      )}
      {error && <p className="text-red-400 text-center">{error}</p>}
      <button
        onClick={handleSubmit}
        className="bg-gray-800 text-white rounded p-2 w-full cursor-pointer"
      >
        {mode === "setup" ? "Create" : "Unlock"}
      </button>
    </div>
  );
}

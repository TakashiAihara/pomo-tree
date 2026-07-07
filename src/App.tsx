import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

type Settings = {
  workMinutes: number;
  shortBreakMinutes: number;
  longBreakMinutes: number;
  pomodorosUntilLongBreak: number;
  autoStartNext: boolean;
};

type SaveState =
  | { kind: "idle" }
  | { kind: "saved" }
  | { kind: "error"; message: string };

function App() {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [saveState, setSaveState] = useState<SaveState>({ kind: "idle" });

  useEffect(() => {
    invoke<Settings>("get_settings").then(setSettings);
  }, []);

  if (settings === null) {
    return <main className="container">読み込み中…</main>;
  }

  const updateNumber = (key: keyof Settings) => (value: string) => {
    const parsed = Number(value);

    if (Number.isFinite(parsed)) {
      setSettings({ ...settings, [key]: parsed });
      setSaveState({ kind: "idle" });
    }
  };

  const save = async () => {
    try {
      await invoke("set_settings", { settings });
      setSaveState({ kind: "saved" });
    } catch (error) {
      setSaveState({ kind: "error", message: String(error) });
    }
  };

  return (
    <main className="container">
      <h1>pomo-tree 設定</h1>

      <form
        className="settings-form"
        onSubmit={(e) => {
          e.preventDefault();
          save();
        }}
      >
        <label>
          作業時間（分）
          <input
            type="number"
            min={1}
            max={180}
            value={settings.workMinutes}
            onChange={(e) => updateNumber("workMinutes")(e.currentTarget.value)}
          />
        </label>

        <label>
          短休憩（分）
          <input
            type="number"
            min={1}
            max={180}
            value={settings.shortBreakMinutes}
            onChange={(e) =>
              updateNumber("shortBreakMinutes")(e.currentTarget.value)
            }
          />
        </label>

        <label>
          長休憩（分）
          <input
            type="number"
            min={1}
            max={180}
            value={settings.longBreakMinutes}
            onChange={(e) =>
              updateNumber("longBreakMinutes")(e.currentTarget.value)
            }
          />
        </label>

        <label>
          長休憩までのポモドーロ数
          <input
            type="number"
            min={1}
            max={12}
            value={settings.pomodorosUntilLongBreak}
            onChange={(e) =>
              updateNumber("pomodorosUntilLongBreak")(e.currentTarget.value)
            }
          />
        </label>

        <label className="checkbox">
          <input
            type="checkbox"
            checked={settings.autoStartNext}
            onChange={(e) => {
              setSettings({ ...settings, autoStartNext: e.currentTarget.checked });
              setSaveState({ kind: "idle" });
            }}
          />
          次のセッションを自動で開始する
        </label>

        <button type="submit">保存</button>

        {saveState.kind === "saved" && <p className="status">保存しました</p>}
        {saveState.kind === "error" && (
          <p className="status error">{saveState.message}</p>
        )}
      </form>

      <p className="note">
        タイマー実行中に保存した場合、新しい時間は次のセッションから反映されます。
      </p>
    </main>
  );
}

export default App;

import { For, Show, createSignal } from "solid-js";
import { createStore } from "solid-js/store";
import logo from "./assets/logo.svg";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";

type LauncherJson = {
  latest: Latest
  versions: Version[]
}

type Latest = {
  release: string,
  snapshot: string
}

type Version = {
  id: string,
  type: string,
  url: string,
  time: string,
  release_time: string,
}

const DefaultLauncherjson: LauncherJson = {
  latest: {
    release: "",
    snapshot: ""
  },
  versions: []
}

function App() {
  const [ready, setReady] = createSignal(false);
  const [manifest, setMainfest] = createSignal(DefaultLauncherjson);
  const [snapshots, setSnapshot] = createSignal(false);

  async function version_manifest() {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    let json = await invoke<LauncherJson>("get_versions_manifest");
    setMainfest(json)
    console.log(json)
  }

  version_manifest().then(() => {
    setReady(true)
  })

  return (
    <div class="container">
      <h1>Launcher</h1>

      <div class="row">
        <div>
          <Show when={ready()}>
            <select>
              <For each={manifest().versions}>{(version) => {
                if (snapshots()) {
                  console.log("a")
                  return (
                    <option>
                      {version.id}
                    </option>
                  )
                } else {
                  console.log(version.type)
                  if (version.type !== "snapshot") {
                    return (
                      <option>
                        {version.id}
                      </option>
                    ) 
                  }
                }
              }}
              </For>
            </select>
          </Show>
          <button onclick={() => {
            setReady(false)
            setSnapshot(!snapshots())
            setReady(true)
            console.log(snapshots())
          }}>Snapshots?</button>
        </div>
      </div>

      <p>{}</p>
    </div>
  );
}

export default App;

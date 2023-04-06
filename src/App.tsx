import { For, JSXElement, Show, createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";
import { LauncherJson, Version, VersionJson } from "./types";

const DefaultLauncherjson: LauncherJson = {
  latest: {
    release: "",
    snapshot: ""
  },
  versions: []
}

function CreateVersionList(version: Version, snapshots: boolean): JSXElement {
  if (snapshots) {
    return (
      <option value={version.url}>
        {version.id}
      </option>
    )
  } else {
    if (version.type !== "snapshot") {
      return (
        <option value={version.url}>
          {version.id}
        </option>
      ) 
    }
  }
}

function App() {
  const [ready, setReady] = createSignal(false);
  const [manifest, setMainfest] = createSignal<LauncherJson>();
  const [snapshots, setSnapshot] = createSignal(false);

  let current_url: string;

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
          <select onClick={(event) => {
            current_url = event.currentTarget.value;
          }}>
            <Show when={ready()}>
              <For each={manifest()?.versions}>{(version) => {
                let local_manifest = manifest();
                if (typeof local_manifest === undefined) {
                  console.log("TODO: Fix me")
                } else {
                  if (
                    (snapshots() && version.id === local_manifest?.latest.snapshot) 
                    || version.id === local_manifest?.latest.release
                    ) {
                    current_url = version.url;
                  }
                  return CreateVersionList(version, snapshots())
                }
              }}
              </For>
            </Show>
          </select>
          <button onclick={() => {
            setReady(false)
            setSnapshot(!snapshots())
            setReady(true)
            console.log(snapshots())
          }}>Snapshots?</button>
        </div>
        <button disabled={false} onclick={(event) => {
          console.log("A")
          event.currentTarget.disabled = true
          invoke<VersionJson>("get_version_json", {url: current_url}).then((version_json) => {
            console.log(version_json.assetIndex)
            invoke<any>("get_assets", {indexUrl: version_json.assetIndex.url}).then((val) => console.log("A" + val))
          })
        }}>Get Stuff :tm:</button>
      </div>

      <p>{}</p>
    </div>
  );
}

export default App;

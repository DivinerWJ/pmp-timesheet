import { ref } from "vue";

const version = ref("—");
let loaded = false;

export function useAppVersion() {
  if (!loaded) {
    loaded = true;
    import("@tauri-apps/api/app")
      .then((m) => m.getVersion())
      .then((v) => {
        version.value = v;
      })
      .catch(() => {
        version.value = "—";
      });
  }
  return version;
}

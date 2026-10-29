/**
 * 更新下载线路。
 *
 * 每条线路是一组「前缀」：空前缀 = GitHub 直连，非空前缀是把原始地址拼在后面的镜像代理，
 * 例如 `https://ghproxy.net/` + `https://github.com/o/r/releases/download/...`。
 * 前缀的拼接与重试逻辑在 src-tauri/src/update.rs 里，这里只负责给出顺序。
 *
 * 镜像节点不在代码里写死，运行时从 https://hubp.tbedu.top/nodes.json 拉取
 * （见 src-tauri/src/update.rs 的 mirror_nodes 命令，带缓存与兜底）。
 */

/** 拉不到节点清单时的兜底，与 Rust 侧 FALLBACK_NODES 保持一致 */
export const FALLBACK_NODES = ["ghproxy.net", "ghfast.top", "gh-proxy.com"];

/** 自动模式下最多试几个镜像（直连失败后才走到这里） */
const AUTO_MIRROR_COUNT = 3;
/** 只走镜像时最多试几个节点 */
const MIRROR_ONLY_COUNT = 5;

/** 固定线路：自动 / 直连 / 镜像自动，其余 id 形如 node:<host> */
export const FIXED_SOURCE_IDS = ["auto", "github", "mirror"] as const;

export function nodePrefix(host: string): string {
  return `https://${host}/`;
}

export function nodeId(host: string): string {
  return `node:${host}`;
}

export function isNodeId(id: string): boolean {
  return id.startsWith("node:") && id.length > 5;
}

function hostsOf(nodes: string[]): string[] {
  return nodes.length ? nodes : FALLBACK_NODES;
}

/** 把用户选的线路 id 翻译成下发给后端的候选前缀顺序 */
export function buildSources(id: string, nodes: string[]): string[] {
  if (id === "github") return [""];

  const hosts = hostsOf(nodes);

  if (id === "mirror") {
    return hosts.slice(0, MIRROR_ONLY_COUNT).map(nodePrefix);
  }

  if (isNodeId(id)) {
    return [nodePrefix(id.slice(5))];
  }

  // auto：直连优先，失败再依次试镜像
  return ["", ...hosts.slice(0, AUTO_MIRROR_COUNT).map(nodePrefix)];
}

export function isKnownSource(id: string): boolean {
  return (FIXED_SOURCE_IDS as readonly string[]).includes(id) || isNodeId(id);
}

/** 下拉选项：三个固定线路 + 清单里的每个节点 */
export function buildSourceOptions(nodes: string[]): { value: string; label: string }[] {
  const options = [
    { value: "auto", label: "自动（直连优先，失败切镜像）" },
    { value: "github", label: "GitHub 直连" },
    { value: "mirror", label: "镜像 · 自动选节点" },
  ];
  for (const host of hostsOf(nodes)) {
    options.push({ value: nodeId(host), label: `镜像 · ${host}` });
  }
  return options;
}

/** 下载进度事件里带回来的线路前缀 → 展示用名称 */
export function sourceLabel(prefix: string): string {
  if (!prefix) return "GitHub 直连";
  return prefix.replace(/^https?:\/\//, "").replace(/\/$/, "");
}

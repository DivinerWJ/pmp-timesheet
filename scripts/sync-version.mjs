/**
 * 版本号单一来源同步脚本
 *
 * 以 package.json 的 version 为准，同步写入：
 *   - src-tauri/tauri.conf.json  （Tauri 打包版本，决定安装包与更新比对）
 *   - src-tauri/Cargo.toml       （[package] 段）
 *   - src-tauri/Cargo.lock       （本项目条目，避免构建后工作区变脏）
 *
 * 由 `npm version <patch|minor|major|x.y.z>` 自动触发（见 package.json 的 version 脚本）。
 */
import { readFileSync, writeFileSync } from "node:fs";

const ROOT = new URL("../", import.meta.url);
const path = (p) => new URL(p, ROOT);
const read = (p) => readFileSync(path(p), "utf8");
const write = (p, s) => writeFileSync(path(p), s);

const version = JSON.parse(read("package.json")).version;
const changed = [];

// 1) tauri.conf.json —— 重排为 2 空格缩进，保持与原文件一致的格式
const confPath = "src-tauri/tauri.conf.json";
const conf = JSON.parse(read(confPath));
if (conf.version !== version) {
  conf.version = version;
  write(confPath, JSON.stringify(conf, null, 2) + "\n");
  changed.push(confPath);
}

// 2) Cargo.toml —— 只改 [package] 段里的 version，避免误伤依赖声明
const cargoPath = "src-tauri/Cargo.toml";
const cargo = read(cargoPath);
const nextCargo = cargo.replace(
  /(\[package\][\s\S]*?\nversion = ")[^"]*(")/,
  `$1${version}$2`
);
if (nextCargo !== cargo) {
  write(cargoPath, nextCargo);
  changed.push(cargoPath);
}

// 3) Cargo.lock —— 本项目条目，避免 cargo 构建后留下未提交改动
const lockPath = "src-tauri/Cargo.lock";
const lock = read(lockPath);
const nextLock = lock.replace(
  /(name = "pmp-timesheet"\nversion = ")[^"]*(")/,
  `$1${version}$2`
);
if (nextLock !== lock) {
  write(lockPath, nextLock);
  changed.push(lockPath);
}

if (changed.length) {
  console.log(`版本号已同步到 ${version}：${changed.join("、")}`);
} else {
  console.log(`各处版本号已是 ${version}，无需改动`);
}

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWindow } from "@tauri-apps/api/window";

if (!window.__TAURI__) {
  console.error("[tauri-bridge] Tauri APIs not available — bridge not loaded");
}

function safeInvoke(cmd, args) {
  try {
    return invoke(cmd, args);
  } catch (err) {
    console.error(`[tauri-bridge] invoke('${cmd}') failed:`, err);
    throw err;
  }
}

window.api = {
  // ===== PROCESS (stubs — Tauri uses start_process with command) =====
  startProject: (id) => {
    console.warn("[tauri-bridge] startProject is not available in Tauri — use start_process command");
    return Promise.resolve(null);
  },
  stopProject: (id) => {
    console.warn("[tauri-bridge] stopProject is not available in Tauri — use stop_process command");
    return Promise.resolve(null);
  },
  restartProject: (id) => {
    console.warn("[tauri-bridge] restartProject is not available in Tauri — use restart_process command");
    return Promise.resolve(null);
  },
  sendInput: (id, data) => {
    console.warn("[tauri-bridge] sendInput is not available in Tauri — xterm input handled differently");
    return Promise.resolve(null);
  },

  // ===== TUNNEL =====
  startTunnel: (id, options) =>
    safeInvoke("start_tunnel", { projectId: id, ...options }),
  stopTunnel: (id) =>
    safeInvoke("stop_tunnel", { projectId: id }),
  getTunnelStatus: (id) =>
    safeInvoke("get_tunnel_status", { projectId: id }),
  getTunnelLogs: (id) =>
    safeInvoke("get_tunnel_logs", { projectId: id }),
  clearTunnelLogs: (id) =>
    safeInvoke("clear_tunnel_logs", { projectId: id }),
  getAllTunnelLogs: () => {
    console.warn("[tauri-bridge] getAllTunnelLogs not implemented");
    return Promise.resolve([]);
  },
  exportTunnelLogsProject: (id) => {
    console.warn("[tauri-bridge] exportTunnelLogsProject not implemented");
    return Promise.resolve(null);
  },
  exportTunnelLogsAll: () => {
    console.warn("[tauri-bridge] exportTunnelLogsAll not implemented");
    return Promise.resolve(null);
  },

  // ===== FILE SYSTEM =====
  readFile: (filePath) =>
    safeInvoke("plugin:fs|read_text_file", { path: filePath }),
  writeFile: (filePath, content) =>
    safeInvoke("plugin:fs|write_text_file", { path: filePath, contents: content }),
  createFile: (projectRoot, targetPath, type, content) => {
    const fullPath = `${projectRoot}/${targetPath}`;
    return safeInvoke("plugin:fs|write_text_file", { path: fullPath, contents: content || "" });
  },
  deletePath: (projectRoot, targetPath) => {
    const fullPath = `${projectRoot}/${targetPath}`;
    return safeInvoke("plugin:fs|remove", { path: fullPath, recursive: false });
  },
  renamePath: (projectRoot, oldPath, newPath) => {
    const fullOld = `${projectRoot}/${oldPath}`;
    const fullNew = `${projectRoot}/${newPath}`;
    return safeInvoke("plugin:fs|rename", { from: fullOld, to: fullNew });
  },
  copyFilesInto: async (projectRoot, destinationPath, sourcePaths) => {
    const dest = `${projectRoot}/${destinationPath}`;
    for (const src of sourcePaths) {
      await safeInvoke("plugin:fs|copy_file", { source: src, destination: dest });
    }
  },
  getPathForFile: (file) => {
    if (file && file.path) return file.path;
    if (typeof file === "string") return file;
    return String(file);
  },
  readDirectory: (path) =>
    safeInvoke("plugin:fs|read_dir", { path }),

  // ===== LOGS (stubs — no log service in Tauri backend) =====
  getLogs: (id) => {
    console.warn("[tauri-bridge] getLogs not implemented");
    return Promise.resolve([]);
  },
  getAllLogs: () => {
    console.warn("[tauri-bridge] getAllLogs not implemented");
    return Promise.resolve([]);
  },
  clearLogs: (id) => {
    console.warn("[tauri-bridge] clearLogs not implemented");
    return Promise.resolve(null);
  },
  exportConsoleLogsProject: (id) => {
    console.warn("[tauri-bridge] exportConsoleLogsProject not implemented");
    return Promise.resolve(null);
  },
  exportConsoleLogsAll: () => {
    console.warn("[tauri-bridge] exportConsoleLogsAll not implemented");
    return Promise.resolve(null);
  },
  getLogHistory: (id) => {
    console.warn("[tauri-bridge] getLogHistory not implemented");
    return Promise.resolve([]);
  },

  // ===== AUTO LAUNCH (stubs) =====
  isAutoLaunchEnabled: () => Promise.resolve(false),
  enableAutoLaunch: () => Promise.resolve(),
  disableAutoLaunch: () => Promise.resolve(),

  // ===== PROJECTS =====
  getProjects: () => safeInvoke("get_projects"),
  addProject: (project) => safeInvoke("add_project", { ...project }),
  deleteProject: (id) => safeInvoke("delete_project", { id }),
  updateProject: (project) => safeInvoke("update_project", { ...project }),
  reorderProjects: (payload) => {
    console.warn("[tauri-bridge] reorderProjects not implemented");
    return Promise.resolve();
  },
  reorderProjectsBulk: (payload) => {
    console.warn("[tauri-bridge] reorderProjectsBulk not implemented");
    return Promise.resolve();
  },
  getProjectStats: (id) => {
    console.warn("[tauri-bridge] getProjectStats not implemented");
    return Promise.resolve(null);
  },

  // ===== CATEGORIES =====
  getCategories: () => safeInvoke("get_categories"),
  addCategory: (category) => safeInvoke("add_category", { ...category }),
  deleteCategory: (id) => safeInvoke("delete_category", { id }),
  updateCategory: (category) => safeInvoke("update_category", { ...category }),
  reorderCategories: (orders) => {
    console.warn("[tauri-bridge] reorderCategories not implemented");
    return Promise.resolve();
  },

  // ===== DIALOG =====
  selectDirectory: () => open({ directory: true, multiple: false }),
  selectFile: () => open({ multiple: false }),
  openBackupFileDialog: () =>
    open({ multiple: false, filters: [{ name: "ZIP", extensions: ["zip"] }] }),

  // ===== SEARCH =====
  searchInProject: (root, query, options) =>
    safeInvoke("search_files", {
      directory: root,
      query,
      caseSensitive: options?.caseSensitive || false,
      wholeWord: options?.wholeWord || false,
      regexMode: options?.regexMode || false,
    }),

  // ===== GIT =====
  gitStatus: (projectPath) =>
    safeInvoke("git_get_status", { projectPath }),
  gitDiff: (projectPath, filePath) =>
    safeInvoke("git_diff_file", { projectPath, filePath }),
  gitAdd: async (projectPath, paths) => {
    for (const filePath of paths) {
      await safeInvoke("git_stage_file", { projectPath, filePath });
    }
  },
  gitUnstage: async (projectPath, paths) => {
    for (const filePath of paths) {
      await safeInvoke("git_unstage_file", { projectPath, filePath });
    }
  },
  gitCommit: (projectPath, message) =>
    safeInvoke("git_commit", { projectPath, message, authorName: "User", authorEmail: "user@local" }),
  gitPush: (projectPath) =>
    safeInvoke("git_push", { projectPath, remoteName: "origin" }),
  gitPull: (projectPath) =>
    safeInvoke("git_pull", { projectPath }),
  gitBranches: (projectPath) =>
    safeInvoke("git_get_branches", { projectPath }),
  gitCheckout: (projectPath, branchOrRef) =>
    safeInvoke("git_checkout_branch", { projectPath, branchName: branchOrRef }),
  gitClone: (repoUrl, targetPath) =>
    safeInvoke("git_clone", { url: repoUrl, destPath: targetPath }),
  gitRemoteUrl: (projectPath) => {
    console.warn("[tauri-bridge] gitRemoteUrl not implemented — use gitRemotes");
    return Promise.resolve("");
  },
  gitInit: (projectPath) => {
    console.warn("[tauri-bridge] gitInit not implemented");
    return Promise.resolve();
  },
  gitAddRemote: (projectPath, name, url) =>
    safeInvoke("git_add_remote", { projectPath, name, url }),
  gitRemotes: (projectPath) =>
    safeInvoke("git_get_remotes", { projectPath }),
  gitRemoveRemote: (projectPath, name) => {
    console.warn("[tauri-bridge] gitRemoveRemote not implemented");
    return Promise.resolve();
  },

  // ===== LSP (stubs — no LSP commands in Tauri backend) =====
  lspStart: (projectPath) => {
    console.warn("[tauri-bridge] lspStart not implemented");
    return Promise.resolve();
  },
  lspStop: (projectPath) => {
    console.warn("[tauri-bridge] lspStop not implemented");
    return Promise.resolve();
  },

  // ===== EXTERNAL / SYSTEM =====
  openExternal: (url) => safeInvoke("open_external", { url }),
  openPath: (path) => safeInvoke("open_folder", { path }),
  getDiscordInfo: (invitecode) => {
    console.warn("[tauri-bridge] getDiscordInfo not implemented");
    return Promise.resolve(null);
  },
  getAppPath: () => {
    console.warn("[tauri-bridge] getAppPath not implemented");
    return Promise.resolve("");
  },
  joinPath: (...args) => args.join("/"),
  getUserDataPath: () => {
    console.warn("[tauri-bridge] getUserDataPath not implemented");
    return Promise.resolve("");
  },
  getPlatformInfo: () => safeInvoke("get_platform_info"),
  validateMediaPath: (path) => safeInvoke("validate_media_path", { path }),

  // ===== SETTINGS =====
  getSettings: () => safeInvoke("get_app_settings"),
  updateSettings: (settings) => safeInvoke("update_app_settings", { settings }),

  // ===== VERSION / UPDATER =====
  getVersion: () => safeInvoke("get_current_version"),
  checkForUpdates: () => safeInvoke("check_for_updates"),
  startInstall: () => safeInvoke("download_and_install_update"),
  restartToApplyUpdate: () => {
    console.warn("[tauri-bridge] restartToApplyUpdate not implemented");
    return Promise.resolve();
  },
  getUpdateStatus: () => safeInvoke("get_update_status"),

  // ===== RUNTIME =====
  runtimeListAvailable: (type) =>
    safeInvoke(type === "node" ? "get_available_node_versions" : "get_available_python_versions"),
  runtimeListInstalled: (type) =>
    safeInvoke(type === "node" ? "get_installed_node_versions" : "get_installed_python_versions"),
  runtimeInstall: (type, version) =>
    safeInvoke(type === "node" ? "install_node_version" : "install_python_version", { version }),
  runtimeUninstall: (type, id, force = false) =>
    safeInvoke("uninstall_runtime_version", { runtime: type, version: id }),
  runtimeGetPath: (type, id) => {
    console.warn("[tauri-bridge] runtimeGetPath not implemented");
    return Promise.resolve("");
  },

  // ===== BACKUP =====
  exportConfig: (passphrase) => safeInvoke("create_backup", { passphrase }),
  importConfig: (passphrase, replaceExisting = true) =>
    safeInvoke("restore_backup", { zipPath: "", passphrase }),
  listLegacyBackupCandidates: () => {
    console.warn("[tauri-bridge] listLegacyBackupCandidates not implemented");
    return Promise.resolve([]);
  },
  restoreFromLegacyBackup: (filePath, replaceExisting) => {
    console.warn("[tauri-bridge] restoreFromLegacyBackup not implemented");
    return Promise.resolve(null);
  },
  listBackups: () => safeInvoke("list_backups"),
  deleteBackup: (filename) => safeInvoke("delete_backup", { filename }),

  // ===== FILE WATCHER =====
  watchFolder: (folderPath) => safeInvoke("watch_directory", { paths: [folderPath] }),
  stopWatchingFolder: (folderPath) => safeInvoke("unwatch_all"),

  // ===== TRAY =====
  refreshTrayMenu: () => safeInvoke("refresh_tray_menu"),

  // ===== WINDOW CONTROLS =====
  closeWindow: () => getCurrentWindow().close(),
  minimizeWindow: () => getCurrentWindow().minimize(),
  toggleMaximize: () => getCurrentWindow().toggleMaximize(),

  // ===== EVENTS =====
  onLog: (callback) => {
    let unlisten = () => {};
    listen("project:log", (e) => callback(e.payload)).then((fn) => { unlisten = fn; });
    return () => unlisten();
  },
  onLogsBatch: (callback) => {
    let unlisten = () => {};
    listen("project:logs-batch", (e) => callback(e.payload)).then((fn) => { unlisten = fn; });
    return () => unlisten();
  },
  onStatusChange: (callback) => {
    let unlisten = () => {};
    listen("project:status", (e) => callback(e.payload)).then((fn) => { unlisten = fn; });
    return () => unlisten();
  },
  onProjectStatusSync: (callback) => {
    let unlisten = () => {};
    listen("project:status-sync", (e) => callback(e.payload)).then((fn) => { unlisten = fn; });
    return () => unlisten();
  },
  onProjectsChange: (callback) => {
    let unlisten = () => {};
    listen("projects:list-changed", () => callback()).then((fn) => { unlisten = fn; });
    return () => unlisten();
  },
  onLogsCleared: (callback) => {
    let unlisten = () => {};
    listen("project:logs-cleared", (e) => callback(e.payload)).then((fn) => { unlisten = fn; });
    return () => unlisten();
  },
  onFileChange: (callback) => {
    let unlisten = () => {};
    listen("file-changed", (e) => callback(e.payload)).then((fn) => { unlisten = fn; });
    return () => unlisten();
  },
  onTunnelStatus: (callback) => {
    let unlisten = () => {};
    listen("tunnel:status", (e) => callback(e.payload)).then((fn) => { unlisten = fn; });
    return () => unlisten();
  },
  onTunnelLog: (callback) => {
    let unlisten = () => {};
    listen("tunnel:log", (e) => callback(e.payload)).then((fn) => { unlisten = fn; });
    return () => unlisten();
  },
  onProjectStats: (callback) => {
    let unlisten = () => {};
    listen("project:stats", (e) => callback(e.payload)).then((fn) => { unlisten = fn; });
    return () => unlisten();
  },
  onMaximize: (callback) => {
    let unlisten = () => {};
    listen("tauri://maximize", () => callback()).then((fn) => { unlisten = fn; });
    return () => unlisten();
  },
  onUnmaximize: (callback) => {
    let unlisten = () => {};
    listen("tauri://unmaximize", () => callback()).then((fn) => { unlisten = fn; });
    return () => unlisten();
  },
  onShutdown: (callback) => {
    let unlisten = () => {};
    listen("tauri://close-requested", () => callback()).then((fn) => { unlisten = fn; });
    return () => unlisten();
  },
  onUpdaterStatus: (callback) => {
    let unlisten = () => {};
    listen("update-available", (e) => callback(e.payload)).then((fn) => { unlisten = fn; });
    return () => unlisten();
  },
  onRuntimeProgress: (callback) => {
    let unlisten = () => {};
    listen("runtime:progress", (e) => callback(e.payload)).then((fn) => { unlisten = fn; });
    return () => unlisten();
  },

  // ===== REMOVE ALL LISTENERS =====
  removeAllListeners: (channel) => {
    console.warn(`[tauri-bridge] removeAllListeners('${channel}') — Tauri manages listeners differently`);
    return true;
  },
};

import { invoke } from "@tauri-apps/api/core";

export const showMainWindow = async () => invoke<void>("show_main_window");

export const switchAutoStart = async (currentState: boolean) => {
  await invoke("switch_auto_start", { currentState });
};

export const getAutostartState = async (): Promise<boolean> => {
  return await invoke<boolean>("auto_start_state");
};

export const checkExecutableFile = async () => {
  return await invoke<boolean>("check_executable_file");
};

export const turnOnProxy = async () => {
  await invoke("turn_on_proxy");
};

export const turnOffProxy = async () => {
  await invoke("turn_off_proxy");
};

export const proxyState = async () => {
  return await invoke<boolean>("proxy_state");
};

export const getTrojanProcessState = async () => {
  return await invoke<TrojanProcessState | null>("get_trojan_process_state");
};

export const downloadExecutableFile = async (id = 1) => {
  return await invoke<string>("download_executable_file", {
    id,
  });
};

export const getExecutableFile = async () => {
  return await invoke<string>("executable_file");
};

export const unzip = async (zip: Zip) => {
  await invoke("unzip", { zip });
};

export const execute = async () => {
  return await invoke<number>("execute");
};

export const checkNetworkConnectivity = async () => {
  return await invoke<number>("check_network_connectivity");
};

export const kill = async (pid?: number) => {
  await invoke("kill_process", { id: pid });
};

export const getConfig = async () => {
  return await invoke<Configuration>("get_config");
};

export const updateConfig = async (config: Configuration) =>
  await invoke("update_config", { config });

export const updateLocalAddr = async (addr: string) =>
  await invoke("update_local_addr", { addr });

export const updateLocalPort = async (port: number) =>
  await invoke("update_local_port", { port });

export const updatePac = async (pac: string) =>
  await invoke("update_pac", { pac });

export const addServerNode = async (serverNode: ServerNode) =>
  await invoke("add_server_node", { serverNode });

export const updateServerNode = async (index: number, serverNode: ServerNode) =>
  await invoke("update_server_node", { index, serverNode });

export const getUsingServerNode = async () =>
  await invoke<ServerNode | null>("get_using_server_node");

export const exit = async () => await invoke<never>("exit");

export const writeServerNodeToTrojanConfigFile = async (
  serverNode: ServerNode,
) => await invoke("write_trojan_config", { serverNode });

export const changeServerNode = async (name: string) =>
  await invoke("change_server_node", { name });

export const switchDaemon = async () => await invoke("switch_daemon");

export const getProxyDaemon = async () =>
  await invoke<boolean>("get_proxy_daemon");

export const setLogLevel = async (level: LogLevel) => {
  await invoke<void>("set_log_level", { level });
};

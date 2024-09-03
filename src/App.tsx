import {
  Show,
  createEffect,
  createResource,
  createSignal,
  onMount,
} from "solid-js";
import "~/App.scss";
import {
  getAutostartState,
  getConfig,
  getProxyDaemon,
  showMainWindow,
} from "./lib";
import { LazyRow } from "~/lazy";
import LocalAddr from "./app/local-addr";
import LocalPort from "./app/local-port";
import DarkSwitch from "./app/darkSwitch";
import Daemon from "./app/daemon";
import Autostart from "./app/autostart";
import ServerNodesTable from "./app/serverNodesTable";
import { checkTrojanProcess } from "./lib/proxy";
import { AppContext } from "./app/context";
import Download from "./app/download";
import Checking from "./app/checking";
import LogLevel from "./app/log-level";

const App = () => {
  const [autostartState, { mutate: mutateAutostartState }] =
    createResource(getAutostartState);

  const [configuration, { refetch: refetchConfiguration }] =
    createResource(getConfig);

  const [runningServerNode, { refetch: refetchRunningServerNode }] =
    createResource(checkTrojanProcess);

  const [
    proxyDaemon,
    { mutate: mutateProxyDaemon, refetch: refetchProxyDaemon },
  ] = createResource(getProxyDaemon);

  const [showDownloader, setShowDownloader] = createSignal<boolean>(false);

  onMount(() => {
    showMainWindow();
  });

  createEffect(() => {
    if (runningServerNode() === undefined) return;

    refetchProxyDaemon();
  });

  return (
    <div id="index">
      <Checking
        show={runningServerNode() === undefined}
        text="检测进程状态..."
      />

      <AppContext.Provider
        value={{
          download: { show: showDownloader, setShow: setShowDownloader },
          autostartState: {
            value: autostartState,
            mutate: mutateAutostartState,
          },
          configuration: {
            value: configuration,
            refetch: refetchConfiguration,
          },
          proxyDaemon: {
            value: proxyDaemon,
            refetch: refetchProxyDaemon,
          },
          runningServerNode: {
            value: runningServerNode,
            refetch: refetchRunningServerNode,
          },
        }}
      >
        <Show when={showDownloader()}>
          <Download />
        </Show>

        <div>
          <LazyRow>
            <LocalAddr
              value={configuration()?.local_addr}
              onChange={refetchConfiguration}
              disabled={!!runningServerNode()}
            />

            <LocalPort
              value={configuration()?.local_port}
              onChange={refetchConfiguration}
              disabled={!!runningServerNode()}
            />

            <Daemon />

            <Autostart />
            <DarkSwitch />
          </LazyRow>
        </div>

        <ServerNodesTable
          serverNodes={configuration()?.nodes}
          switch={() => refetchRunningServerNode()}
          refetch={refetchConfiguration}
          usingServerNodeName={runningServerNode()?.name}
          handleDaemon={mutateProxyDaemon}
          pac={configuration()?.pac ?? ""}
        />
      </AppContext.Provider>

      <LogLevel
        level={configuration()?.log_level ?? "Info"}
        disabled={!!runningServerNode()}
      />
    </div>
  );
};

export default App;

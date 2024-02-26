import { createEffect, createResource } from 'solid-js';
import '~/App.scss';
import { LazyRow } from '~/components';
import { getAutostartState, getConfig, getProxyDaemon } from './lib';
import LocalAddr from './app/local-addr';
import LocalPort from './app/local-port';
import DarkSwitch from './app/darkSwitch';
import Daemon from './app/daemon';
import Autostart from './app/autostart';
import ServerNodesTable from './app/serverNodesTable';
import { checkTrojanProcess } from './lib/proxy';
import { AppContext } from './app/context';

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

  createEffect(() => {
    if (runningServerNode() === undefined) return;

    refetchProxyDaemon();
  });

  return (
    <div id="index">
      <AppContext.Provider
        value={{
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
          pac={configuration()?.pac ?? ''}
        />
      </AppContext.Provider>
    </div>
  );
};

export default App;

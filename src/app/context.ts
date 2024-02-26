import { createContext } from 'solid-js';

interface AppContextType {
  autostartState: {
    value: () => boolean | undefined;
    mutate: (value: boolean) => void;
  };
  configuration: {
    value: () => Configuration | undefined;
    refetch: () => void;
  };
  proxyDaemon: { value: () => boolean | undefined; refetch: () => void };
  runningServerNode: {
    value: () => ServerNode | null | undefined;
    refetch: () => void;
  };
}

export const AppContext = createContext<AppContextType>();

interface TableContextType extends Pick<AppContextType, 'runningServerNode'> {
  proxyState: { value: () => boolean | undefined; refecth: () => void };
}

export const TableContext = createContext<TableContextType>();

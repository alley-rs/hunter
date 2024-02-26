import { createEffect, createMemo, useContext, createResource } from 'solid-js';
import { createStore } from 'solid-js/store';
import { LazyTable } from '~/lazy';
import type { TableProps } from './table/interface';
import { confirm } from '@tauri-apps/api/dialog';
import {
  addServerNode,
  changeServerNode,
  kill,
  proxyState as getProxyState,
  turnOffProxy,
  turnOnProxy,
  updateServerNode,
} from '~/lib';
import ServerNodesTableFooter from './serverNodesTableFooter';
import notify from '~/lib/notify';
import { AppContext, TableContext } from './context';

const columns: TableProps['columns'] = [
  { title: '名称', width: '11%', key: 'name', editable: true },
  { title: '地址', width: '20%', key: 'addr', editable: true },
  { title: '端口', width: '8%', key: 'port', editable: true },
  { title: '密码', width: '45%', key: 'password', editable: true },
  { title: '使用', key: 'using', width: '8%' },
];

interface ServerNodesTableProps {
  serverNodes?: ServerNode[];
  switch: (name: string) => void;
  usingServerNodeName?: string;
  refetch: () => void;
  handleDaemon: (value: boolean) => void;
  pac: string;
}

const ServerNodesTable = (props: ServerNodesTableProps) => {
  const { proxyDaemon, runningServerNode } = useContext(AppContext)!;

  const [proxyState, { mutate: mutateProxyState, refetch: refetchProxyState }] =
    createResource(getProxyState);

  const [serverNodes, setServerNodes] = createStore(props.serverNodes ?? []);

  createEffect(() => {
    props.serverNodes &&
      setServerNodes(
        props.serverNodes.map((o) => ({
          ...o,
          using: props.usingServerNodeName
            ? props.usingServerNodeName === o.name
            : false,
        })),
      );
  });

  const disableFooterAddButton = createMemo(
    () =>
      serverNodes.length > 0 &&
      (!serverNodes[serverNodes.length - 1].name ||
        !serverNodes[serverNodes.length - 1].addr ||
        !serverNodes[serverNodes.length - 1].port ||
        !serverNodes[serverNodes.length - 1].password),
  );

  const disableProxySwitch = createMemo(
    () =>
      serverNodes.length === 0 || serverNodes.findIndex((n) => n.using) === -1,
  );

  const onDeleteServerNode = async (index: number) => {
    const { name, addr, port, password } = serverNodes[index];

    const shouldWarning = name && addr && port && password;

    if (shouldWarning) {
      const ok = await confirm('此操作不可恢复', {
        title: '删除节点：' + serverNodes[index].name,
        type: 'warning',
        okLabel: '确认',
        cancelLabel: '取消',
      });

      if (!ok) return;

      if (serverNodes.length === 1) {
        setServerNodes([]);
        return;
      }
    }

    setServerNodes((prev) => [
      ...prev.slice(0, index),
      ...prev.slice(index + 1),
    ]);
  };

  const closeOtherNodes = (index: number) => {
    // 关闭其他开关
    setServerNodes(
      ({ name, using: state }) => name !== serverNodes[index].name && !!state,
      'using',
      false,
    );
  };

  const changeServerNodeAndTurnOnProxy = async (name: string) => {
    await changeServerNode(name);

    props.handleDaemon(true);

    const pacState = await getProxyState();
    if (!pacState) await turnOnProxy();

    notify(`开启代理并后台驻留，节点：` + name);
  };

  const toggleServerNode = async (index: number) => {
    const { name } = serverNodes[index];
    // 开启代理
    if (!serverNodes[index].using) {
      if (props.usingServerNodeName) {
        if (props.usingServerNodeName == name) {
          // 后台进程存在，且正在使用的节点名与当前选择的节点名相同，直接开启系统 pac
          const pacState = await getProxyState();
          if (!pacState) await turnOnProxy();
        } else {
          // 否则切换节点
          changeServerNodeAndTurnOnProxy(name);
        }
      } else {
        changeServerNodeAndTurnOnProxy(name);
      }

      // 设置 pac 后立即获取 pac 状态可能会有延迟，不能通过 refecth 设置 pac 状态
      mutateProxyState(true);
    } else {
      // 关闭代理并杀死后台进程
      await turnOffProxy();

      await kill();

      proxyDaemon.refetch();
      runningServerNode.refetch();

      // 设置 pac 后立即获取 pac 状态可能会有延迟，不能通过 refecth 设置 pac 状态
      mutateProxyState(false);

      notify('已关闭系统代理');
    }

    // 切换 index 开关
    setServerNodes(index, 'using', (checked) => {
      props.switch(serverNodes[index].name);
      return !checked;
    });
  };

  const onGlobalChange = (index: number) => {
    closeOtherNodes(index);

    toggleServerNode(index);
  };

  const addNewServerNode = () => {
    setServerNodes(serverNodes.length, {
      name: '',
      addr: '',
      port: 443,
      password: '',
      defaultEditing: true,
      using: false,
      isNewAdded: true,
    });
  };

  const onSaveNewServerNode = async (index: number, serverNode: ServerNode) => {
    if (index === serverNodes.length) {
      // 添加节点
      await addServerNode(serverNode);
    } else {
      // 修改节点
      await updateServerNode(index, serverNode);
    }
    props.refetch();
  };

  return (
    <LazyTable
      size="small"
      style={{ 'margin-top': '20px' }}
      columns={columns}
      data={serverNodes}
      onGlobalChange={onGlobalChange}
      footer={
        <TableContext.Provider
          value={{
            proxyState: { value: proxyState, refecth: refetchProxyState },
            runningServerNode,
          }}
        >
          <ServerNodesTableFooter
            addNewServerNode={addNewServerNode}
            disableAddButton={disableFooterAddButton()}
            disableProxySwitch={disableProxySwitch()}
            pac={props.pac}
          />
        </TableContext.Provider>
      }
      actions={[
        {
          editing: {
            tooltip: '编辑',
          },
          cancel: { tooltip: '取消' },
          edited: {
            tooltip: '保存',
            onClick: onSaveNewServerNode,
          },
        },
        {
          tooltip: '删除',
          onClick: onDeleteServerNode,
        },
      ]}
    />
  );
};

export default ServerNodesTable;

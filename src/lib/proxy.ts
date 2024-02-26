import { confirm } from '@tauri-apps/api/dialog';
import {
  writeServerNodeToTrojanConfigFile,
  getTrojanProcessState,
  kill,
  exit,
} from '.';

export const switchToServerNode = async (serverNode: ServerNode) => {
  await writeServerNodeToTrojanConfigFile(serverNode);
};

export const checkTrojanProcess = async () => {
  const state = await getTrojanProcessState();

  if (!state) return null;

  if (state.type === 'OTHER' || state.type === 'INVALID') {
    const { pid } = state;
    if (pid) {
      const ok = await confirm(
        `检测到 trojan [pid=${pid}] 正在运行，但不是由本程序${state.type === 'OTHER' ? '启动' : '配置的有效节点'
        }，是否终止此进程？`,
        {
          title: '进程冲突',
          okLabel: '终止此进程',
          cancelLabel: '关闭本程序',
        },
      );

      if (ok) {
        await kill(pid);
        return null;
      } else {
        return await exit();
      }
    }
  } else {
    return state as ServerNode;
  }
};

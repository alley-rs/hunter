import { BiRegularLinkAlt, BiRegularPlus } from 'solid-icons/bi';
import { TbWashDryclean, TbWashDrycleanOff } from 'solid-icons/tb';
import { createSignal, useContext } from 'solid-js';
import {
  LazyButton,
  LazyFlex,
  LazySpace,
  LazySwitch,
  LazyToast,
  LazyTooltip,
} from '~/lazy';
import { checkNetworkConnectivity, turnOffProxy, turnOnProxy } from '~/lib';
import Pac from './pac';
import notify from '~/lib/notify';
import { TableContext } from './context';
import { AlertProps } from 'alley-components';

interface ServerNodesTableFooterProps {
  addNewServerNode: () => void;
  disableAddButton: boolean;
  disableProxySwitch: boolean;
  pac: string;
}

const ServerNodesTableFooter = (props: ServerNodesTableFooterProps) => {
  const { runningServerNode, proxyState } = useContext(TableContext)!;

  const [checkingConnect, setCheckingConnect] = createSignal(false);
  const [connectState, setConnectState] = createSignal<{
    type: AlertProps['type'];
    message: AlertProps['message'];
  } | null>(null);

  const handleSwitchProxy = async (value: boolean) => {
    if (value) {
      await turnOnProxy();
      notify('已开启系统代理');
    } else {
      await turnOffProxy();
      notify('已关闭系统代理');
    }

    proxyState.refecth();
  };

  const checkConnect = async () => {
    setCheckingConnect(true);
    try {
      const cost = await checkNetworkConnectivity();
      const costFormatted =
        cost > 1 ? cost.toFixed(1) + 's' : Math.round(cost * 1000) + 'ms';

      setConnectState({
        type: 'success',
        message: `网络已连通，访问谷歌用时：${costFormatted}`,
      });
    } catch (e) {
      setConnectState({
        type: 'error',
        message: String(e),
      });
    } finally {
      setCheckingConnect(false);
    }
  };

  return (
    <LazyFlex justify="between" align="center">
      <LazyToast
        open={connectState() !== null}
        placement="bottom"
        onClose={() => setConnectState(null)}
        alert={{
          type: connectState()?.type ?? 'success',
          showIcon: true,
          message: connectState()?.message,
        }}
      />

      <LazyTooltip
        text="检测代理是否有效。发起一个真正的网络请求并计算整个过程的耗时。"
        placement="bottom"
        disabled={!proxyState.value()}
      >
        <LazyButton
          isLoading={checkingConnect()}
          icon=<BiRegularLinkAlt />
          shape="circle"
          size="small"
          onClick={checkConnect}
          disabled={!proxyState.value()}
        />
      </LazyTooltip>

      <Pac
        value={props.pac}
        onChange={proxyState.refecth}
        disabled={proxyState.value()}
      />

      <LazySpace gap={10}>
        <LazyTooltip
          text="添加服务器节点"
          placement="bottom"
          disabled={props.disableAddButton}
        >
          <LazyButton
            shape="circle"
            size="small"
            icon={<BiRegularPlus />}
            style={{ display: 'inline-flex' }}
            disabled={props.disableAddButton}
            onClick={props.addNewServerNode}
          />
        </LazyTooltip>

        <LazyTooltip
          text={(proxyState.value() ? '关闭' : '开启') + '系统代理'}
          placement="bottom"
          disabled={!runningServerNode.value()}
        >
          <LazySwitch
            disabled={!runningServerNode.value()}
            checked={proxyState.value()}
            setChecked={handleSwitchProxy}
            size="small"
            checkedChild={<TbWashDryclean />}
            uncheckedChild={<TbWashDrycleanOff />}
          />
        </LazyTooltip>
      </LazySpace>
    </LazyFlex>
  );
};

export default ServerNodesTableFooter;

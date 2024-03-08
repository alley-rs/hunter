import { BiRegularCheck, BiRegularX } from 'solid-icons/bi';
import { useContext } from 'solid-js';
import { LazyCol, LazySwitch, LazyTooltip } from '~/lazy';
import { switchDaemon } from '~/lib';
import { AppContext } from './context';

const Daemon = () => {
  const { proxyDaemon, runningServerNode } = useContext(AppContext)!;

  const onChange = async () => {
    await switchDaemon();
    proxyDaemon?.refetch();
  };

  return (
    <LazyCol span={4} align="center">
      <label>后台驻留</label>

      <LazyTooltip
        text="开启一个节点后将默认后台驻留核心进程，不需要后台驻留需手动关闭。"
        placement="bottom"
        disabled={!runningServerNode.value()}
      >
        <LazySwitch
          checked={proxyDaemon?.value()}
          size="small"
          setChecked={onChange}
          checkedChild={<BiRegularCheck />}
          uncheckedChild={<BiRegularX />}
          disabled={!runningServerNode.value()}
        />
      </LazyTooltip>
    </LazyCol>
  );
};

export default Daemon;

import { createMemo, useContext } from 'solid-js';
import { BiRegularCheck, BiRegularX } from 'solid-icons/bi';
import { LazyCol, LazySwitch, LazyTooltip } from '~/components';
import { AppContext } from './context';
import { switchAutoStart } from '~/lib';

const Autostart = () => {
  const { autostartState, proxyDaemon, runningServerNode } =
    useContext(AppContext)!;

  const disabled = createMemo(
    () => !proxyDaemon.value() || !runningServerNode.value(),
  );

  const onChange = async (value: boolean) => {
    await switchAutoStart(!value);
    autostartState.mutate(value);
  };

  return (
    <LazyCol span={4} align="center">
      <label>开机自启</label>

      <LazyTooltip
        text="开机时开启后台代理服务而非本程序，仅在开启后台驻留后可用"
        placement="bottom"
        disabled={disabled()}
      >
        <LazySwitch
          checked={autostartState.value()}
          size="small"
          setChecked={onChange}
          checkedChild={<BiRegularCheck />}
          uncheckedChild={<BiRegularX />}
          disabled={disabled()}
        />
      </LazyTooltip>
    </LazyCol>
  );
};

export default Autostart;

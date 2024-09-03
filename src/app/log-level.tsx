import { createEffect, createSignal } from "solid-js";
import { LazySpace, LazySwitch, LazyTooltip } from "~/lazy";
import { setLogLevel } from "~/lib/api";
import { BiRegularCheck, BiRegularX } from "solid-icons/bi";

interface LogLevelProps {
  level: LogLevel;
  disabled?: boolean;
}

const LogLevel = (props: LogLevelProps) => {
  const [level, setLevel] = createSignal(props.level);

  createEffect(() => setLevel(props.level));

  return (
    <LazySpace justify="center">
      <label>调试</label>

      <LazyTooltip
        text="当你需要查看 trojan-go 日志时才需要开启此项，否则没必要开启。仅在关闭后台进程时可用。"
        placement="bottom"
      >
        <LazySwitch
          disabled={props.disabled}
          size="small"
          checkedChild={<BiRegularCheck />}
          uncheckedChild={<BiRegularX />}
          checked={level() === "Trace"}
          setChecked={(checked) => {
            const newLevel: LogLevel = checked ? "Trace" : "Info";
            setLevel(newLevel);
            setLogLevel(newLevel);
          }}
        />
      </LazyTooltip>
    </LazySpace>
  );
};

export default LogLevel;

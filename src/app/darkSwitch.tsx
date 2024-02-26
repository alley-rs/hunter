import { BiRegularSun, BiSolidMoon } from 'solid-icons/bi';
import { LazyCol, LazySwitch, LazyTooltip } from '~/components';
import useDark from './hooks/useDark';

const DarkSwitch = () => {
  const [isDark, setIsDark] = useDark();

  return (
    <LazyCol span={2} align="end" style={{ position: 'relative' }}>
      <LazyTooltip text={`切换为${isDark() ? '亮' : '暗'}色`} placement="left">
        <LazySwitch
          checked={isDark()}
          setChecked={() => {
            setIsDark((pre) => {
              return !pre;
            });
          }}
          uncheckedChild={<BiRegularSun />}
          checkedChild={<BiSolidMoon />}
          class="dark-switch"
        />
      </LazyTooltip>
    </LazyCol>
  );
};

export default DarkSwitch;

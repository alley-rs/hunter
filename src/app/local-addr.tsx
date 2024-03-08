import { AiOutlineCheck } from 'solid-icons/ai';
import { createSignal } from 'solid-js';
import { LazyButton, LazyCol, LazyInput, LazySpaceCompact } from '~/lazy';
import { updateLocalAddr } from '~/lib';

interface LocalAddrProps {
  value?: string;
  disabled?: boolean;
  onChange: () => void;
}

const LocalAddr = (props: LocalAddrProps) => {
  const [addr, setAddr] = createSignal(props.value);

  const onClick = async () => {
    await updateLocalAddr(addr()!);
    props.onChange();
  };

  return (
    <LazyCol span={8} align="center">
      <label>本地地址</label>

      <LazySpaceCompact size="small">
        <LazyInput
          value={addr()}
          placeholder="输入本地地址"
          onChange={setAddr}
          style={{ width: '120px' }}
          disabled={props.disabled}
        />

        <LazyButton
          disabled={!addr() || addr() === props.value}
          icon={<AiOutlineCheck />}
          onClick={onClick}
        />
      </LazySpaceCompact>
    </LazyCol>
  );
};

export default LocalAddr;

import { createSignal } from 'solid-js';
import { AiOutlineCheck } from 'solid-icons/ai';
import { LazyButton, LazyCol, LazyInputNumber, LazySpaceCompact } from '~/lazy';
import { updateLocalPort } from '~/lib';

interface LocalPortProps {
  value?: number;
  onChange: () => void;
  disabled?: boolean;
}

const LocalPort = (props: LocalPortProps) => {
  const [port, setPort] = createSignal(props.value);

  const onClick = async () => {
    await updateLocalPort(port()!);
    props.onChange();
  };

  return (
    <LazyCol span={6} align="center">
      <label>本地端口</label>

      <LazySpaceCompact size="small">
        <LazyInputNumber
          value={port()}
          placeholder="输入端口"
          disabled={props.disabled}
          onChange={setPort}
        />

        <LazyButton
          icon={<AiOutlineCheck />}
          disabled={!port() || port() === props.value}
          onClick={onClick}
        />
      </LazySpaceCompact>
    </LazyCol>
  );
};

export default LocalPort;

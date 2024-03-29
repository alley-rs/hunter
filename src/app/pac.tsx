import { AiOutlineCheck } from 'solid-icons/ai';
import { createEffect, createSignal } from 'solid-js';
import { LazyButton, LazyInput, LazySpaceCompact } from '~/lazy';
import { updatePac } from '~/lib';

interface PacProps {
  value?: string;
  onChange: () => void;
  disabled?: boolean;
}

const Pac = (props: PacProps) => {
  const [lastPac, setLastPac] = createSignal(props.value);
  const [pac, setPac] = createSignal(props.value);

  createEffect(() => {
    console.log(pac(), lastPac());
  });

  const onClick = async () => {
    await updatePac(pac()!);

    props.onChange();
    setLastPac(pac());
  };

  return (
    <LazySpaceCompact style={{ width: '620px' }}>
      <LazyInput
        placeholder="输入 pac 地址"
        value={pac()}
        onChange={setPac}
        disabled={props.disabled}
        onClick={(e) => {
          e.currentTarget.scrollBy({ left: e.currentTarget.scrollWidth });
        }}
      />

      <LazyButton
        icon={<AiOutlineCheck />}
        disabled={!pac() || lastPac() === pac()}
        onClick={onClick}
      />
    </LazySpaceCompact>
  );
};

export default Pac;

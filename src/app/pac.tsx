import { AiOutlineCheck } from 'solid-icons/ai';
import { createSignal } from 'solid-js';
import { LazyButton, LazyInput, LazySpaceCompact } from '~/lazy';
import { updatePac } from '~/lib';

interface PacProps {
  value?: string;
  onChange: () => void;
  disabled?: boolean;
}

const Pac = (props: PacProps) => {
  const [pac, setPac] = createSignal(props.value);

  const onClick = async () => {
    await updatePac(pac()!);

    props.onChange();
  };

  return (
    <LazySpaceCompact style={{ width: '620px' }}>
      <LazyInput
        placeholder="输入 pac 地址"
        value={pac()}
        onChange={setPac}
        disabled={props.disabled}
        onClick={(e) => {
          console.log('点击');
          console.log(e.currentTarget.scrollWidth);
          e.currentTarget.scrollBy({ left: e.currentTarget.scrollWidth });
        }}
      />

      <LazyButton
        icon={<AiOutlineCheck />}
        disabled={!pac() || pac() === props.value}
        onClick={onClick}
      />
    </LazySpaceCompact>
  );
};

export default Pac;

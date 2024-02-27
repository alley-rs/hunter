import { Show } from 'solid-js';
import { LazySpinner } from '~/components';

interface CheckingProps {
  show?: boolean;
  text: string;
}

const Checking = (props: CheckingProps) => {
  return (
    <Show when={props.show}>
      <div id="checking">
        <LazySpinner
          size="large"
          color="var(--color-primary)"
          thickness="4px"
          style={{ 'margin-bottom': '10px' }}
        />

        {props.text}
      </div>
    </Show>
  );
};

export default Checking;

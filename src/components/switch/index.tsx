import { JSXElement } from 'solid-js';
import { addClassNames } from '../utils/class';
import './index.scss';

interface SwitchProps {
  class?: string;
  checked?: boolean;
  setChecked: (checked: boolean) => void;
  disabled?: boolean;
  checkedChild?: JSXElement;
  uncheckedChild?: JSXElement;
  size?: 'large' | 'middle' | 'small';
}

const baseClassName = 'switch';

const Switch = (props: SwitchProps) => {
  const classNames = () =>
    addClassNames(
      baseClassName,
      props.size && `${baseClassName}-${props.size}`,
      props.checked && `${baseClassName}-checked`,
      props.disabled && `${baseClassName}-disabled`,
      props.class,
    );

  const onClick = () => {
    if (props.disabled) return;

    props.setChecked(!props.checked);
  };

  return (
    <div class={classNames()} onClick={onClick}>
      <div class={`${baseClassName}-checkbox`}>
        <div class={`${baseClassName}-inner`}>
          {props.checked ? props.checkedChild : props.uncheckedChild}
        </div>
      </div>
    </div>
  );
};

export default Switch;

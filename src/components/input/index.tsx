import type { JSXElement, JSX } from 'solid-js';
import { createSignal, onMount } from 'solid-js';
import { addClassNames } from '../utils';
import './index.scss';

interface InputProps {
  class?: string;
  placeholder?: string;
  disabled?: boolean;
  value?: string | number;
  onChange?: (value: string) => void;
  style?: JSX.CSSProperties;
  size?: SizeType;
  autofocus?: boolean;
  onFocus?: (e: FocusEvent & { target: HTMLInputElement }) => void;
  onClick?: (
    e: MouseEvent & {
      currentTarget: HTMLInputElement;
      target: JSXElement;
    },
  ) => void;
}

const baseClassName = 'alley-input';

const Input = (props: InputProps) => {
  // ref 用来获取 compact 样式
  let ref: HTMLInputElement | undefined;

  // 保存 compact 样式
  const [classList, setClassList] = createSignal<string[]>([]);

  onMount(() => {
    const arr: string[] = [];
    ref!.classList.forEach((v) => arr.push(v));

    setClassList(arr);
  });

  const className = () => {
    // 之前有 compact 样式时只修改 disabled
    if (
      classList().length &&
      classList().includes(`${baseClassName}-compact-item`)
    ) {
      const disabledClass = `${baseClassName}-disabled`;
      if (!props.disabled && classList().includes(disabledClass)) {
        setClassList((prev) => {
          const idx = prev.indexOf(disabledClass);

          return [...prev.slice(0, idx), ...prev.slice(idx + 1)];
        });
      } else if (props.disabled && !classList().includes(disabledClass)) {
        setClassList((prev) => [...prev, disabledClass]);
      }

      return classList().join(' ');
    }

    return addClassNames(
      baseClassName,
      props.size && `${baseClassName}-${props.size}`,
      props.disabled && `${baseClassName}-disabled`,
    );
  };

  return (
    <input
      ref={ref}
      class={className()}
      placeholder={props.placeholder}
      value={props.value ?? ''}
      onChange={(e) => props.onChange?.(e.target.value)}
      disabled={props.disabled}
      style={props.style}
      autofocus={props.autofocus}
      onFocus={props.onFocus}
      onClick={props.onClick}
    />
  );
};

export default Input;

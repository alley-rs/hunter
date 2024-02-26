import { For } from 'solid-js';
import type { JSX, JSXElement } from 'solid-js';
import { addClassNames } from '~/components/utils';
import './index.scss';

const baseClassName = 'space';

interface SpaceProps {
  class?: string;
  children: JSXElement;
  wrap?: boolean;
  direction?: 'vertical' | 'horizontal';
  gap?: number | string;
  block?: boolean;
  onClick?: () => void;
  align?: 'start' | 'end' | 'center' | 'baseline';
  justify?:
  | 'start'
  | 'end'
  | 'center'
  | 'between'
  | 'around'
  | 'evenly'
  | 'stretch';
  style?: JSX.CSSProperties;
}

const Space = (props: SpaceProps) => {
  const classNames = () =>
    addClassNames(
      baseClassName,
      props.class,
      props.wrap ? `${baseClassName}-wrap` : undefined,
      `${baseClassName}-${props.direction || 'horizontal'}`,
      props.block ? `${baseClassName}-block` : undefined,
      props.align ? `${baseClassName}-align-${props.align}` : undefined,
      props.justify ? `${baseClassName}-justify-${props.justify}` : undefined,
    );

  const style = (): JSX.CSSProperties => ({
    ...props.style,
    '--gap': props.gap
      ? typeof props.gap === 'number'
        ? `${props.gap}px`
        : props.gap
      : 0,
  });

  return (
    <div class={classNames()} style={style()}>
      <For
        each={Array.isArray(props.children) ? props.children : [props.children]}
      >
        {(item) => <div class={`${baseClassName}-item`}>{item}</div>}
      </For>
    </div>
  );
};

export default Space;

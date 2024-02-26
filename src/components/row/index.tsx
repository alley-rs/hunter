import type { JSXElement, JSX } from 'solid-js';
import './index.scss';
import { addClassNames } from '../utils/class';

interface RowProps {
  class?: string;
  children: JSXElement;
  gutter?: number;
  style?: JSX.CSSProperties;
  onClick?: () => void;
}

const baseClassName = 'row';

const Row = (props: RowProps) => {
  const className = () => addClassNames(baseClassName, props.class || '');

  const style = () =>
    props.gutter
      ? {
        ...props.style,
        'row-gap': `${props.gutter}px`,
      }
      : props.style;

  return (
    <div class={className()} style={style()} onClick={props.onClick}>
      {props.children}
    </div>
  );
};

export default Row;

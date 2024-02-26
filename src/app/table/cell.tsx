import type { JSXElement } from 'solid-js';

interface TableCellProps {
  children: JSXElement;
}

const baseClassName = 'alley-table';

const TableCell = (props: TableCellProps) => (
  <td class={`${baseClassName}-cell`}>{props.children}</td>
);

export default TableCell;

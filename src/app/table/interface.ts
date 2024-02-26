import type { JSXElement, JSX } from 'solid-js';

interface BaseTableColumn {
  title: string;
  width?: string | number;
  class?: string;
}

export interface TableColumn extends BaseTableColumn {
  editable?: boolean;
  key: keyof ServerNode;
}

export interface BaseTableActionColumn {
  tooltip?: string;
  icon?: JSXElement;
  onClick: (index: number) => void;
}

export interface EditTableActionColumn {
  editing: Omit<BaseTableActionColumn, 'onClick'>;
  cancel: Omit<BaseTableActionColumn, 'onClick'>;
  edited: Omit<BaseTableActionColumn, 'onClick'> & {
    onClick: (index: number, value: ServerNode) => void;
  };
}

export type TableActionColumn = [EditTableActionColumn, BaseTableActionColumn];

export interface TableProps {
  class?: string;
  columns: TableColumn[];
  data: ServerNode[];
  size?: 'large' | 'middle' | 'small';
  actions?: TableActionColumn;
  footer?: JSXElement;
  style?: JSX.CSSProperties;
  onGlobalChange?: (
    index: number,
    changing?: { key: string; value: string | number | boolean },
  ) => void;
}

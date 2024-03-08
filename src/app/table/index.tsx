import { For, Show, mergeProps } from 'solid-js';
import { addClassNames } from 'alley-components';
import './index.scss';
import { LazyEmpty } from '~/lazy';
import { TableProps } from './interface';
import TableEditableRow from './editable-row';

const baseClassName = 'alley-table';

const Table = (props: TableProps) => {
  const merged = mergeProps({ size: 'large' }, props);

  const className = () =>
    addClassNames(
      baseClassName,
      `${baseClassName}-${merged.size}`,
      merged.class,
    );

  return (
    <div class={className()} style={props.style}>
      <div class={`${baseClassName}-container`}>
        <div class={`${baseClassName}-content`}>
          <table style={{ 'table-layout': 'auto' }}>
            <colgroup>
              <For
                each={
                  merged.actions
                    ? [...merged.columns, { title: '操作', width: undefined }]
                    : merged.columns
                }
              >
                {(item) => (
                  <col
                    style={
                      item.width
                        ? {
                            width:
                              typeof item.width === 'string'
                                ? item.width
                                : `${item.width}px`,
                          }
                        : undefined
                    }
                  />
                )}
              </For>
            </colgroup>
            <thead class={`${baseClassName}-thead`}>
              <tr>
                <For
                  each={
                    merged.actions
                      ? [
                          ...merged.columns,
                          { title: '操作', width: undefined, class: undefined },
                        ]
                      : merged.columns
                  }
                >
                  {(item) => (
                    <th
                      class={addClassNames(`${baseClassName}-cell`, item.class)}
                    >
                      {item.title}
                    </th>
                  )}
                </For>
              </tr>
            </thead>
            <tbody class={`${baseClassName}-tbody`}>
              <Show
                when={merged.data.length}
                fallback={
                  <tr class={`${baseClassName}-placeholder`}>
                    <td class={`${baseClassName}-cell`} colspan="6">
                      <LazyEmpty description="无节点" />
                    </td>
                  </tr>
                }
              >
                <For each={merged.data}>
                  {(record, i) => (
                    <TableEditableRow
                      index={i()}
                      record={record}
                      columns={props.columns}
                      actions={props.actions}
                      defaultEditing={!!record['defaultEditing']}
                      onGlobalChange={props.onGlobalChange}
                    />
                  )}
                </For>
              </Show>
            </tbody>
          </table>
        </div>
      </div>

      <Show when={props.footer}>
        <div class={`${baseClassName}-footer`}>{props.footer}</div>
      </Show>
    </div>
  );
};

export default Table;

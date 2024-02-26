import {
  For,
  JSXElement,
  Match,
  Show,
  Switch,
  createMemo,
  createSignal,
} from 'solid-js';
import TabelCell from './cell';
import {
  BaseTableActionColumn,
  EditTableActionColumn,
  TableColumn,
  TableProps,
} from './interface';
import {
  BiRegularCheck,
  BiRegularEdit,
  BiRegularTrash,
  BiRegularX,
} from 'solid-icons/bi';
import {
  LazyButton,
  LazyFlex,
  LazyInput,
  LazyInputNumber,
  LazySwitch,
  LazyTooltip,
  LazySpace,
} from '~/components';
import { deepEqual } from '~/lib';

interface InputCellProps {
  index: number;
  defaultEditing?: boolean;
  value: string | number;
  onChange: (value: string | number) => void;
}

const InputCell = (props: InputCellProps) => {
  return (
    <LazyFlex>
      <Switch>
        <Match when={typeof props.value === 'string'}>
          <LazyInput
            size="small"
            value={props.value}
            onChange={props.onChange}
            autofocus={props.index === 0 && props.defaultEditing}
          />
        </Match>

        <Match when={typeof props.value === 'number'}>
          <LazyInputNumber
            value={props.value as number}
            min={1}
            max={65535}
            onChange={props.onChange}
            size="small"
          />
        </Match>
      </Switch>
    </LazyFlex>
  );
};

interface SwitchServerNodeProps {
  state: boolean;
  disabled: boolean;
  onChange: (value: boolean) => void;
}

const SwitchServerNode = (props: SwitchServerNodeProps) => {
  return (
    <LazyTooltip
      text={props.state ? '退出后台进程并关闭系统代理' : '使用此节点开启代理'}
    >
      <LazySwitch
        checked={props.state}
        size="small"
        checkedChild={<BiRegularCheck />}
        uncheckedChild={<BiRegularX />}
        disabled={props.disabled}
        setChecked={props.onChange}
      />
    </LazyTooltip>
  );
};

interface TooltipActionProps {
  index: number;
  action:
  | Omit<BaseTableActionColumn, 'onClick'>
  | Omit<EditTableActionColumn['edited'], 'onClick'>
  | EditTableActionColumn['editing'];
  onClick: () => void;
  icon: JSXElement;
  disabled?: boolean;
}

const TooltipAction = (props: TooltipActionProps) => {
  return (
    <Show
      when={props.action.tooltip}
      fallback={
        <LazyButton
          shape="circle"
          icon={props.icon}
          onClick={props.onClick}
          size="small"
          filter
        />
      }
    >
      <LazyTooltip
        text={props.action.tooltip!}
        placement="top"
        disabled={props.disabled}
      >
        <LazyButton
          shape="circle"
          icon={props.icon}
          onClick={props.onClick}
          size="small"
          filter
          disabled={props.disabled}
        />
      </LazyTooltip>
    </Show>
  );
};

interface DeleteActionProps {
  index: number;
  action: BaseTableActionColumn;
  using?: boolean;
}

const DeleteAction = (props: DeleteActionProps) => {
  return (
    <TooltipAction
      index={props.index}
      action={props.action}
      disabled={props.using}
      icon={<BiRegularTrash />}
      onClick={() => props.action.onClick(props.index)}
    />
  );
};

interface CancelActionProps {
  index: number;
  cancel: EditTableActionColumn['cancel'];
  onClick: () => void;
}

const CancelAction = (props: CancelActionProps) => {
  return (
    <TooltipAction
      index={props.index}
      action={props.cancel}
      icon={<BiRegularX />}
      onClick={props.onClick}
    />
  );
};

interface EditActionProps {
  index: number;
  editing: EditTableActionColumn['editing'];
  using?: boolean;
  onClick: () => void;
}

const EditAction = (props: EditActionProps) => {
  return (
    <TooltipAction
      index={props.index}
      action={props.editing}
      disabled={props.using}
      onClick={props.onClick}
      icon={<BiRegularEdit />}
    />
  );
};

interface SaveActionProps {
  index: number;
  edited: EditTableActionColumn['edited'];
  disabled?: boolean;
  value: ServerNode;
  beforeClick: () => void;
}

const SaveAction = (props: SaveActionProps) => {
  return (
    <TooltipAction
      index={props.index}
      action={props.edited}
      icon={<BiRegularCheck />}
      disabled={props.disabled}
      onClick={() => {
        props.beforeClick();
        props.edited.onClick(props.index, {
          ...props.value,
          isNewAdded: false,
          defaultEditing: false,
        });
      }}
    />
  );
};

const baseClassName = 'alley-table';

interface TableEditableRowProps {
  index: number;
  record: ServerNode;
  columns: TableColumn[];
  actions: TableProps['actions'];
  defaultEditing: boolean;
  onGlobalChange?: TableProps['onGlobalChange'];
}

const TableEditableRow = (props: TableEditableRowProps) => {
  const [editing, setEditing] = createSignal(props.defaultEditing);
  const [value, setValue] = createSignal(props.record);

  const disabled = createMemo(
    () =>
      Object.values(value()).filter((v) => typeof v !== 'boolean' && !v)
        .length > 0,
  );

  const switchServerNode = () => {
    props.onGlobalChange?.(props.index);
  };

  return (
    <tr class={`${baseClassName}-row`}>
      <For each={props.columns.map((c) => c.key)}>
        {(key, index) => (
          <TabelCell>
            <Show
              when={props.columns[index()].editable && editing()}
              fallback={
                <Show
                  when={
                    typeof props.record[key as keyof ServerNode] === 'boolean'
                  }
                  fallback={props.record[key as keyof ServerNode]}
                >
                  <SwitchServerNode
                    state={value()[key] as boolean}
                    disabled={disabled()}
                    onChange={switchServerNode}
                  />
                </Show>
              }
            >
              <InputCell
                index={index()}
                value={props.record[key] as string | number}
                onChange={(v) => setValue((prev) => ({ ...prev, [key]: v }))}
              />
            </Show>
          </TabelCell>
        )}
      </For>

      {props.actions ? (
        <TabelCell>
          <Show
            when={editing()}
            fallback={
              <LazySpace gap={4}>
                <EditAction
                  index={props.index}
                  editing={props.actions[0].editing}
                  using={props.record.using}
                  onClick={() => setEditing((prev) => !prev)}
                />

                <DeleteAction
                  index={props.index}
                  action={props.actions[1]}
                  using={props.record.using}
                />
              </LazySpace>
            }
          >
            <LazySpace gap={4}>
              <SaveAction
                index={props.index}
                edited={props.actions[0].edited}
                disabled={disabled() || deepEqual(value(), props.record)}
                beforeClick={() => setEditing((prev) => !prev)}
                value={value()}
              />

              <Show
                when={props.record.isNewAdded}
                fallback={
                  <CancelAction
                    index={props.index}
                    cancel={props.actions[0].cancel}
                    onClick={() => {
                      setEditing(false);
                      setValue(props.record);
                    }}
                  />
                }
              >
                <DeleteAction
                  index={props.index}
                  action={props.actions[1]}
                  using={props.record.using}
                />
              </Show>
            </LazySpace>
          </Show>
        </TabelCell>
      ) : undefined}
    </tr>
  );
};

export default TableEditableRow;

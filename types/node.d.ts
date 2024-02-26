interface ServerNode {
  name: string;
  addr: string;
  port: number;
  password: string;
  using?: boolean;
  defaultEditing?: boolean;
  isNewAdded?: boolean;
}

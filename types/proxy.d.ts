interface DaemonTrojanProcessState extends ServerNode {
  type: 'DAEMON';
}

interface InvalidTrojanProcessState {
  type: 'INVALID';
  pid: number;
}

interface OtherTrojanProcessState {
  type: 'OTHER';
  pid: number;
}

type TrojanProcessState =
  | DaemonTrojanProcessState
  | InvalidTrojanProcessState
  | OtherTrojanProcessState;

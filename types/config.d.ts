interface Configuration {
  local_addr: string;
  local_port: number;
  log_level: LogLevel;
  pac: string;
  nodes: ServerNode[];
}

type LogLevel = 'Trace' | 'Debug' | 'Info' | 'Warn' | 'Error';

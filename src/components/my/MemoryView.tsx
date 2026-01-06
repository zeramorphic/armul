import MemoryRow from './MemoryRow';
import { List } from 'react-window';

interface MemoryViewProps {
  mode: 'Disassemble' | 'Memory',
}

export function MemoryView(props: MemoryViewProps) {
  return <List rowComponent={MemoryRow} rowCount={1000} rowHeight={16} rowProps={{ mode: props.mode }} />;
}

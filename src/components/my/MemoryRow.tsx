import { invoke } from "@tauri-apps/api/core";
import "./MemoryRow.css"
import { useEffect, useState } from "react";

interface MemoryRowProps {
  addr: number
};

export default function (props: MemoryRowProps) {
  const [data, setData] = useState("");

  (async () => {
    const result: string = await invoke("line_at", { addr: props.addr });
    setData(result);
  })();

  return (
    <p className="MemoryRow">
      <span style={{
        color: `var(--very-muted-foreground)`
      }}>
        {("00000000" + props.addr.toString(16).toUpperCase()).slice(-8)}
      </span>
      &nbsp;
      {data}
    </p>
  )
}

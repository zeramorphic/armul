import "./MemoryRow.css"

interface MemoryRowProps {
    addr: number
};

export default function (props: MemoryRowProps) {
    return (
        <p className="MemoryRow">
            <span style={{
                color: `var(--very-muted-foreground)`
            }}>
                {("00000000" + props.addr.toString(16).toUpperCase()).slice(-8)}
            </span>
            &nbsp;
            Hello! I'm row {props.addr}!
        </p>
    )
}

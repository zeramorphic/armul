import { cn } from "@/lib/utils";

export default function Octagon({ enabled }: { enabled: boolean }) {
    return <svg
        width="15"
        viewBox="0 0 24 24"
        version="1.1"
        id="svg1"
        xmlns="http://www.w3.org/2000/svg"
        className={cn(
            "fill-(--color-breakpoint)",
            enabled ? "hover:opacity-50" : "opacity-0 hover:opacity-20",
            "mt-1")}>
        <defs id="defs1" />
        <g id="layer1">
            <path
                id="path3"
                d="M 26.320604,0 18.611478,18.611478 0,26.320604 -18.611478,18.611478 -26.320604,0 -18.611478,-18.611478 0,-26.320604 l 18.611478,7.709126 z"
                transform="matrix(-0.45591658,0.18884683,-0.18884683,-0.45591657,12,12)" />
        </g>
    </svg>;
}

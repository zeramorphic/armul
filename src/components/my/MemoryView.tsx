import { useVirtualizer } from '@tanstack/react-virtual';
import React from 'react';
import "./MemoryView.css";
import MemoryRow from './MemoryRow';
import { invoke } from '@tauri-apps/api/core';

export function TestApp() {
  // The scrollable element for your list
  const parentRef = React.useRef(null);

  // The lookup table from line numbers to their contents.
  const [cache, setCache] = React.useState(new Map<number, string>());

  function getCached(cache: Map<number, string>, addr: number): string {
    const value = cache.get(addr);
    if (value === undefined) {
      cache.set(addr, "");
      (async () => {
        const line: string = await invoke("line_at", { addr });
        setCache(cache => {
          cache.set(addr, line);
          return cache;
        });
      })();
      return "";
    } else {
      return value;
    }
  }

  // The virtualizer
  const count = 10000;
  const rowVirtualizer = useVirtualizer({
    count: count,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 16,
    initialOffset: 16 * count / 2 - 24,
    overscan: 10,
  });

  return (
    <>
      {/* The scrollable element for your list */}
      <div
        ref={parentRef}
        style={{
          overflow: 'auto', // Make it scroll!
        }}
        className="scrollable"
      >
        {/* The large inner element to hold all of the items */}
        <div
          style={{
            height: `${rowVirtualizer.getTotalSize()}px`,
            width: '100%',
            position: 'relative',
          }}
        >
          {/* Only the visible items in the virtualizer, manually positioned to be in view */}
          {rowVirtualizer.getVirtualItems().map((virtualItem) => {
            let rowOffset = virtualItem.index - count / 2;
            let addr = (rowOffset * 4 + 0x100000000) % 0x100000000;
            return (
              <div
                key={virtualItem.key}
                style={{
                  position: 'absolute',
                  top: 0,
                  left: 0,
                  width: '100%',
                  height: `${virtualItem.size}px`,
                  transform: `translateY(${virtualItem.start}px)`,
                }}
              >
                <MemoryRow addr={addr} contents={getCached(cache, addr)} />
              </div>
            );
          })}
        </div>
      </div>
    </>
  )
}

import { useVirtualizer } from '@tanstack/react-virtual';
import React from 'react';
import "./MemoryView.css";
import MemoryRow from './MemoryRow';

export function TestApp() {
  // The scrollable element for your list
  const parentRef = React.useRef(null)

  // The virtualizer
  const count = 10000;
  const rowVirtualizer = useVirtualizer({
    count: count,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 16,
    initialOffset: 16 * count / 2 - 24,
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
                <MemoryRow addr={addr} />
              </div>
            );
          })}
        </div>
      </div>
    </>
  )
}

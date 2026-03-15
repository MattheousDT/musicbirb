import { TrackItem } from "@/components/entities/TrackItem";
import { useMusicbirb } from "@/context/MusicbirbContext";
import React, { useEffect, useMemo, useRef } from "react";
import { VirtualizedList } from "react-native";

export function QueueList() {
  const { uiState, playIndex } = useMusicbirb();
  const listRef = useRef<VirtualizedList<any>>(null);

  const currentIdx = useMemo(
    () => uiState?.queuePosition ?? 0,
    [uiState?.queuePosition],
  );

  useEffect(() => {
    setTimeout(() => {
      if (
        listRef.current &&
        currentIdx >= 0 &&
        currentIdx < (uiState?.queue.length ?? 0)
      ) {
        listRef.current.scrollToIndex({
          index: currentIdx,
          animated: false,
          viewPosition: 0.25,
        });
      }
    }, 100);
  }, []);

  return (
    <VirtualizedList
      ref={listRef}
      keyExtractor={(item, idx) => `${item.id}-${idx}`}
      getItemCount={() => uiState?.queue.length ?? 0}
      getItem={(_, i) => uiState?.queue[i]}
      contentContainerStyle={{ paddingBlockEnd: 80, paddingInline: 24 }}
      onScrollToIndexFailed={() => {}}
      renderItem={({ item, index }) => {
        const isActive = index === currentIdx;
        return (
          <TrackItem
            isActive={isActive}
            trackNum={(index + 1).toString().padStart(2, "0")}
            title={item.title}
            artist={item.artist}
            onPress={() => playIndex(index)}
          />
        );
      }}
    />
  );
}

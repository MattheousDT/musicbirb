import React, { useEffect, useRef } from "react";
import { View, Text, FlatList, StyleSheet, Pressable } from "react-native";
import { useMusicbirb } from "../context/MusicbirbContext";

export function QueueList() {
  const { uiState, playIndex } = useMusicbirb();
  const listRef = useRef<FlatList>(null);
  const queue = uiState?.queue || [];
  const currentIdx = uiState?.queuePosition || 0;

  useEffect(() => {
    setTimeout(() => {
      if (listRef.current && currentIdx >= 0 && currentIdx < queue.length) {
        listRef.current.scrollToIndex({
          index: currentIdx,
          animated: false,
          viewPosition: 0.25,
        });
      }
    }, 100);
  }, []);

  return (
    <FlatList
      ref={listRef}
      data={queue}
      keyExtractor={(item, idx) => `${item.id}-${idx}`}
      showsVerticalScrollIndicator={false}
      contentContainerStyle={{ paddingBottom: 40 }}
      onScrollToIndexFailed={() => {}}
      renderItem={({ item, index }) => {
        const isActive = index === currentIdx;
        return (
          <Pressable
            onPress={() => playIndex(index)}
            style={[styles.row, isActive && styles.activeRow]}
          >
            <Text style={[styles.idx, isActive && styles.activeIdx]}>
              {(index + 1).toString().padStart(2, "0")}
            </Text>
            <View style={styles.info}>
              <Text
                numberOfLines={1}
                style={[styles.title, isActive && styles.activeText]}
              >
                {item.title}
              </Text>
              <Text
                numberOfLines={1}
                style={[styles.artist, isActive && styles.activeArtist]}
              >
                {item.artist}
              </Text>
            </View>
            {isActive && <View style={styles.playingIndicator} />}
          </Pressable>
        );
      }}
    />
  );
}

const styles = StyleSheet.create({
  row: {
    flexDirection: "row",
    alignItems: "center",
    paddingVertical: 18,
    paddingHorizontal: 16,
    borderRadius: 16,
    marginBottom: 4,
  },
  activeRow: { backgroundColor: "#f8fafc" },
  idx: { color: "#94a3b8", fontWeight: "800", fontSize: 14, width: 32 },
  activeIdx: { color: "#3b82f6" },
  info: { flex: 1 },
  title: { color: "#1e293b", fontSize: 16, fontWeight: "700", marginBottom: 2 },
  artist: { color: "#64748b", fontSize: 14, fontWeight: "600" },
  activeText: { color: "#0f172a" },
  activeArtist: { color: "#3b82f6" },
  playingIndicator: {
    width: 8,
    height: 8,
    borderRadius: 4,
    backgroundColor: "#3b82f6",
    marginLeft: 12,
  },
});

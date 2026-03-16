import { useApi } from "@/api";
import { TrackItem } from "@/components/entities/TrackItem";
import { InteractivePressable } from "@/components/ui/InteractivePressable";
import { useMusicbirb } from "@/context/MusicbirbContext";
import { getCoverUrl } from "@/utils/subsonic";
import { Ionicons } from "@expo/vector-icons";
import { Image } from "expo-image";
import { Stack, useLocalSearchParams } from "expo-router";
import React, { useMemo } from "react";
import { FlatList, RefreshControl, StyleSheet, Text, View } from "react-native";
import { useSafeAreaInsets } from "react-native-safe-area-context";

export default function PlaylistScreen() {
  const { id } = useLocalSearchParams<{ id: string }>();
  const insets = useSafeAreaInsets();
  const api = useApi();
  const { playPlaylist, playIndex, queuePlaylist, clearQueue } = useMusicbirb();

  const {
    data: playlist,
    isLoading,
    isRefetching,
    refetch,
  } = api.getPlaylistDetails.useQuery([id], {
    enabled: !!id,
  });

  const handlePlayPlaylist = async () => {
    if (!id) return;
    await playPlaylist(id);
  };

  const handlePlayTrack = async (idx: number) => {
    if (!id) return;
    clearQueue();
    await queuePlaylist(id);
    playIndex(idx);
  };

  const screenOptions = useMemo(
    () => (
      <Stack.Screen
        options={{
          title: isLoading ? "Loading" : (playlist?.name ?? "Unknown Playlist"),
          headerTitle: () => <View />,
          headerShown: true,
          headerTransparent: true,
          headerBackButtonDisplayMode: "minimal",
        }}
      />
    ),
    [isLoading, playlist?.name],
  );

  if (isLoading || !playlist) {
    return (
      <View style={styles.center}>
        {screenOptions}
        <Text>Loading...</Text>
      </View>
    );
  }

  return (
    <View style={styles.root}>
      {screenOptions}
      <FlatList
        data={playlist.songs}
        keyExtractor={(item, idx) => `${item.id}-${idx}`}
        showsVerticalScrollIndicator={false}
        contentContainerStyle={{
          paddingTop: insets.top,
          paddingBottom: insets.bottom + 120,
        }}
        refreshControl={
          <RefreshControl refreshing={isRefetching} onRefresh={refetch} />
        }
        ListHeaderComponent={
          <View style={styles.info}>
            <Image
              key={playlist.coverArt}
              source={
                getCoverUrl(playlist.coverArt)
                  ? { uri: getCoverUrl(playlist.coverArt)! }
                  : require("@assets/icon.png")
              }
              style={styles.cover}
              cachePolicy="memory-disk"
            />
            <Text style={styles.title}>{playlist.name}</Text>
            {playlist.owner && (
              <Text style={styles.owner}>Created by {playlist.owner}</Text>
            )}
            <Text style={styles.meta}>
              {playlist.songCount} tracks •{" "}
              {Math.floor(playlist.durationSecs / 60)} mins
            </Text>
            {playlist.comment && (
              <Text style={styles.comment} numberOfLines={3}>
                {playlist.comment}
              </Text>
            )}
            <InteractivePressable
              style={styles.playBtn}
              onPress={handlePlayPlaylist}
            >
              <Ionicons name="play" size={20} color="#fff" />
              <Text style={styles.playBtnText}>Play</Text>
            </InteractivePressable>
          </View>
        }
        renderItem={({ item: track, index: idx }) => (
          <TrackItem
            showArt
            imageUrl={getCoverUrl(track.coverArt)}
            title={track.title}
            artist={track.artist}
            durationSecs={track.durationSecs}
            onPress={() => handlePlayTrack(idx)}
            style={styles.trackItem}
          />
        )}
      />
    </View>
  );
}

const styles = StyleSheet.create({
  root: { flex: 1, backgroundColor: "#f8fafc" },
  info: {
    alignItems: "center",
    paddingHorizontal: 24,
    paddingTop: 64,
    paddingBottom: 32,
  },
  cover: {
    width: 240,
    height: 240,
    borderRadius: 32,
    marginBottom: 24,
    backgroundColor: "#e2e8f0",
    shadowColor: "#000",
    shadowOffset: { width: 0, height: 20 },
    shadowOpacity: 0.15,
    shadowRadius: 30,
    elevation: 20,
  },
  title: {
    fontSize: 28,
    fontWeight: "900",
    color: "#0f172a",
    textAlign: "center",
    marginBottom: 8,
    letterSpacing: -0.5,
  },
  owner: {
    fontSize: 18,
    fontWeight: "700",
    color: "#3b82f6",
    textAlign: "center",
    marginBottom: 8,
  },
  meta: {
    fontSize: 14,
    fontWeight: "600",
    color: "#64748b",
    textAlign: "center",
    marginBottom: 16,
  },
  comment: {
    fontSize: 14,
    color: "#475569",
    textAlign: "center",
    marginBottom: 24,
  },
  playBtn: {
    flexDirection: "row",
    alignItems: "center",
    backgroundColor: "#3b82f6",
    paddingHorizontal: 32,
    paddingVertical: 14,
    borderRadius: 100,
    gap: 8,
  },
  playBtnText: { color: "#fff", fontSize: 16, fontWeight: "800" },
  trackItem: { paddingHorizontal: 24 },
  center: { flex: 1, justifyContent: "center", alignItems: "center" },
});

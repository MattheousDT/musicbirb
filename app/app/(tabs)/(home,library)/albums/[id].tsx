import { useApi } from "@/api";
import { TrackItem } from "@/components/entities/TrackItem";
import { InteractivePressable } from "@/components/ui/InteractivePressable";
import { useMusicbirb } from "@/context/MusicbirbContext";
import { getCoverUrl } from "@/utils/subsonic";
import { Ionicons } from "@expo/vector-icons";
import { Image } from "expo-image";
import { Link, Stack, useLocalSearchParams, useRouter } from "expo-router";
import React, { useMemo } from "react";
import { FlatList, RefreshControl, StyleSheet, Text, View } from "react-native";
import { useSafeAreaInsets } from "react-native-safe-area-context";

export default function AlbumScreen() {
  const { id } = useLocalSearchParams<{ id: string }>();
  const router = useRouter();
  const insets = useSafeAreaInsets();
  const api = useApi();
  const { playAlbum, playIndex, queueAlbum, clearQueue } = useMusicbirb();

  const {
    data: album,
    isLoading,
    isRefetching,
    refetch,
  } = api.getAlbumDetails.useQuery([id], {
    enabled: !!id,
  });

  const handlePlayAlbum = async () => {
    if (!id) return;
    await playAlbum(id);
  };

  const handlePlayTrack = async (idx: number) => {
    if (!id) return;
    clearQueue();
    await queueAlbum(id);
    playIndex(idx);
  };

  const screenOptions = useMemo(
    () => (
      <Stack.Screen
        options={{
          title: isLoading ? "Loading" : (album?.title ?? "Unknown Album"),
          headerTitle: () => <View />,
          headerShown: true,
          headerTransparent: true,
          headerBackButtonDisplayMode: "minimal",
        }}
      />
    ),
    [isLoading, album?.title],
  );

  if (isLoading || !album) {
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
        data={album.songs}
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
            <Link.AppleZoomTarget>
              <Image
                key={album.coverArt}
                source={
                  getCoverUrl(album.coverArt)
                    ? { uri: getCoverUrl(album.coverArt)! }
                    : require("@assets/icon.png")
                }
                style={styles.cover}
                cachePolicy="memory-disk"
              />
            </Link.AppleZoomTarget>
            <Text style={styles.title}>{album.title}</Text>
            <InteractivePressable
              onPress={() =>
                album.artistId
                  ? router.push({
                      pathname: "/artists/[id]",
                      params: { id: album.artistId },
                    })
                  : null
              }
            >
              <Text style={styles.artist}>{album.artist}</Text>
            </InteractivePressable>
            <Text style={styles.meta}>
              {album.year ? `${album.year} • ` : ""}
              {album.songCount} tracks • {Math.floor(album.durationSecs / 60)}{" "}
              mins
            </Text>
            <InteractivePressable
              style={styles.playBtn}
              onPress={handlePlayAlbum}
            >
              <Ionicons name="play" size={20} color="#fff" />
              <Text style={styles.playBtnText}>Play Album</Text>
            </InteractivePressable>
          </View>
        }
        renderItem={({ item: track, index: idx }) => (
          <TrackItem
            trackNum={track.trackNum || idx + 1}
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
  artist: {
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

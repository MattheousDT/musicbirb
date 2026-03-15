import { useApi } from "@/api";
import { AlbumGridItem } from "@/components/entities/AlbumGridItem";
import { TrackItem } from "@/components/entities/TrackItem";
import { PaginatedList } from "@/components/layout/PaginatedList";
import { useMusicbirb } from "@/context/MusicbirbContext";
import { getCoverUrl } from "@/utils/subsonic";
import { Image } from "expo-image";
import { Stack, useLocalSearchParams } from "expo-router";
import React, { useMemo } from "react";
import {
  Dimensions,
  Platform,
  ScrollView,
  StyleSheet,
  Text,
  View,
} from "react-native";
import {
  SafeAreaView,
  useSafeAreaInsets,
} from "react-native-safe-area-context";

export default function ArtistScreen() {
  const { id } = useLocalSearchParams<{ id: string }>();
  const api = useApi();
  const { core, clearQueue, playIndex } = useMusicbirb();
  const insets = useSafeAreaInsets();

  const { data: artist, isLoading } = api.getArtistDetails.useQuery([id], {
    enabled: !!id,
  });

  const playTopSong = async (idx: number) => {
    if (!artist) return;
    clearQueue();
    for (const track of artist.topSongs) {
      await core?.queueTrack(track.id);
    }
    playIndex(idx);
  };

  const screenOptions = useMemo(
    () => (
      <Stack.Screen
        options={{
          title: isLoading ? "Loading" : (artist?.name ?? "Unknown Artist"),
          headerTitle: () => <View />,
          headerShown: true,
          headerTransparent: true,
          headerBackButtonDisplayMode: "minimal",
        }}
      />
    ),
    [isLoading, artist?.name],
  );

  if (isLoading || !artist) {
    return (
      <View style={styles.center}>
        {screenOptions}
        <Text>Loading...</Text>
      </View>
    );
  }

  const cleanBio =
    artist.biography?.replace(/<[^>]+>/g, "").trim() ||
    "No biography available.";

  return (
    <ScrollView
      style={styles.root}
      showsVerticalScrollIndicator={false}
      {...(Platform.OS === "ios"
        ? {
            contentInsetAdjustmentBehavior: "always",
            contentInset: { top: -(insets.top + 54) },
            bounces: false,
          }
        : {})}
    >
      {screenOptions}
      <SafeAreaView edges={["bottom", "left", "right"]}>
        <Image
          source={
            getCoverUrl(artist.coverArt)
              ? { uri: getCoverUrl(artist.coverArt)! }
              : require("@assets/icon.png")
          }
          style={[styles.heroImage, { height: 300 + insets.top }]}
          contentFit="cover"
          transition={500}
          cachePolicy="memory-disk"
        />
        <View style={styles.content}>
          <Text style={styles.title}>{artist.name}</Text>
          <Text style={styles.bio} numberOfLines={8}>
            {cleanBio}
          </Text>
        </View>

        {artist.topSongs && artist.topSongs.length > 0 && (
          <View style={styles.topSongs}>
            <Text
              style={[styles.sectionTitle, styles.content, { marginBottom: 0 }]}
            >
              Top Tracks
            </Text>
            <PaginatedList
              data={artist.topSongs}
              perPage={5}
              renderItem={({ item, index }) => (
                <TrackItem
                  key={item.id}
                  showArt
                  imageUrl={getCoverUrl(item.coverArt)}
                  title={item.title}
                  artist={item.artist}
                  durationSecs={item.durationSecs}
                  onPress={() => playTopSong(index)}
                  style={styles.trackRow}
                />
              )}
            />
          </View>
        )}

        <View style={styles.content}>
          <Text style={styles.sectionTitle}>Albums</Text>
          <View style={styles.albums}>
            {artist.albums.map((item) => (
              <AlbumGridItem
                key={item.id}
                id={item.id}
                title={item.title}
                artist={item.year?.toString() || "Album"}
                coverArt={item.coverArt}
                width={(Dimensions.get("screen").width - 48 - 16) / 2}
                useAppleZoom
              />
            ))}
          </View>
        </View>
      </SafeAreaView>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  root: { flex: 1, backgroundColor: "#f8fafc" },
  heroImage: { width: "100%", objectFit: "cover" },
  content: { paddingHorizontal: 24, marginTop: 24 },
  title: {
    fontSize: 32,
    fontWeight: "900",
    color: "#0f172a",
    letterSpacing: -1,
  },
  bio: { fontSize: 15, lineHeight: 22, color: "#475569", marginTop: 4 },
  sectionTitle: {
    fontSize: 20,
    fontWeight: "800",
    color: "#1e293b",
    marginBottom: 12,
    letterSpacing: -0.5,
  },
  topSongs: { gap: 12 },
  trackRow: {
    backgroundColor: "#fff",
    shadowColor: "#000",
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.05,
    shadowRadius: 8,
    elevation: 2,
  },
  albums: { display: "flex", flexDirection: "row", flexWrap: "wrap", gap: 16 },
  center: { flex: 1, justifyContent: "center", alignItems: "center" },
});

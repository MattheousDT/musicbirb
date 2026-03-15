import { useApi } from "@/api";
import { AlbumGridItem } from "@/components/entities/AlbumGridItem";
import { AlbumListItem } from "@/components/entities/AlbumListItem";
import { PlaceholderAlbumItem } from "@/components/entities/PlaceholderAlbumItem";
import { PlaylistItem } from "@/components/entities/PlaylistItem";
import { PaginatedList } from "@/components/layout/PaginatedList";
import { Stack } from "expo-router";
import React from "react";
import { FlatList, ScrollView, StyleSheet, Text } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";

export default function HomeScreen() {
  const api = useApi();

  const { data: lastPlayed, isLoading: lastPlayedLoading } =
    api.getLastPlayedAlbums.useQuery([], { throwOnError: true });

  const { data: recentlyAdded, isLoading: recentlyAddedLoading } =
    api.getRecentlyAddedAlbums.useQuery([]);

  const { data: newReleases } = api.getNewlyReleasedAlbums.useQuery([]);
  const { data: playlists } = api.getPlaylists.useQuery([]);

  return (
    <ScrollView
      style={styles.root}
      showsVerticalScrollIndicator={false}
      contentContainerStyle={styles.scroll}
    >
      <Stack.Screen options={{ title: "Home", headerShown: false }} />
      <SafeAreaView>
        <Text style={styles.header}>Home</Text>

        <Text style={styles.sectionTitle}>Last Played</Text>
        {lastPlayedLoading ? (
          <FlatList
            horizontal
            showsHorizontalScrollIndicator={false}
            data={Array(5)}
            keyExtractor={(_, i) => `lp-${i}`}
            renderItem={() => <PlaceholderAlbumItem width={140} />}
            contentContainerStyle={styles.carousel}
            snapToInterval={140 + 16}
            decelerationRate="fast"
            scrollEnabled={false}
          />
        ) : (
          <FlatList
            horizontal
            showsHorizontalScrollIndicator={false}
            data={lastPlayed}
            keyExtractor={(item) => `lp-${item.id}`}
            renderItem={({ item }) => (
              <AlbumGridItem
                id={item.id}
                title={item.title}
                artist={item.artist}
                coverArt={item.coverArt}
                useAppleZoom
              />
            )}
            contentContainerStyle={styles.carousel}
            snapToInterval={140 + 16}
            decelerationRate="fast"
          />
        )}

        <Text style={styles.sectionTitle}>Recently Added</Text>
        {recentlyAddedLoading ? (
          <FlatList
            horizontal
            showsHorizontalScrollIndicator={false}
            data={Array(5)}
            keyExtractor={(_, i) => `ra-${i}`}
            renderItem={() => <PlaceholderAlbumItem width={140} />}
            contentContainerStyle={styles.carousel}
            snapToInterval={140 + 16}
            decelerationRate="fast"
            scrollEnabled={false}
          />
        ) : (
          <FlatList
            horizontal
            showsHorizontalScrollIndicator={false}
            data={recentlyAdded}
            keyExtractor={(item) => `ra-${item.id}`}
            renderItem={({ item }) => (
              <AlbumGridItem
                id={item.id}
                title={item.title}
                artist={item.artist}
                coverArt={item.coverArt}
                useAppleZoom
              />
            )}
            contentContainerStyle={styles.carousel}
            snapToInterval={140 + 16}
            decelerationRate="fast"
          />
        )}

        <Text style={styles.sectionTitle}>New Releases</Text>
        <PaginatedList
          data={newReleases ?? []}
          perPage={5}
          keyExtractor={(_, idx) => `nr-${idx}`}
          renderItem={({ item }) => (
            <AlbumListItem
              key={item.id}
              id={item.id}
              title={item.title}
              artist={item.artist}
              coverArt={item.coverArt}
              songCount={item.songCount}
              year={item.year}
            />
          )}
        />

        <Text style={styles.sectionTitle}>Playlists</Text>
        <PaginatedList
          data={playlists ?? []}
          perPage={5}
          keyExtractor={(_, idx) => `pl-${idx}`}
          renderItem={({ item }) => (
            <PlaylistItem
              key={item.id}
              id={item.id}
              name={item.name}
              songCount={item.songCount}
              durationSecs={item.durationSecs}
              coverArt={item.coverArt}
            />
          )}
        />
      </SafeAreaView>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  root: { flex: 1, backgroundColor: "#ffffff" },
  scroll: { paddingVertical: 16 },
  header: {
    fontSize: 36,
    fontWeight: "900",
    color: "#0f172a",
    paddingHorizontal: 24,
    letterSpacing: -1,
  },
  sectionTitle: {
    fontSize: 22,
    fontWeight: "800",
    color: "#1e293b",
    paddingHorizontal: 24,
    marginBottom: 16,
    marginTop: 32,
    letterSpacing: -0.5,
  },
  carousel: { paddingHorizontal: 24, gap: 16 },
});

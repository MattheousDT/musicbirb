import { Image } from "expo-image";
import { Link } from "expo-router";
import React from "react";
import { StyleSheet, View } from "react-native";
import { getCoverUrl } from "@/utils/subsonic";
import { ListItem } from "@/components/ui/ListItem";

interface PlaylistItemProps {
  id: string;
  name: string;
  songCount: number;
  durationSecs: number;
  coverArt?: string | null;
}

export function PlaylistItem({
  id,
  name,
  songCount,
  durationSecs,
  coverArt,
}: PlaylistItemProps) {
  const imageUrl = getCoverUrl(coverArt) ?? undefined;
  const subtitle = `${songCount} tracks • ${Math.floor(durationSecs / 60)} mins`;

  return (
    <Link href={{ pathname: "/playlists/[id]", params: { id } }} asChild>
      <ListItem style={styles.container}>
        <ListItem.Leading>
          {imageUrl ? (
            <Image
              source={{ uri: imageUrl }}
              style={styles.image}
              cachePolicy="memory-disk"
            />
          ) : (
            <View style={styles.image} />
          )}
        </ListItem.Leading>
        <ListItem.Content>
          <ListItem.Title>{name}</ListItem.Title>
          <ListItem.Subtitle>{subtitle}</ListItem.Subtitle>
        </ListItem.Content>
      </ListItem>
    </Link>
  );
}

const styles = StyleSheet.create({
  container: {
    backgroundColor: "#f8fafc",
    paddingVertical: 10,
    paddingHorizontal: 10,
  },
  image: {
    width: 48,
    height: 48,
    borderRadius: 12,
    backgroundColor: "#e2e8f0",
  },
});

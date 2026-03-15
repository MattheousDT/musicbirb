import { Image } from "expo-image";
import { Link } from "expo-router";
import React from "react";
import { StyleSheet, View } from "react-native";
import { getCoverUrl } from "@/utils/subsonic";
import { ListItem } from "@/components/ui/ListItem";

interface AlbumListItemProps {
  id: string;
  title: string;
  artist: string;
  coverArt?: string | null;
  useAppleZoom?: boolean;
  songCount?: number;
  year?: number;
}

export function AlbumListItem({
  id,
  title,
  artist,
  coverArt,
  useAppleZoom,
  songCount,
  year,
}: AlbumListItemProps) {
  const imageUrl = getCoverUrl(coverArt) ?? undefined;

  const imageComponent =
    useAppleZoom && imageUrl ? (
      <Link.AppleZoom>
        <Image
          source={{ uri: imageUrl }}
          style={styles.image}
          cachePolicy="memory-disk"
        />
      </Link.AppleZoom>
    ) : undefined;

  const subtitle = [artist, year, songCount ? `${songCount} tracks` : null]
    .filter(Boolean)
    .join(" • ");

  return (
    <Link href={{ pathname: "/albums/[id]", params: { id } }} asChild>
      <ListItem style={styles.container}>
        <ListItem.Leading>
          {imageComponent ? (
            imageComponent
          ) : imageUrl ? (
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
          <ListItem.Title>{title}</ListItem.Title>
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

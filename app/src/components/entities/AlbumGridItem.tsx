import { Image } from "expo-image";
import { Link } from "expo-router";
import React from "react";
import { View } from "react-native";
import { getCoverUrl } from "@/utils/subsonic";
import { GridItem } from "@/components/ui/GridItem";

interface AlbumGridItemProps {
  id: string;
  title: string;
  artist: string;
  coverArt?: string | null;
  width?: number;
  useAppleZoom?: boolean;
}

export function AlbumGridItem({
  id,
  title,
  artist,
  coverArt,
  width = 140,
  useAppleZoom,
}: AlbumGridItemProps) {
  const imageUrl = getCoverUrl(coverArt) ?? undefined;

  const imageStyles = {
    width,
    height: width,
    borderRadius: 20,
    marginBottom: 8,
    backgroundColor: "#f8fafc",
  };

  const renderImage = () => {
    if (useAppleZoom && imageUrl) {
      return (
        <Link.AppleZoom>
          <Image
            source={{ uri: imageUrl }}
            style={imageStyles}
            cachePolicy="memory-disk"
          />
        </Link.AppleZoom>
      );
    }
    if (imageUrl) {
      return (
        <Image
          source={{ uri: imageUrl }}
          style={imageStyles}
          cachePolicy="memory-disk"
        />
      );
    }
    return <View style={imageStyles} />;
  };

  return (
    <Link href={{ pathname: "/albums/[id]", params: { id } }} asChild>
      <GridItem style={{ width }}>
        {renderImage()}
        <GridItem.Title>{title}</GridItem.Title>
        <GridItem.Subtitle>{artist}</GridItem.Subtitle>
      </GridItem>
    </Link>
  );
}

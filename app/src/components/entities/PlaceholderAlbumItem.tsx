import React from "react";
import { View } from "react-native";
import { GridItem } from "@/components/ui/GridItem";

export function PlaceholderAlbumItem({ width = 140 }: { width?: number }) {
  return (
    <GridItem style={{ width }} disabled>
      <View
        style={{
          width,
          height: width,
          borderRadius: 20,
          marginBottom: 8,
          backgroundColor: "#f8fafc",
        }}
      />
      <GridItem.Title> </GridItem.Title>
      <GridItem.Subtitle> </GridItem.Subtitle>
    </GridItem>
  );
}

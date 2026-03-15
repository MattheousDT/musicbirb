import { Image } from "expo-image";
import React from "react";
import {
  PressableProps,
  StyleProp,
  StyleSheet,
  Text,
  View,
  ViewStyle,
} from "react-native";
import { ListItem } from "@/components/ui/ListItem";
import { formatDuration } from "@/utils/time";

export interface TrackItemProps extends PressableProps {
  title: string;
  artist?: string;
  durationSecs?: number;
  trackNum?: number | string;
  imageUrl?: string | null;
  isActive?: boolean;
  showArt?: boolean;
  style?: StyleProp<ViewStyle>;
}

export const TrackItem = React.forwardRef<View, TrackItemProps>(
  (
    {
      title,
      artist,
      durationSecs,
      trackNum,
      imageUrl,
      isActive,
      showArt,
      style,
      ...rest
    },
    ref,
  ) => {
    return (
      <ListItem
        ref={ref}
        style={[isActive && styles.activeContainer, style]}
        {...rest}
      >
        <ListItem.Leading style={!showArt && styles.trackNumContainer}>
          {showArt ? (
            <Image
              source={
                imageUrl ? { uri: imageUrl } : require("@assets/icon.png")
              }
              style={styles.art}
              cachePolicy="memory-disk"
            />
          ) : (
            <Text style={[styles.trackNum, isActive && styles.activeText]}>
              {trackNum}
            </Text>
          )}
        </ListItem.Leading>

        <ListItem.Content>
          <ListItem.Title style={isActive && styles.activeTitle}>
            {title}
          </ListItem.Title>
          {artist ? (
            <ListItem.Subtitle style={isActive && styles.activeArtist}>
              {artist}
            </ListItem.Subtitle>
          ) : null}
        </ListItem.Content>

        <ListItem.Trailing>
          {durationSecs !== undefined && (
            <Text style={styles.duration}>{formatDuration(durationSecs)}</Text>
          )}
          {isActive && <View style={styles.playingIndicator} />}
        </ListItem.Trailing>
      </ListItem>
    );
  },
);

TrackItem.displayName = "TrackItem";

const styles = StyleSheet.create({
  activeContainer: { backgroundColor: "#f8fafc" },
  trackNumContainer: { width: 32, marginRight: 12 },
  trackNum: {
    fontSize: 15,
    fontWeight: "700",
    color: "#94a3b8",
    textAlign: "center",
  },
  art: { width: 48, height: 48, borderRadius: 12, backgroundColor: "#e2e8f0" },
  activeTitle: { color: "#0f172a" },
  activeArtist: { color: "#3b82f6" },
  duration: { fontSize: 13, fontWeight: "600", color: "#94a3b8" },
  activeText: { color: "#3b82f6" },
  playingIndicator: {
    width: 8,
    height: 8,
    borderRadius: 4,
    backgroundColor: "#3b82f6",
    marginLeft: 12,
  },
});

import React, { useMemo, useRef, useState } from "react";
import {
	LayoutRectangle,
	PanResponder,
	StyleSheet,
	Text,
	View,
} from "react-native";
import { useMusicbirb } from "../context/MusicbirbContext";

export function ProgressBar() {
  const { playlistStatus, uiState, seek } = useMusicbirb();

  const currentTrack = useMemo(
    () => uiState?.queue[uiState.queuePosition],
    [uiState?.queue, uiState?.queuePosition],
  );

  const trackLayout = useRef<LayoutRectangle | null>(null);

  const [isSeeking, setIsSeeking] = useState(false);
  const [seekX, setSeekX] = useState(0);

  const duration = currentTrack?.durationSecs || 0;
  const actualProgress =
    duration > 0 ? ((playlistStatus?.currentTime || 0) / duration) * 100 : 0;

  let displayProgress = actualProgress;
  if (isSeeking && trackLayout.current) {
    const relativeX = Math.max(
      0,
      Math.min(trackLayout.current.width, seekX - trackLayout.current.x),
    );
    displayProgress = (relativeX / trackLayout.current.width) * 100;
  }

  const scrobbleMark = uiState?.scrobbleMarkPos || 0;
  const scrobbleProgress =
    duration > 0 && scrobbleMark > 0 ? (scrobbleMark / duration) * 100 : 0;

  const formatTime = (s: number) =>
    `${Math.floor(s / 60)}:${Math.floor(s % 60)
      .toString()
      .padStart(2, "0")}`;

  const panResponder = useRef(
    PanResponder.create({
      onStartShouldSetPanResponder: () => true,
      onMoveShouldSetPanResponder: () => true,
      onPanResponderGrant: (evt) => {
        setIsSeeking(true);
        setSeekX(evt.nativeEvent.pageX);
      },
      onPanResponderMove: (evt) => {
        setSeekX(evt.nativeEvent.pageX);
      },
      onPanResponderRelease: (evt) => {
        if (!trackLayout.current || duration <= 0) {
          setIsSeeking(false);
          return;
        }
        const relativeX = Math.max(
          0,
          Math.min(
            trackLayout.current.width,
            evt.nativeEvent.pageX - trackLayout.current.x,
          ),
        );
        const targetTime = (relativeX / trackLayout.current.width) * duration;
        seek(targetTime);
        setIsSeeking(false);
      },
    }),
  ).current;

  return (
    <View style={styles.container}>
      <View
        style={styles.track}
        onLayout={(e) => {
          trackLayout.current = e.nativeEvent.layout;
        }}
        {...panResponder.panHandlers}
      >
        {scrobbleProgress > 0 && (
          <View
            style={[styles.scrobbleMark, { left: `${scrobbleProgress}%` }]}
          />
        )}
        <View style={[styles.fill, { width: `${displayProgress}%` }]} />
      </View>
      <View style={styles.labels}>
        <Text style={styles.time}>
          {formatTime(
            isSeeking
              ? (displayProgress / 100) * duration
              : playlistStatus?.currentTime || 0,
          )}
        </Text>
        <Text style={styles.time}>{formatTime(duration)}</Text>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: { width: "100%" },
  track: {
    height: 12,
    width: "100%",
    backgroundColor: "#f1f5f9",
    borderRadius: 6,
    overflow: "hidden",
    justifyContent: "center",
  },
  fill: { height: "100%", backgroundColor: "#0f172a", borderRadius: 6 },
  scrobbleMark: {
    position: "absolute",
    top: 0,
    bottom: 0,
    width: 2,
    backgroundColor: "#3b82f6",
    zIndex: 10,
    opacity: 0.8,
  },
  labels: {
    flexDirection: "row",
    justifyContent: "space-between",
    marginTop: 12,
  },
  time: { color: "#94a3b8", fontSize: 12, fontWeight: "800" },
});

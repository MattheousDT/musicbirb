import { useMusicbirb } from "@/context/MusicbirbContext";
import { formatDuration } from "@/utils/time";
import React, { useEffect, useMemo, useRef, useState } from "react";
import { Animated, PanResponder, StyleSheet, Text, View } from "react-native";

export function ProgressBar() {
  const { playlistStatus, uiState, seek } = useMusicbirb();

  const currentTrack = useMemo(
    () => uiState?.queue[uiState.queuePosition],
    [uiState?.queue, uiState?.queuePosition],
  );

  const duration = useMemo(
    () => currentTrack?.durationSecs || 0,
    [currentTrack?.durationSecs],
  );

  const durationRef = useRef(duration);
  durationRef.current = duration;

  const seekRef = useRef(seek);
  seekRef.current = seek;

  const trackWidthRef = useRef(0);
  const initialXRef = useRef(0);

  const [isSeeking, setIsSeeking] = useState(false);
  const [seekProgress, setSeekProgress] = useState(0);
  const [targetSeekTime, setTargetSeekTime] = useState<number | null>(null);

  // Track the most up-to-date player time for relative offset calculations
  const currentTimeRef = useRef(playlistStatus?.currentTime || 0);

  // Reset optimistic state if the track changes
  useEffect(() => {
    setTargetSeekTime(null);
  }, [duration]);

  // Only update baseline if we are NOT optimistically waiting for a seek to complete.
  // This prevents stale times from ruining rapid successive relative seeks.
  useEffect(() => {
    if (targetSeekTime === null) {
      currentTimeRef.current = playlistStatus?.currentTime || 0;
    }
  }, [playlistStatus?.currentTime, targetSeekTime]);

  // Effect to clear optimistic seek state when the player catches up
  useEffect(() => {
    if (targetSeekTime !== null && playlistStatus?.currentTime !== undefined) {
      if (Math.abs(playlistStatus.currentTime - targetSeekTime) < 2) {
        setTargetSeekTime(null);
      }
    }
  }, [playlistStatus?.currentTime, targetSeekTime]);

  // Fallback to clear targetSeekTime in case of failure or end of track
  useEffect(() => {
    if (targetSeekTime !== null) {
      const t = setTimeout(() => setTargetSeekTime(null), 3000);
      return () => clearTimeout(t);
    }
  }, [targetSeekTime]);

  // Animation for the tooltip and thumb bouncing in/out
  const scaleAnim = useRef(new Animated.Value(0)).current;

  const panResponder = useRef(
    PanResponder.create({
      onStartShouldSetPanResponder: () => true,
      onMoveShouldSetPanResponder: () => true,
      onPanResponderGrant: (evt) => {
        setTargetSeekTime(null); // Clear optimistic state if user starts seeking again
        initialXRef.current = evt.nativeEvent.locationX;
        const w = trackWidthRef.current;
        const prog =
          w > 0 ? Math.max(0, Math.min(1, initialXRef.current / w)) : 0;

        setIsSeeking(true);
        setSeekProgress(prog);

        Animated.spring(scaleAnim, {
          toValue: 1,
          friction: 6,
          tension: 100,
          useNativeDriver: true,
        }).start();
      },
      onPanResponderMove: (evt, gestureState) => {
        const currentX = initialXRef.current + gestureState.dx;
        const w = trackWidthRef.current;
        const prog = w > 0 ? Math.max(0, Math.min(1, currentX / w)) : 0;
        setSeekProgress(prog);
      },
      onPanResponderRelease: (evt, gestureState) => {
        const currentX = initialXRef.current + gestureState.dx;
        const w = trackWidthRef.current;
        const prog = w > 0 ? Math.max(0, Math.min(1, currentX / w)) : 0;

        const targetTime = prog * durationRef.current;
        setTargetSeekTime(targetTime);
        setIsSeeking(false);

        Animated.spring(scaleAnim, {
          toValue: 0,
          friction: 7,
          tension: 80,
          useNativeDriver: true,
        }).start();

        if (durationRef.current > 0) {
          const relativeOffset = targetTime - currentTimeRef.current;
          seekRef.current(relativeOffset);
          // Update the ref immediately so subsequent rapid seeks are relative to this new target!
          currentTimeRef.current = targetTime;
        }
      },
      onPanResponderTerminate: () => {
        setIsSeeking(false);
        setTargetSeekTime(null);
        Animated.spring(scaleAnim, {
          toValue: 0,
          friction: 7,
          tension: 80,
          useNativeDriver: true,
        }).start();
      },
    }),
  ).current;

  const actualProgress =
    duration > 0 ? (playlistStatus?.currentTime || 0) / duration : 0;

  let displayProgress = actualProgress;
  if (isSeeking) {
    displayProgress = seekProgress;
  } else if (targetSeekTime !== null && duration > 0) {
    displayProgress = targetSeekTime / duration;
  }

  const displayProgressClamped = Math.max(0, Math.min(1, displayProgress));
  const currentDisplayTime = displayProgressClamped * duration;

  const scrobbleMark = uiState?.scrobbleMarkPos || 0;
  const scrobbleProgress =
    duration > 0 && scrobbleMark > 0 ? scrobbleMark / duration : 0;

  return (
    <View style={styles.container}>
      <View
        style={styles.touchableArea}
        {...panResponder.panHandlers}
        collapsable={false}
      >
        <View
          style={styles.track}
          onLayout={(e) => {
            trackWidthRef.current = e.nativeEvent.layout.width;
          }}
        >
          {scrobbleProgress > 0 && (
            <View
              style={[
                styles.scrobbleMark,
                { left: `${scrobbleProgress * 100}%` },
              ]}
            />
          )}
          <View
            style={[styles.fill, { width: `${displayProgressClamped * 100}%` }]}
          />
        </View>

        <Animated.View
          pointerEvents="none"
          style={[
            styles.thumbContainer,
            {
              left: `${displayProgressClamped * 100}%`,
              transform: [{ scale: scaleAnim }],
            },
          ]}
        >
          <View style={styles.tooltipContainer}>
            <View style={styles.tooltipBubble}>
              <Text style={styles.tooltipText}>
                {formatDuration(currentDisplayTime)}
              </Text>
            </View>
            <View style={styles.tooltipArrow} />
          </View>

          <View style={styles.thumb} />
        </Animated.View>
      </View>

      <View style={styles.labels}>
        <Text style={styles.time}>{formatDuration(currentDisplayTime)}</Text>
        <Text style={styles.time}>{formatDuration(duration)}</Text>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: { width: "100%" },
  touchableArea: {
    paddingVertical: 24,
    justifyContent: "center",
    position: "relative",
    marginBottom: -12,
    marginTop: -12,
  },
  track: {
    height: 12,
    width: "100%",
    backgroundColor: "#f1f5f9",
    borderRadius: 6,
    overflow: "hidden",
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
  thumbContainer: {
    position: "absolute",
    top: 30,
    zIndex: 20,
  },
  thumb: {
    position: "absolute",
    width: 24,
    height: 24,
    borderRadius: 12,
    backgroundColor: "#0f172a",
    borderWidth: 4,
    borderColor: "#ffffff",
    shadowColor: "#000",
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.3,
    shadowRadius: 4,
    elevation: 5,
    marginTop: -12,
    marginLeft: -12,
  },
  tooltipContainer: {
    position: "absolute",
    bottom: 20,
    width: 100,
    marginLeft: -50,
    alignItems: "center",
  },
  tooltipBubble: {
    backgroundColor: "#0f172a",
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 8,
    alignItems: "center",
    shadowColor: "#000",
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.2,
    shadowRadius: 4,
    elevation: 4,
  },
  tooltipText: {
    color: "#ffffff",
    fontSize: 14,
    fontWeight: "800",
    fontVariant: ["tabular-nums"],
  },
  tooltipArrow: {
    width: 0,
    height: 0,
    borderLeftWidth: 6,
    borderRightWidth: 6,
    borderTopWidth: 6,
    borderLeftColor: "transparent",
    borderRightColor: "transparent",
    borderTopColor: "#0f172a",
  },
  labels: {
    flexDirection: "row",
    justifyContent: "space-between",
    marginTop: 8,
  },
  time: {
    color: "#94a3b8",
    fontSize: 12,
    fontWeight: "800",
    fontVariant: ["tabular-nums"],
  },
});

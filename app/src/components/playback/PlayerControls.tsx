import { InteractivePressable } from "@/components/ui/InteractivePressable";
import { useMusicbirb } from "@/context/MusicbirbContext";
import { Ionicons } from "@expo/vector-icons";
import React from "react";
import { ActivityIndicator, StyleSheet, View } from "react-native";

export function PlayerControls() {
  const { playlistStatus, togglePause, next, prev, isBuffering } =
    useMusicbirb();
  const isPlaying = playlistStatus?.playing ?? false;

  return (
    <View style={styles.container}>
      <InteractivePressable onPress={prev} style={styles.skipBtn}>
        <Ionicons name="play-back" size={32} color="#0f172a" />
      </InteractivePressable>

      <InteractivePressable
        onPress={togglePause}
        style={styles.playBtn}
        disabled={isBuffering}
      >
        {isBuffering ? (
          <ActivityIndicator color="#fff" size="large" />
        ) : (
          <Ionicons
            name={isPlaying ? "pause" : "play"}
            size={48}
            color="#fff"
            style={!isPlaying && { marginLeft: 6 }}
          />
        )}
      </InteractivePressable>

      <InteractivePressable onPress={next} style={styles.skipBtn}>
        <Ionicons name="play-forward" size={32} color="#0f172a" />
      </InteractivePressable>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flexDirection: "row",
    alignItems: "center",
    justifyContent: "center",
    width: "100%",
    marginTop: 48,
  },
  skipBtn: { padding: 16 },
  playBtn: {
    width: 96,
    height: 96,
    borderRadius: 48,
    backgroundColor: "#0f172a",
    justifyContent: "center",
    alignItems: "center",
    marginHorizontal: 24,
    shadowColor: "#000",
    shadowOffset: { width: 0, height: 10 },
    shadowOpacity: 0.2,
    shadowRadius: 15,
    elevation: 10,
  },
});

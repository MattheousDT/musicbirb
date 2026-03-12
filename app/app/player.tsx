import { PlayerControls } from "@/components/PlayerControls";
import { ProgressBar } from "@/components/ProgressBar";
import { QueueList } from "@/components/QueueList";
import { useMusicbirb } from "@/context/MusicbirbContext";
import { Ionicons } from "@expo/vector-icons";
import { Image } from "expo-image";
import React, { useMemo, useState } from "react";
import {
	Animated,
	Modal,
	Platform,
	Pressable,
	StyleSheet,
	Text,
	View,
} from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";

export default function PlayerScreen() {
  const { uiState } = useMusicbirb();
  const [isQueueOpen, setQueueOpen] = useState(false);
  const [scaleAnim] = useState(new Animated.Value(1));

  const currentTrack = useMemo(
    () => uiState?.queue[uiState.queuePosition],
    [uiState?.queue, uiState?.queuePosition],
  );

  const coverUrl = useMemo(
    () =>
      currentTrack?.coverArtId
        ? `${process.env.EXPO_PUBLIC_SUBSONIC_URL}/rest/getCoverArt?id=${currentTrack.coverArtId}&u=${process.env.EXPO_PUBLIC_SUBSONIC_USER}&p=${process.env.EXPO_PUBLIC_SUBSONIC_PASS}&v=1.16.1&c=musicbirb`
        : null,
    [currentTrack?.coverArtId],
  );

  const handleOpenQueue = () => {
    setQueueOpen(true);
    Animated.timing(scaleAnim, {
      toValue: 0.94,
      duration: 400,
      useNativeDriver: true,
    }).start();
  };

  const handleCloseQueue = () => {
    setQueueOpen(false);
    Animated.timing(scaleAnim, {
      toValue: 1,
      duration: 300,
      useNativeDriver: true,
    }).start();
  };

  return (
    <View style={styles.root}>
      <Animated.View
        style={[styles.container, { transform: [{ scale: scaleAnim }] }]}
      >
        <SafeAreaView style={{ flex: 1 }}>
          <View style={styles.header}>
            <Pressable onPress={handleOpenQueue} style={styles.queueBtn}>
              <Ionicons name="list" size={24} color="#1e293b" />
            </Pressable>
          </View>

          <View style={styles.content}>
            <View style={styles.artContainer}>
              <View style={styles.shadow}>
                <Image
                  source={
                    coverUrl ? { uri: coverUrl } : require("../assets/icon.png")
                  }
                  style={styles.artwork}
                  contentFit="cover"
                  transition={500}
                />
              </View>
            </View>

            <View style={styles.meta}>
              <Text numberOfLines={1} style={styles.title}>
                {currentTrack?.title || "Waiting..."}
              </Text>
              <Text numberOfLines={1} style={styles.artist}>
                {currentTrack?.artist || "Birb"}
              </Text>
            </View>

            <ProgressBar />
            <PlayerControls />
          </View>
        </SafeAreaView>
      </Animated.View>

      <Modal
        visible={isQueueOpen}
        animationType="slide"
        presentationStyle={Platform.OS === "ios" ? "pageSheet" : "fullScreen"}
        onRequestClose={handleCloseQueue}
      >
        <View style={styles.modal}>
          <SafeAreaView>
            {Platform.OS === "ios" && <View style={styles.handle} />}
            <View style={styles.modalHeader}>
              <Text style={styles.modalTitle}>Queue</Text>
              <Pressable onPress={handleCloseQueue} style={styles.closeBtn}>
                <Ionicons name="close-circle" size={32} color="#cbd5e1" />
              </Pressable>
            </View>
            <QueueList />
          </SafeAreaView>
        </View>
      </Modal>
    </View>
  );
}

const styles = StyleSheet.create({
  root: { flex: 1, backgroundColor: "#f8fafc" },
  container: { flex: 1, backgroundColor: "#f8fafc", overflow: "hidden" },
  header: {
    flexDirection: "row",
    justifyContent: "flex-end",
    alignItems: "center",
    paddingHorizontal: 24,
    paddingVertical: 16,
  },
  pill: {
    backgroundColor: "#e2e8f0",
    paddingHorizontal: 16,
    paddingVertical: 6,
    borderRadius: 20,
  },
  pillText: {
    fontSize: 12,
    fontWeight: "800",
    color: "#475569",
    letterSpacing: 0.5,
    textTransform: "uppercase",
  },
  queueBtn: {
    width: 44,
    height: 44,
    borderRadius: 22,
    backgroundColor: "#ffffff",
    justifyContent: "center",
    alignItems: "center",
    shadowColor: "#000",
    shadowOffset: { width: 0, height: 4 },
    shadowOpacity: 0.05,
    shadowRadius: 10,
    elevation: 2,
  },
  content: { flex: 1, paddingHorizontal: 40, justifyContent: "center" },
  artContainer: { width: "100%", aspectRatio: 1, marginBottom: 48 },
  shadow: {
    flex: 1,
    borderRadius: 32,
    backgroundColor: "#fff",
    shadowColor: "#000",
    shadowOffset: { width: 0, height: 20 },
    shadowOpacity: 0.15,
    shadowRadius: 30,
    elevation: 20,
  },
  artwork: { width: "100%", height: "100%", borderRadius: 32 },
  meta: { width: "100%", marginBottom: 32, alignItems: "center" },
  title: {
    color: "#0f172a",
    fontSize: 28,
    fontWeight: "900",
    letterSpacing: -0.5,
    marginBottom: 6,
    textAlign: "center",
  },
  artist: {
    color: "#3b82f6",
    fontSize: 18,
    fontWeight: "700",
    textAlign: "center",
  },
  modal: {
    flex: 1,
    backgroundColor: "#ffffff",
    borderTopLeftRadius: 24,
    borderTopRightRadius: 24,
    paddingHorizontal: 24,
    paddingTop: 12,
  },
  handle: {
    width: 40,
    height: 5,
    backgroundColor: "#e2e8f0",
    borderRadius: 2.5,
    alignSelf: "center",
    marginBottom: 20,
  },
  modalHeader: {
    flexDirection: "row",
    justifyContent: "space-between",
    alignItems: "center",
    marginBottom: 20,
  },
  modalTitle: { fontSize: 24, fontWeight: "900", color: "#0f172a" },
  closeBtn: { padding: 4 },
});

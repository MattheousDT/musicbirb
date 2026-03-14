import { Ionicons } from "@expo/vector-icons";
import { Image } from "expo-image";
import { useRouter } from "expo-router";
import React, { useMemo } from "react";
import {
	ActivityIndicator,
	Pressable,
	StyleSheet,
	Text,
	View,
} from "react-native";
import { useSafeAreaInsets } from "react-native-safe-area-context";
import { useMusicbirb } from "../context/MusicbirbContext";

export function CurrentlyPlayingBar() {
	const { uiState, playlistStatus, togglePause, next, isBuffering } =
		useMusicbirb();
	const router = useRouter();
	const insets = useSafeAreaInsets();

	const currentTrack = useMemo(
		() => uiState?.queue[uiState.queuePosition],
		[uiState?.queue, uiState?.queuePosition],
	);

	const coverUrl = useMemo(
		() =>
			currentTrack?.coverArt
				? `${process.env.EXPO_PUBLIC_SUBSONIC_URL}/rest/getCoverArt?id=${currentTrack.coverArt}&u=${process.env.EXPO_PUBLIC_SUBSONIC_USER}&p=${process.env.EXPO_PUBLIC_SUBSONIC_PASS}&v=1.16.1&c=musicbirb`
				: null,
		[currentTrack?.coverArt],
	);

	if (!currentTrack) return null;

	return (
		<View style={[styles.wrapper, { bottom: 64 + insets.bottom }]}>
			<Pressable
				style={styles.container}
				onPress={() => router.push("/player")}
			>
				<Image
					source={
						coverUrl ? { uri: coverUrl } : require("../../assets/icon.png")
					}
					style={styles.art}
				/>
				<View style={styles.info}>
					<Text numberOfLines={1} style={styles.title}>
						{currentTrack.title}
					</Text>
					<Text numberOfLines={1} style={styles.artist}>
						{currentTrack.artist}
					</Text>
				</View>
				<View style={styles.controls}>
					<Pressable
						style={styles.btn}
						onPress={togglePause}
						disabled={isBuffering}
					>
						{isBuffering ? (
							<ActivityIndicator color="#0f172a" />
						) : (
							<Ionicons
								name={playlistStatus?.playing ? "pause" : "play"}
								size={26}
								color="#0f172a"
							/>
						)}
					</Pressable>
					<Pressable style={styles.btn} onPress={next}>
						<Ionicons name="play-forward" size={26} color="#0f172a" />
					</Pressable>
				</View>
			</Pressable>
		</View>
	);
}

const styles = StyleSheet.create({
	wrapper: {
		position: "absolute",
		left: 16,
		right: 16,
		zIndex: 100,
	},
	container: {
		flexDirection: "row",
		alignItems: "center",
		backgroundColor: "rgba(255, 255, 255, 0.98)",
		padding: 8,
		borderRadius: 14,
		shadowColor: "#000",
		shadowOffset: { width: 0, height: 8 },
		shadowOpacity: 0.15,
		shadowRadius: 12,
		elevation: 8,
		borderWidth: 1,
		borderColor: "#f1f5f9",
	},
	art: { width: 48, height: 48, borderRadius: 10, backgroundColor: "#e2e8f0" },
	info: { flex: 1, marginLeft: 14, marginRight: 8 },
	title: { fontSize: 16, fontWeight: "800", color: "#0f172a", marginBottom: 2 },
	artist: { fontSize: 13, fontWeight: "600", color: "#3b82f6" },
	controls: { flexDirection: "row", alignItems: "center" },
	btn: { padding: 8, marginLeft: 4 },
});

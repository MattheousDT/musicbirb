import React from "react";
import { View, Pressable, StyleSheet, ActivityIndicator } from "react-native";
import { Ionicons } from "@expo/vector-icons";
import { useMusicbirb } from "../context/MusicbirbContext";

export function PlayerControls() {
	const { playlistStatus, togglePause, next, prev, isBuffering } =
		useMusicbirb();
	const isPlaying = playlistStatus?.playing ?? false;

	return (
		<View style={styles.container}>
			<Pressable onPress={prev} style={styles.skipBtn}>
				<Ionicons name="play-back" size={32} color="#0f172a" />
			</Pressable>

			<Pressable
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
			</Pressable>

			<Pressable onPress={next} style={styles.skipBtn}>
				<Ionicons name="play-forward" size={32} color="#0f172a" />
			</Pressable>
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

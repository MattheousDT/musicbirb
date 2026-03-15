import { useMusicbirb } from "@/context/MusicbirbContext";
import { Ionicons } from "@expo/vector-icons";
import { useQuery } from "@tanstack/react-query";
import { Image } from "expo-image";
import { Link, Stack, useLocalSearchParams, useRouter } from "expo-router";
import React, { useMemo } from "react";
import { Pressable, ScrollView, StyleSheet, Text, View } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";

const getCoverUrl = (id?: string | null) => {
	if (!id) return null;
	return `${process.env.EXPO_PUBLIC_SUBSONIC_URL}/rest/getCoverArt?id=${id}&u=${process.env.EXPO_PUBLIC_SUBSONIC_USER}&p=${process.env.EXPO_PUBLIC_SUBSONIC_PASS}&v=1.16.1&c=musicbirb`;
};

export default function AlbumScreen() {
	const { id } = useLocalSearchParams<{ id: string }>();
	const router = useRouter();
	const { core, playAlbum, playIndex, queueAlbum, clearQueue } = useMusicbirb();

	const { data: album, isLoading } = useQuery({
		queryKey: ["album", id],
		queryFn: async () => {
			if (!core) return null;
			return await core.getAlbumDetails(id);
		},
		enabled: !!core && !!id,
	});

	const handlePlayAlbum = async () => {
		if (!id) return;
		await playAlbum(id);
	};

	const handlePlayTrack = async (idx: number) => {
		if (!id) return;
		clearQueue();
		await queueAlbum(id);
		playIndex(idx);
	};

	const screenOptions = useMemo(
		() => (
			<Stack.Screen
				options={{
					title: isLoading ? "Loading" : (album?.title ?? "Unknown Album"),
					headerTitle: () => <View />,
					headerShown: true,
					headerTransparent: true,
					headerBackButtonDisplayMode: "minimal",
				}}
			/>
		),
		[isLoading, album?.title],
	);

	if (isLoading || !album) {
		return (
			<View style={styles.center}>
				{screenOptions}
				<Text>Loading...</Text>
			</View>
		);
	}

	return (
		<ScrollView style={styles.root} showsVerticalScrollIndicator={false}>
			{screenOptions}
			<SafeAreaView>
				<View style={styles.info}>
					<Link.AppleZoomTarget>
						<Image
							key={album.coverArt}
							source={
								getCoverUrl(album.coverArt)
									? { uri: getCoverUrl(album.coverArt)! }
									: require("@assets/icon.png")
							}
							style={styles.cover}
							cachePolicy="memory-disk"
						/>
					</Link.AppleZoomTarget>
					<Text style={styles.title}>{album.title}</Text>
					<Pressable
						onPress={() =>
							album.artistId
								? router.push({
										pathname: "/artists/[id]",
										params: { id: album.artistId },
									})
								: null
						}
					>
						<Text style={styles.artist}>{album.artist}</Text>
					</Pressable>
					<Text style={styles.meta}>
						{album.year ? `${album.year} • ` : ""}
						{album.songCount} tracks • {Math.floor(album.durationSecs / 60)}{" "}
						mins
					</Text>
					<Pressable style={styles.playBtn} onPress={handlePlayAlbum}>
						<Ionicons name="play" size={20} color="#fff" />
						<Text style={styles.playBtnText}>Play Album</Text>
					</Pressable>
				</View>

				<View style={styles.tracklist}>
					{album.songs.map((track: any, idx: number) => (
						<Pressable
							key={track.id}
							style={styles.trackRow}
							onPress={() => handlePlayTrack(idx)}
						>
							<Text style={styles.trackNum}>{track.trackNum || idx + 1}</Text>
							<View style={styles.trackInfo}>
								<Text numberOfLines={1} style={styles.trackTitle}>
									{track.title}
								</Text>
								<Text numberOfLines={1} style={styles.trackArtist}>
									{track.artist}
								</Text>
							</View>
							<Text style={styles.trackDuration}>
								{Math.floor(track.durationSecs / 60)}:
								{(track.durationSecs % 60).toString().padStart(2, "0")}
							</Text>
						</Pressable>
					))}
				</View>
			</SafeAreaView>
		</ScrollView>
	);
}

const styles = StyleSheet.create({
	root: { flex: 1, backgroundColor: "#f8fafc" },
	header: { paddingHorizontal: 16, paddingTop: 8 },
	backBtn: {
		width: 44,
		height: 44,
		borderRadius: 22,
		backgroundColor: "#fff",
		justifyContent: "center",
		alignItems: "center",
		shadowColor: "#000",
		shadowOffset: { width: 0, height: 4 },
		shadowOpacity: 0.05,
		shadowRadius: 10,
		elevation: 2,
	},
	info: {
		alignItems: "center",
		paddingHorizontal: 24,
		paddingTop: 64,
		paddingBottom: 32,
	},
	cover: {
		width: 240,
		height: 240,
		borderRadius: 32,
		marginBottom: 24,
		backgroundColor: "#e2e8f0",
		shadowColor: "#000",
		shadowOffset: { width: 0, height: 20 },
		shadowOpacity: 0.15,
		shadowRadius: 30,
		elevation: 20,
	},
	title: {
		fontSize: 28,
		fontWeight: "900",
		color: "#0f172a",
		textAlign: "center",
		marginBottom: 8,
		letterSpacing: -0.5,
	},
	artist: {
		fontSize: 18,
		fontWeight: "700",
		color: "#3b82f6",
		textAlign: "center",
		marginBottom: 8,
	},
	meta: {
		fontSize: 14,
		fontWeight: "600",
		color: "#64748b",
		textAlign: "center",
		marginBottom: 24,
	},
	playBtn: {
		flexDirection: "row",
		alignItems: "center",
		backgroundColor: "#3b82f6",
		paddingHorizontal: 32,
		paddingVertical: 14,
		borderRadius: 100,
		gap: 8,
	},
	playBtnText: { color: "#fff", fontSize: 16, fontWeight: "800" },
	tracklist: { paddingHorizontal: 24, paddingBottom: 64, gap: 4 },
	trackRow: { flexDirection: "row", alignItems: "center", paddingVertical: 12 },
	trackNum: {
		width: 32,
		fontSize: 15,
		fontWeight: "700",
		color: "#94a3b8",
		textAlign: "center",
	},
	trackInfo: { flex: 1, marginLeft: 12 },
	trackTitle: {
		fontSize: 15,
		fontWeight: "700",
		color: "#0f172a",
		marginBottom: 2,
	},
	trackArtist: { fontSize: 13, fontWeight: "500", color: "#64748b" },
	trackDuration: {
		fontSize: 13,
		fontWeight: "600",
		color: "#94a3b8",
		marginLeft: 12,
	},
	center: { flex: 1, justifyContent: "center", alignItems: "center" },
});

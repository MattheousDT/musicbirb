import { PaginatedList } from "@/components/PaginatedList";
import { useMusicbirb } from "@/context/MusicbirbContext";
import { useQuery } from "@tanstack/react-query";
import { Image } from "expo-image";
import { Stack, useLocalSearchParams, useRouter } from "expo-router";
import React, { useMemo } from "react";
import {
	Dimensions,
	Platform,
	Pressable,
	ScrollView,
	StyleSheet,
	Text,
	View,
} from "react-native";
import {
	SafeAreaView,
	useSafeAreaInsets,
} from "react-native-safe-area-context";

const getCoverUrl = (id?: string | null) => {
	if (!id) return null;
	return `${process.env.EXPO_PUBLIC_SUBSONIC_URL}/rest/getCoverArt?id=${id}&u=${process.env.EXPO_PUBLIC_SUBSONIC_USER}&p=${process.env.EXPO_PUBLIC_SUBSONIC_PASS}&v=1.16.1&c=musicbirb`;
};

export default function ArtistScreen() {
	const { id } = useLocalSearchParams<{ id: string }>();
	const router = useRouter();
	const { core, clearQueue, playIndex } = useMusicbirb();
	const insets = useSafeAreaInsets();

	const { data: artist, isLoading } = useQuery({
		queryKey: ["artist", id],
		queryFn: async () => {
			if (!core) return null;
			return await core.getArtistDetails(id);
		},
		enabled: !!core && !!id,
	});

	const playTopSong = async (idx: number) => {
		if (!artist) return;
		clearQueue();
		for (const track of artist.topSongs) {
			await core?.queueTrack(track.id);
		}
		playIndex(idx);
	};

	const screenOptions = useMemo(
		() => (
			<Stack.Screen
				options={{
					title: isLoading ? "Loading" : (artist?.name ?? "Unknown Artist"),
					headerTitle: () => <View />,
					headerShown: true,
					headerTransparent: true,
					headerBackButtonDisplayMode: "minimal",
				}}
			/>
		),
		[isLoading, artist?.name],
	);

	if (isLoading || !artist) {
		return (
			<View style={styles.center}>
				{screenOptions}
				<Text>Loading...</Text>
			</View>
		);
	}

	const cleanBio =
		artist.biography?.replace(/<[^>]+>/g, "").trim() ||
		"No biography available.";

	return (
		<ScrollView
			style={styles.root}
			showsVerticalScrollIndicator={false}
			{
				// Don't ask me why I need to do this...
				...(Platform.OS === "ios"
					? {
							contentInsetAdjustmentBehavior: "always",
							contentInset: { top: -(insets.top + 54) },
							bounces: false,
						}
					: {})
			}
		>
			{screenOptions}
			<SafeAreaView edges={["bottom", "left", "right"]}>
				<Image
					source={
						getCoverUrl(artist.coverArt)
							? { uri: getCoverUrl(artist.coverArt)! }
							: require("@assets/icon.png")
					}
					style={[styles.heroImage, { height: 300 + insets.top }]}
					contentFit="cover"
					transition={500}
					cachePolicy="memory-disk"
				/>
				<View style={styles.content}>
					<Text style={styles.title}>{artist.name}</Text>
					<Text style={styles.bio} numberOfLines={8}>
						{cleanBio}
					</Text>
				</View>

				{artist.topSongs && artist.topSongs.length > 0 && (
					<View style={styles.topSongs}>
						<Text
							style={[styles.sectionTitle, styles.content, { marginBottom: 0 }]}
						>
							Top Tracks
						</Text>
						<PaginatedList
							data={artist.topSongs}
							perPage={5}
							renderItem={({ item, index }) => (
								<Pressable
									key={item.id}
									style={styles.trackRow}
									onPress={() => playTopSong(index)}
								>
									<Image
										source={
											getCoverUrl(item.coverArt)
												? { uri: getCoverUrl(item.coverArt)! }
												: require("@assets/icon.png")
										}
										style={styles.trackArt}
										cachePolicy="memory-disk"
									/>
									<View style={styles.trackInfo}>
										<Text numberOfLines={1} style={styles.trackTitle}>
											{item.title}
										</Text>
										<Text numberOfLines={1} style={styles.trackArtist}>
											{item.artist}
										</Text>
									</View>
									<Text style={styles.trackDuration}>
										{Math.floor(item.durationSecs / 60)}:
										{(item.durationSecs % 60).toString().padStart(2, "0")}
									</Text>
								</Pressable>
							)}
						/>
					</View>
				)}

				<View style={styles.content}>
					<Text style={styles.sectionTitle}>Albums</Text>
					<View style={styles.albums}>
						{artist.albums.map((item) => (
							<Pressable
								id={item.id}
								style={styles.albumCard}
								onPress={() =>
									router.push({
										pathname: "/albums/[id]",
										params: { id: item.id },
									})
								}
							>
								<Image
									source={
										getCoverUrl(item.coverArt)
											? { uri: getCoverUrl(item.coverArt)! }
											: require("@assets/icon.png")
									}
									style={styles.albumArt}
									cachePolicy="memory-disk"
								/>
								<Text numberOfLines={1} style={styles.albumTitle}>
									{item.title}
								</Text>
								<Text numberOfLines={1} style={styles.albumYear}>
									{item.year || "Album"}
								</Text>
							</Pressable>
						))}
					</View>
				</View>
			</SafeAreaView>
		</ScrollView>
	);
}

const styles = StyleSheet.create({
	root: { flex: 1, backgroundColor: "#f8fafc" },
	heroImage: {
		width: "100%",
		objectFit: "cover",
	},
	content: { paddingHorizontal: 24, marginTop: 24 },
	title: {
		fontSize: 32,
		fontWeight: "900",
		color: "#0f172a",
		letterSpacing: -1,
	},
	bio: { fontSize: 15, lineHeight: 22, color: "#475569", marginTop: 4 },
	sectionTitle: {
		fontSize: 20,
		fontWeight: "800",
		color: "#1e293b",
		marginBottom: 12,
		letterSpacing: -0.5,
	},
	topSongs: { gap: 12 },
	trackRow: {
		flexDirection: "row",
		alignItems: "center",
		backgroundColor: "#fff",
		padding: 12,
		paddingEnd: 20,
		borderRadius: 16,
		shadowColor: "#000",
		shadowOffset: { width: 0, height: 2 },
		shadowOpacity: 0.05,
		shadowRadius: 8,
		elevation: 2,
	},
	trackArt: {
		width: 48,
		height: 48,
		borderRadius: 12,
		backgroundColor: "#e2e8f0",
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
	albums: { display: "flex", flexDirection: "row", flexWrap: "wrap", gap: 16 },
	albumCard: {
		width: (Dimensions.get("screen").width - 48 - 16) / 2,
	},
	albumArt: {
		width: "100%",
		aspectRatio: 1,
		borderRadius: 20,
		marginBottom: 8,
		backgroundColor: "#e2e8f0",
	},
	albumTitle: {
		fontSize: 14,
		fontWeight: "700",
		color: "#0f172a",
		marginBottom: 2,
	},
	albumYear: { fontSize: 13, fontWeight: "600", color: "#64748b" },
	center: { flex: 1, justifyContent: "center", alignItems: "center" },
});

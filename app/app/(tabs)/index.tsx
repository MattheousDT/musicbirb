import { PaginatedList } from "@/components/PaginatedList";
import { useMusicbirb } from "@/context/MusicbirbContext";
import { useQuery } from "@tanstack/react-query";
import { Image } from "expo-image";
import React from "react";
import {
	Dimensions,
	FlatList,
	Pressable,
	ScrollView,
	StyleSheet,
	Text,
	View,
} from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";

const { width } = Dimensions.get("window");

export default function HomeScreen() {
	const { mobileClient, playAlbum, playPlaylist } = useMusicbirb();

	const { data: lastPlayed } = useQuery({
		queryKey: ["lastPlayed"],
		queryFn: async () => {
			if (!mobileClient) return [];
			return await mobileClient.getLastPlayedAlbums();
		},
		enabled: !!mobileClient,
		throwOnError: true,
	});

	const { data: recentlyAdded } = useQuery({
		queryKey: ["recentlyAdded"],
		queryFn: async () => {
			if (!mobileClient) return [];
			return await mobileClient.getRecentlyAddedAlbums();
		},
		enabled: !!mobileClient,
	});

	const { data: newReleases } = useQuery({
		queryKey: ["newReleases"],
		queryFn: async () => {
			if (!mobileClient) return [];
			return await mobileClient.getNewReleases();
		},
		enabled: !!mobileClient,
	});

	const { data: playlists } = useQuery({
		queryKey: ["playlists"],
		queryFn: async () => {
			if (!mobileClient) return [];
			return await mobileClient.getPlaylists();
		},
		enabled: !!mobileClient,
	});

	const renderAlbum = ({ item }: { item: any }) => {
		const coverUrl = item.coverArt
			? `${process.env.EXPO_PUBLIC_SUBSONIC_URL}/rest/getCoverArt?id=${item.coverArt}&u=${process.env.EXPO_PUBLIC_SUBSONIC_USER}&p=${process.env.EXPO_PUBLIC_SUBSONIC_PASS}&v=1.16.1&c=musicbirb`
			: null;

		return (
			<Pressable style={styles.albumCard} onPress={() => playAlbum(item.id)}>
				<Image
					source={
						coverUrl ? { uri: coverUrl } : require("../../assets/icon.png")
					}
					style={styles.albumArt}
				/>
				<Text numberOfLines={1} style={styles.albumTitle}>
					{item.title}
				</Text>
				<Text numberOfLines={1} style={styles.albumArtist}>
					{item.artist}
				</Text>
			</Pressable>
		);
	};

	const renderPlaylistRow = (item: any, isAlbum = false) => {
		const coverUrl = item.coverArt
			? `${process.env.EXPO_PUBLIC_SUBSONIC_URL}/rest/getCoverArt?id=${item.coverArt}&u=${process.env.EXPO_PUBLIC_SUBSONIC_USER}&p=${process.env.EXPO_PUBLIC_SUBSONIC_PASS}&v=1.16.1&c=musicbirb`
			: null;

		return (
			<Pressable
				key={item.id}
				style={styles.playlistRow}
				onPress={() => (isAlbum ? playAlbum(item.id) : playPlaylist(item.id))}
			>
				<Image
					source={
						coverUrl ? { uri: coverUrl } : require("../../assets/icon.png")
					}
					style={styles.playlistArt}
				/>
				<View style={styles.playlistInfo}>
					<Text numberOfLines={1} style={styles.playlistName}>
						{isAlbum ? item.title : item.name}
					</Text>
					<Text numberOfLines={1} style={styles.playlistMeta}>
						{isAlbum
							? item.artist
							: `${item.songCount} tracks • ${Math.floor(item.durationSecs / 60)} mins`}
					</Text>
				</View>
			</Pressable>
		);
	};

	return (
		<ScrollView
			style={styles.root}
			showsVerticalScrollIndicator={false}
			contentContainerStyle={styles.scroll}
		>
			<SafeAreaView>
				<Text style={styles.header}>Home</Text>

				<Text style={styles.sectionTitle}>Last Played</Text>
				<FlatList
					horizontal
					showsHorizontalScrollIndicator={false}
					data={lastPlayed}
					keyExtractor={(item) => item.id}
					renderItem={renderAlbum}
					contentContainerStyle={styles.carousel}
					snapToInterval={140 + 16}
					decelerationRate="fast"
				/>

				<Text style={styles.sectionTitle}>Recently Added</Text>
				<FlatList
					horizontal
					showsHorizontalScrollIndicator={false}
					data={recentlyAdded}
					keyExtractor={(item) => item.id}
					renderItem={renderAlbum}
					contentContainerStyle={styles.carousel}
					snapToInterval={140 + 16}
					decelerationRate="fast"
				/>

				<Text style={styles.sectionTitle}>New Releases</Text>
				<PaginatedList
					data={newReleases ?? []}
					perPage={5}
					keyExtractor={(_, idx) => `nr-${idx}`}
					renderItem={({ item }) => renderPlaylistRow(item, true)}
				/>

				<Text style={styles.sectionTitle}>Playlists</Text>
				<PaginatedList
					data={playlists ?? []}
					perPage={5}
					keyExtractor={(_, idx) => `pl-${idx}`}
					renderItem={({ item }) => renderPlaylistRow(item, false)}
				/>
			</SafeAreaView>
		</ScrollView>
	);
}

const styles = StyleSheet.create({
	root: { flex: 1, backgroundColor: "#ffffff" },
	scroll: { paddingVertical: 16 },
	header: {
		fontSize: 36,
		fontWeight: "900",
		color: "#0f172a",
		paddingHorizontal: 24,
		letterSpacing: -1,
	},
	sectionTitle: {
		fontSize: 22,
		fontWeight: "800",
		color: "#1e293b",
		paddingHorizontal: 24,
		marginBottom: 16,
		marginTop: 32,
		letterSpacing: -0.5,
	},
	carousel: { paddingHorizontal: 24, gap: 16 },
	albumCard: { width: 140 },
	albumArt: {
		width: 140,
		height: 140,
		borderRadius: 20,
		marginBottom: 8,
		backgroundColor: "#f8fafc",
	},
	albumTitle: {
		fontSize: 15,
		fontWeight: "700",
		color: "#0f172a",
		marginBottom: 2,
	},
	albumArtist: { fontSize: 13, fontWeight: "600", color: "#64748b" },
	playlistRow: {
		flexDirection: "row",
		alignItems: "center",
		backgroundColor: "#f8fafc",
		padding: 10,
		borderRadius: 16,
	},
	playlistArt: {
		width: 48,
		height: 48,
		borderRadius: 12,
		backgroundColor: "#e2e8f0",
	},
	playlistInfo: { marginLeft: 12, flex: 1 },
	playlistName: {
		fontSize: 14,
		fontWeight: "700",
		color: "#0f172a",
		marginBottom: 2,
	},
	playlistMeta: { fontSize: 13, fontWeight: "500", color: "#64748b" },
});

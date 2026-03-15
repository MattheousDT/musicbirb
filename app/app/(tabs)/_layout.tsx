import { NativeTabs } from "expo-router/unstable-native-tabs";
import { StatusBar } from "expo-status-bar";
import React from "react";
import { Platform, StyleSheet, View } from "react-native";
import { CurrentlyPlayingBar } from "../../src/components/CurrentlyPlayingBar";

export default function TabsLayout() {
	return (
		<View style={styles.container}>
			{Platform.OS !== "ios" && <StatusBar animated />}
			<CurrentlyPlayingBar />
			<NativeTabs
				tintColor={Platform.OS === "ios" ? "#3b82f6" : undefined}
				labelVisibilityMode="labeled"
			>
				<NativeTabs.Trigger name="(home)">
					<NativeTabs.Trigger.Label>Home</NativeTabs.Trigger.Label>
					<NativeTabs.Trigger.Icon sf="house.fill" md="home" />
				</NativeTabs.Trigger>

				<NativeTabs.Trigger name="(library)">
					<NativeTabs.Trigger.Label>Library</NativeTabs.Trigger.Label>
					<NativeTabs.Trigger.Icon sf="music.note.list" md="library_music" />
				</NativeTabs.Trigger>

				<NativeTabs.Trigger name="downloads" role="downloads">
					<NativeTabs.Trigger.Label>Downloads</NativeTabs.Trigger.Label>
					<NativeTabs.Trigger.Icon md="download" />
				</NativeTabs.Trigger>

				<NativeTabs.Trigger name="search" role="search">
					<NativeTabs.Trigger.Label>Search</NativeTabs.Trigger.Label>
					<NativeTabs.Trigger.Icon md="search" />
				</NativeTabs.Trigger>
			</NativeTabs>
		</View>
	);
}

const styles = StyleSheet.create({
	container: { flex: 1 },
});

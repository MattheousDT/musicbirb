import { NativeTabs } from "expo-router/unstable-native-tabs";
import React from "react";
import { StyleSheet, View } from "react-native";
import { CurrentlyPlayingBar } from "../../src/components/CurrentlyPlayingBar";

export default function TabsLayout() {
  return (
    <View style={styles.container}>
      <NativeTabs tintColor="#3b82f6" backgroundColor="#ffffff">
        <NativeTabs.Trigger name="index">
          <NativeTabs.Trigger.Label>Home</NativeTabs.Trigger.Label>
          <NativeTabs.Trigger.Icon sf="house.fill" />
        </NativeTabs.Trigger>

        <NativeTabs.Trigger name="library">
          <NativeTabs.Trigger.Label>Library</NativeTabs.Trigger.Label>
          <NativeTabs.Trigger.Icon sf="music.note.list" />
        </NativeTabs.Trigger>

        <NativeTabs.Trigger name="downloads" role="downloads">
          <NativeTabs.Trigger.Label>Downloads</NativeTabs.Trigger.Label>
        </NativeTabs.Trigger>

        <NativeTabs.Trigger name="search" role="search">
          <NativeTabs.Trigger.Label>Search</NativeTabs.Trigger.Label>
        </NativeTabs.Trigger>
      </NativeTabs>
      <CurrentlyPlayingBar />
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1 },
});

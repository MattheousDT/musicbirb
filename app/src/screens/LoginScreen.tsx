import React, { useState } from "react";
import { View, Text, TextInput, Pressable, SafeAreaView } from "react-native";
import { useMusicbirb } from "../context/MusicbirbContext";
import { Ionicons } from "@expo/vector-icons";

export function LoginScreen() {
  const { setConfig } = useMusicbirb();
  const [url, setUrl] = useState("");
  const [user, setUser] = useState("");
  const [pass, setPass] = useState("");

  const handleLogin = () => {
    if (url && user && pass) {
      setConfig({ url, user, pass });
    }
  };

  return (
    <SafeAreaView className="flex-1 bg-slate-950 justify-center px-8">
      <View className="items-center mb-16">
        <View className="w-20 h-20 bg-blue-500 rounded-3xl justify-center items-center mb-6 shadow-xl shadow-blue-500/30">
          <Ionicons name="musical-notes" size={40} color="white" />
        </View>
        <Text className="text-4xl font-black text-white tracking-tight mb-2">
          Musicbirb
        </Text>
        <Text className="text-white/50 text-base font-medium">
          Connect to your Subsonic server
        </Text>
      </View>

      <View className="gap-y-4">
        <TextInput
          className="bg-white/5 border border-white/10 text-white px-5 py-4 rounded-2xl text-lg font-medium"
          placeholder="Server URL (e.g. https://...)"
          placeholderTextColor="rgba(255,255,255,0.3)"
          value={url}
          onChangeText={setUrl}
          autoCapitalize="none"
          autoCorrect={false}
          keyboardType="url"
        />
        <TextInput
          className="bg-white/5 border border-white/10 text-white px-5 py-4 rounded-2xl text-lg font-medium"
          placeholder="Username"
          placeholderTextColor="rgba(255,255,255,0.3)"
          value={user}
          onChangeText={setUser}
          autoCapitalize="none"
          autoCorrect={false}
        />
        <TextInput
          className="bg-white/5 border border-white/10 text-white px-5 py-4 rounded-2xl text-lg font-medium"
          placeholder="Password"
          placeholderTextColor="rgba(255,255,255,0.3)"
          value={pass}
          onChangeText={setPass}
          secureTextEntry
        />

        <Pressable
          onPress={handleLogin}
          className="bg-blue-600 mt-6 py-4 rounded-2xl items-center shadow-lg shadow-blue-600/40 active:opacity-80 active:scale-[0.98]"
        >
          <Text className="text-white text-lg font-bold">Connect Server</Text>
        </Pressable>
      </View>
    </SafeAreaView>
  );
}

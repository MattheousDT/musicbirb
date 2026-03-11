import React, { useState } from "react";
import {
  View,
  Text,
  TextInput,
  StyleSheet,
  KeyboardAvoidingView,
  Platform,
  TouchableWithoutFeedback,
  Keyboard,
} from "react-native";
import { useMusicbirb } from "../context/MusicbirbContext";

interface Props {
  onComplete: () => void;
}

export function SetupScreen({ onComplete }: Props) {
  const { queueTrack, queueAlbum, queuePlaylist } = useMusicbirb();
  const [input, setInput] = useState("");

  const handleCommand = () => {
    const cmd = input.trim();
    if (!cmd) return;

    if (cmd.startsWith("al:")) queueAlbum(cmd.replace("al:", ""));
    else if (cmd.startsWith("pl:")) queuePlaylist(cmd.replace("pl:", ""));
    else queueTrack(cmd);

    setInput("");
    onComplete();
  };

  return (
    <TouchableWithoutFeedback onPress={Keyboard.dismiss}>
      <KeyboardAvoidingView
        behavior={Platform.OS === "ios" ? "padding" : "height"}
        style={styles.container}
      >
        <View style={styles.inner}>
          <Text style={styles.logo}>BIRB</Text>
          <View style={styles.inputContainer}>
            <Text style={styles.prompt}>{">"}</Text>
            <TextInput
              autoFocus
              style={styles.input}
              placeholder="Track ID, al:ID, or pl:ID"
              placeholderTextColor="#94a3b8"
              value={input}
              onChangeText={setInput}
              onSubmitEditing={handleCommand}
              autoCorrect={false}
              autoCapitalize="none"
            />
          </View>
          <Text style={styles.help}>Enter a Subsonic ID to start playback</Text>
        </View>
      </KeyboardAvoidingView>
    </TouchableWithoutFeedback>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: "#f8fafc" },
  inner: { flex: 1, justifyContent: "center", padding: 40 },
  logo: {
    fontSize: 48,
    fontWeight: "900",
    color: "#0f172a",
    letterSpacing: -2,
    marginBottom: 40,
  },
  inputContainer: {
    flexDirection: "row",
    alignItems: "center",
    backgroundColor: "#ffffff",
    borderRadius: 20,
    paddingHorizontal: 20,
    height: 72,
    shadowColor: "#000",
    shadowOffset: { width: 0, height: 10 },
    shadowOpacity: 0.05,
    shadowRadius: 20,
    elevation: 5,
  },
  prompt: {
    fontSize: 24,
    fontWeight: "800",
    color: "#3b82f6",
    marginRight: 12,
  },
  input: { flex: 1, fontSize: 18, color: "#1e293b", fontWeight: "600" },
  help: {
    marginTop: 20,
    color: "#64748b",
    fontSize: 14,
    fontWeight: "500",
    textAlign: "center",
  },
});

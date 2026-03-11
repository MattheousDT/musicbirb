import React, { useState } from "react";
import { StatusBar } from "react-native";
import { SafeAreaProvider } from "react-native-safe-area-context";
import { MusicbirbProvider } from "./src/context/MusicbirbContext";
import { PlayerScreen } from "./src/screens/PlayerScreen";
import { SetupScreen } from "./src/screens/SetupScreen";

export default function App() {
  const [isSetupComplete, setSetupComplete] = useState(false);

  return (
    <SafeAreaProvider>
      <StatusBar
        barStyle="dark-content"
        translucent
        backgroundColor="transparent"
      />
      <MusicbirbProvider>
        {!isSetupComplete ? (
          <SetupScreen onComplete={() => setSetupComplete(true)} />
        ) : (
          <PlayerScreen />
        )}
      </MusicbirbProvider>
    </SafeAreaProvider>
  );
}

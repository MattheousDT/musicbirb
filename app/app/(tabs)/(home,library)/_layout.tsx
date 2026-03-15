import { Stack } from "expo-router";
import { useMemo } from "react";

const Layout = ({ segment }: { segment: string }) => {
  const rootScreen = useMemo(() => {
    switch (segment) {
      case "(home)":
        return <Stack.Screen name="index" />;
      case "(library)":
        return <Stack.Screen name="library" />;
    }
  }, [segment]);

  // shared routes go here
  return <Stack>{rootScreen}</Stack>;
};

export default Layout;

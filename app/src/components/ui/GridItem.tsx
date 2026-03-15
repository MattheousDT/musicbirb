import React from "react";
import {
  PressableProps,
  StyleProp,
  StyleSheet,
  Text,
  TextProps,
  View,
  ViewStyle,
} from "react-native";
import { InteractivePressable } from "@/components/ui/InteractivePressable";

export interface GridItemProps extends PressableProps {
  style?: StyleProp<ViewStyle>;
  children?: React.ReactNode;
}

const GridItemRoot = React.forwardRef<View, GridItemProps>(
  ({ style, ...rest }, ref) => {
    return (
      <InteractivePressable ref={ref} style={[styles.root, style]} {...rest} />
    );
  },
);
GridItemRoot.displayName = "GridItem";

const GridItemTitle = ({ style, ...rest }: TextProps) => (
  <Text numberOfLines={1} style={[styles.title, style]} {...rest} />
);
GridItemTitle.displayName = "GridItem.Title";

const GridItemSubtitle = ({ style, ...rest }: TextProps) => (
  <Text numberOfLines={1} style={[styles.subtitle, style]} {...rest} />
);
GridItemSubtitle.displayName = "GridItem.Subtitle";

export const GridItem = Object.assign(GridItemRoot, {
  Title: GridItemTitle,
  Subtitle: GridItemSubtitle,
});

const styles = StyleSheet.create({
  root: {},
  title: { fontSize: 15, fontWeight: "700", color: "#0f172a", marginBottom: 2 },
  subtitle: { fontSize: 13, fontWeight: "600", color: "#64748b" },
});

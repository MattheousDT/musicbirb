import { InteractivePressable } from "@/components/ui/InteractivePressable";
import React from "react";
import {
  PressableProps,
  StyleProp,
  StyleSheet,
  Text,
  TextProps,
  View,
  ViewProps,
  ViewStyle,
} from "react-native";

export interface ListItemProps extends PressableProps {
  style?: StyleProp<ViewStyle>;
  children?: React.ReactNode;
}

const ListItemRoot = React.forwardRef<View, ListItemProps>(
  ({ style, ...rest }, ref) => {
    return (
      <InteractivePressable ref={ref} style={[styles.root, style]} {...rest} />
    );
  },
);
ListItemRoot.displayName = "ListItem";

const ListItemLeading = ({ style, ...rest }: ViewProps) => (
  <View style={[styles.leading, style]} {...rest} />
);
ListItemLeading.displayName = "ListItem.Leading";

const ListItemContent = ({ style, ...rest }: ViewProps) => (
  <View style={[styles.content, style]} {...rest} />
);
ListItemContent.displayName = "ListItem.Content";

const ListItemTitle = ({ style, ...rest }: TextProps) => (
  <Text numberOfLines={1} style={[styles.title, style]} {...rest} />
);
ListItemTitle.displayName = "ListItem.Title";

const ListItemSubtitle = ({ style, ...rest }: TextProps) => (
  <Text numberOfLines={1} style={[styles.subtitle, style]} {...rest} />
);
ListItemSubtitle.displayName = "ListItem.Subtitle";

const ListItemTrailing = ({ style, ...rest }: ViewProps) => (
  <View style={[styles.trailing, style]} {...rest} />
);
ListItemTrailing.displayName = "ListItem.Trailing";

export const ListItem = Object.assign(ListItemRoot, {
  Leading: ListItemLeading,
  Content: ListItemContent,
  Title: ListItemTitle,
  Subtitle: ListItemSubtitle,
  Trailing: ListItemTrailing,
});

const styles = StyleSheet.create({
  root: {
    flexDirection: "row",
    alignItems: "center",
    paddingVertical: 12,
    paddingHorizontal: 16,
    borderRadius: 16,
    minHeight: 64,
  },
  leading: { marginRight: 12, justifyContent: "center", alignItems: "center" },
  content: { flex: 1, justifyContent: "center" },
  title: { fontSize: 15, fontWeight: "700", color: "#0f172a", marginBottom: 2 },
  subtitle: { fontSize: 13, fontWeight: "500", color: "#64748b" },
  trailing: { marginLeft: 12, flexDirection: "row", alignItems: "center" },
});

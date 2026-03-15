import React from "react";
import { Pressable, PressableProps, View } from "react-native";

export const InteractivePressable = React.forwardRef<View, PressableProps>(
  ({ style, ...props }, ref) => {
    return (
      <Pressable
        ref={ref}
        style={(state) => {
          const baseStyle = typeof style === "function" ? style(state) : style;
          return [baseStyle, state.pressed && { opacity: 0.7 }];
        }}
        {...props}
      />
    );
  },
);

InteractivePressable.displayName = "InteractivePressable";

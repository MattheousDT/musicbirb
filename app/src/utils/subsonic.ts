export const getCoverUrl = (id?: string | null) => {
  if (!id) return null;
  return `${process.env.EXPO_PUBLIC_SUBSONIC_URL}/rest/getCoverArt?id=${id}&u=${process.env.EXPO_PUBLIC_SUBSONIC_USER}&p=${process.env.EXPO_PUBLIC_SUBSONIC_PASS}&v=1.16.1&c=musicbirb`;
};

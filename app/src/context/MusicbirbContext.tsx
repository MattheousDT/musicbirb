import React, {
  createContext,
  useContext,
  useEffect,
  useRef,
  useState,
  useCallback,
} from "react";
import {
  useAudioPlaylist,
  AudioPlaylist,
  setAudioModeAsync,
  AudioPlaylistStatus,
} from "expo-audio";
import {
  AudioEngineDelegate,
  FfiPlayerState,
  FfiPlayerStatus,
  FfiUiState,
  MusicbirbMobile,
} from "musicbirb-ffi";

interface MusicbirbContextValue {
  uiState: FfiUiState | null;
  playlistStatus: AudioPlaylistStatus | null;
  queueTrack: (id: string) => void;
  queueAlbum: (id: string) => void;
  queuePlaylist: (id: string) => void;
  togglePause: () => void;
  next: () => void;
  prev: () => void;
  seek: (seconds: number) => void;
  playIndex: (index: number) => void;
}

const MusicbirbContext = createContext<MusicbirbContextValue | null>(null);

class DelegateImpl implements AudioEngineDelegate {
  private lastKnownCount = 0;

  constructor(
    private playlist: AudioPlaylist,
    private onUpdate: () => void,
  ) {}

  play() {
    this.playlist.play();
  }
  pause() {
    this.playlist.pause();
  }
  togglePause() {
    this.playlist.playing ? this.playlist.pause() : this.playlist.play();
  }
  stop() {
    this.playlist.pause();
    this.playlist.seekTo(0);
  }

  add(url: string) {
    this.playlist.add({ uri: url });
    this.lastKnownCount++;
    this.onUpdate();
  }

  insert(url: string, index: number) {
    this.playlist.insert({ uri: url }, index);
    this.lastKnownCount++;
    this.onUpdate();
  }

  removeIndex(index: number) {
    this.playlist.remove(index);
    this.lastKnownCount = Math.max(0, this.lastKnownCount - 1);
    this.onUpdate();
  }

  clearPlaylist() {
    this.playlist.clear();
    this.lastKnownCount = 0;
    this.onUpdate();
  }

  playIndex(index: number) {
    this.playlist.skipTo(index);
  }
  seekRelative(seconds: number) {
    this.playlist.seekTo(this.playlist.currentTime + seconds);
  }
  seekAbsolute(seconds: number) {
    this.playlist.seekTo(seconds);
  }
  setVolume(volume: number) {
    this.playlist.volume = volume;
  }
  getVolume() {
    return this.playlist.volume;
  }

  getState(): FfiPlayerState {
    let status = FfiPlayerStatus.Stopped;
    if (this.playlist.playing) status = FfiPlayerStatus.Playing;
    else if (this.playlist.currentTime > 0) status = FfiPlayerStatus.Paused;

    return {
      positionSecs: this.playlist.currentTime,
      status,
      playlistIndex: this.playlist.currentIndex,
      playlistCount: Math.max(this.lastKnownCount, this.playlist.trackCount),
    };
  }
}

export function MusicbirbProvider({ children }: { children: React.ReactNode }) {
  const [uiState, setUiState] = useState<FfiUiState | null>(null);
  const [playlistStatus, setPlaylistStatus] =
    useState<AudioPlaylistStatus | null>(null);
  const playlist = useAudioPlaylist({ loop: "none" });
  const mobileRef = useRef<MusicbirbMobile | null>(null);
  const isProcessingDelegateCall = useRef(false);

  const updateUiState = useCallback(() => {
    if (mobileRef.current) {
      setUiState(mobileRef.current.getUiState());
    }
  }, []);

  const wrappedOnUpdate = useCallback(() => {
    isProcessingDelegateCall.current = true;
    updateUiState();
    setTimeout(() => {
      isProcessingDelegateCall.current = false;
    }, 100);
  }, [updateUiState]);

  useEffect(() => {
    setAudioModeAsync({
      playsInSilentMode: true,
      shouldPlayInBackground: true,
      interruptionMode: "doNotMix",
    });

    const url = process.env.EXPO_PUBLIC_SUBSONIC_URL || "";
    const user = process.env.EXPO_PUBLIC_SUBSONIC_USER || "";
    const pass = process.env.EXPO_PUBLIC_SUBSONIC_PASS || "";

    if (url && user && pass) {
      const delegate = new DelegateImpl(playlist, wrappedOnUpdate);
      mobileRef.current = new MusicbirbMobile(url, user, pass, delegate);
      updateUiState();
    }

    return () => {
      mobileRef.current?.uniffiDestroy();
      mobileRef.current = null;
    };
  }, [playlist, wrappedOnUpdate, updateUiState]);

  useEffect(() => {
    if (!mobileRef.current) return;
    const target = mobileRef.current.getEventTarget();

    const sub = playlist.addListener("playlistStatusUpdate", (status) => {
      setPlaylistStatus(status);
      if (isProcessingDelegateCall.current) return;

      let ffiStatus = FfiPlayerStatus.Stopped;
      if (status.playing) ffiStatus = FfiPlayerStatus.Playing;
      else if (status.currentTime > 0) ffiStatus = FfiPlayerStatus.Paused;

      target.onStatusUpdate(ffiStatus);
      target.onPositionCorrection(status.currentTime);
      if (status.didJustFinish) target.onEndOfTrack();
    });

    const trackSub = playlist.addListener("trackChanged", () => {
      if (isProcessingDelegateCall.current) return;
      target.onTrackStarted();
      updateUiState();
    });

    return () => {
      sub.remove();
      trackSub.remove();
    };
  }, [playlist, updateUiState]);

  return (
    <MusicbirbContext.Provider
      value={{
        uiState,
        playlistStatus,
        queueTrack: (id) => mobileRef.current?.queueTrack(id),
        queueAlbum: (id) => mobileRef.current?.queueAlbum(id),
        queuePlaylist: (id) => mobileRef.current?.queuePlaylist(id),
        togglePause: () => mobileRef.current?.togglePause(),
        next: () => mobileRef.current?.next(),
        prev: () => mobileRef.current?.prev(),
        seek: (s) => mobileRef.current?.seek(s),
        playIndex: (idx) => mobileRef.current?.playIndex(idx),
      }}
    >
      {children}
    </MusicbirbContext.Provider>
  );
}

export const useMusicbirb = () => {
  const ctx = useContext(MusicbirbContext);
  if (!ctx) throw new Error("MusicbirbProvider missing");
  return ctx;
};

import {
	AudioPlaylist,
	AudioPlaylistStatus,
	setAudioModeAsync,
	useAudioPlaylist,
} from "expo-audio";
import { Paths } from "expo-file-system";
import {
	AudioEngineDelegate,
	FfiPlayerState,
	PlayerStatus,
	UiState,
	MusicbirbInterface,
	StateObserver,
	initClient, // <-- Using the freestanding synchronous function
} from "musicbirb-ffi";
import React, {
	createContext,
	useContext,
	useEffect,
	useRef,
	useState,
} from "react";

interface MusicbirbContextValue {
	uiState: UiState | null;
	playlistStatus: AudioPlaylistStatus | null;
	core: MusicbirbInterface | null;
	isBuffering: boolean;

	// Appends to queue
	queueTrack: (id: string) => Promise<void>;
	queueAlbum: (id: string) => Promise<number>;
	queuePlaylist: (id: string) => Promise<number>;

	// Replaces queue completely
	playTrack: (id: string) => Promise<void>;
	playAlbum: (id: string) => Promise<number>;
	playPlaylist: (id: string) => Promise<number>;

	// Queue mutators
	clearQueue: () => void;
	removeIndex: (index: number) => void;

	togglePause: () => void;
	next: () => void;
	prev: () => void;
	seek: (seconds: number) => void;
	playIndex: (index: number) => void;
}

const MusicbirbContext = createContext<MusicbirbContextValue | null>(null);

class DelegateImpl implements AudioEngineDelegate {
	private isDestroyed = false;
	private internalCount = 0;

	constructor(private playlist: AudioPlaylist) {}

	destroy() {
		this.isDestroyed = true;
	}

	play() {
		if (!this.isDestroyed) this.playlist.play();
	}
	pause() {
		if (!this.isDestroyed) this.playlist.pause();
	}
	togglePause() {
		if (!this.isDestroyed) {
			this.playlist.playing ? this.playlist.pause() : this.playlist.play();
		}
	}
	stop() {
		if (this.isDestroyed) return;
		this.playlist.pause();
		this.playlist.seekTo(0);
	}

	add(url: string) {
		if (this.isDestroyed) return;
		this.playlist.add({ uri: url });
		this.internalCount++;
	}
	insert(url: string, index: number) {
		if (this.isDestroyed) return;
		this.playlist.insert({ uri: url }, index);
		this.internalCount++;
	}
	removeIndex(index: number) {
		if (this.isDestroyed) return;
		this.playlist.remove(index);
		this.internalCount = Math.max(0, this.internalCount - 1);
	}
	clearPlaylist() {
		if (this.isDestroyed) return;
		this.playlist.clear();
		this.internalCount = 0;
	}
	playIndex(index: number) {
		if (!this.isDestroyed) this.playlist.skipTo(index);
	}
	seekRelative(seconds: number) {
		if (!this.isDestroyed)
			this.playlist.seekTo(this.playlist.currentTime + seconds);
	}
	seekAbsolute(seconds: number) {
		if (!this.isDestroyed) this.playlist.seekTo(seconds);
	}
	setVolume(volume: number) {
		if (!this.isDestroyed) this.playlist.volume = volume;
	}
	getVolume() {
		return this.playlist.volume;
	}

	getState(): FfiPlayerState {
		let status = PlayerStatus.Stopped;

		if (this.playlist.isBuffering) status = PlayerStatus.Buffering;
		else if (this.playlist.playing) status = PlayerStatus.Playing;
		else if (this.playlist.currentTime > 0) status = PlayerStatus.Paused;

		const count = Math.max(this.internalCount, this.playlist.trackCount ?? 0);

		return {
			positionSecs: this.playlist.currentTime,
			status,
			playlistIndex: this.playlist.currentIndex,
			playlistCount: count,
		};
	}
}

class ObserverImpl implements StateObserver {
	constructor(private setUiState: (state: UiState) => void) {}
	onStateChanged(state: UiState) {
		this.setUiState(state);
	}
}

export function MusicbirbProvider({ children }: { children: React.ReactNode }) {
	const [uiState, setUiState] = useState<UiState | null>(null);
	const [playlistStatus, setPlaylistStatus] =
		useState<AudioPlaylistStatus | null>(null);
	const [core, setCore] = useState<MusicbirbInterface | null>(null);

	const playlist = useAudioPlaylist({ loop: "none" });
	const delegateRef = useRef<DelegateImpl | null>(null);

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
			try {
				const delegate = new DelegateImpl(playlist);
				delegateRef.current = delegate;
				const observer = new ObserverImpl(setUiState);

				const dataDir = Paths.document.uri.replace(/^file:\/\//, "") || "";
				const cacheDir = Paths.cache.uri.replace(/^file:\/\//, "") || "";

				// Synchronous boot resolves all Tokio context panic issues immediately
				const initialisedCore = initClient(
					url,
					user,
					pass,
					dataDir,
					cacheDir,
					delegate,
					observer,
				);

				setCore(initialisedCore);
			} catch (e) {
				console.error("FFI Initialization Error:", e);
			}
		}

		return () => {
			delegateRef.current?.destroy();
		};
	}, [playlist]);

	useEffect(() => {
		if (!core) return;
		const target = core.getEventTarget();

		const sub = playlist.addListener("playlistStatusUpdate", (status) => {
			setPlaylistStatus(status);

			let ffiStatus = PlayerStatus.Stopped;
			if (status.isBuffering) ffiStatus = PlayerStatus.Buffering;
			else if (status.playing) ffiStatus = PlayerStatus.Playing;
			else if (status.currentTime > 0) ffiStatus = PlayerStatus.Paused;

			target.onStatusUpdate(ffiStatus);
			target.onPositionCorrection(status.currentTime);

			if (status.didJustFinish) {
				target.onEndOfTrack();
			}
		});

		const trackSub = playlist.addListener("trackChanged", () => {
			target.onTrackStarted();
		});

		return () => {
			sub.remove();
			trackSub.remove();
		};
	}, [playlist, core]);

	const isBuffering = uiState?.status === PlayerStatus.Buffering;

	return (
		<MusicbirbContext.Provider
			value={{
				uiState,
				playlistStatus,
				core,
				isBuffering,

				queueTrack: async (id) => {
					await core?.queueTrack(id);
				},
				queueAlbum: async (id) => core?.queueAlbum(id) ?? 0,
				queuePlaylist: async (id) => core?.queuePlaylist(id) ?? 0,

				playTrack: async (id) => {
					await core?.playTrack(id);
				},
				playAlbum: async (id) => core?.playAlbum(id) ?? 0,
				playPlaylist: async (id) => core?.playPlaylist(id) ?? 0,

				clearQueue: () => core?.clearQueue(),
				removeIndex: (idx) => core?.removeIndex(idx),

				togglePause: () => core?.togglePause(),
				next: () => core?.next(),
				prev: () => core?.prev(),
				seek: (s) => core?.seek(s),
				playIndex: (idx) => core?.playIndex(idx),
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

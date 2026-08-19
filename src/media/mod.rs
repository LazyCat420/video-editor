pub mod ffmpeg_locator;
pub mod frame_cache;
pub mod peak_extractor;
pub mod probe;
pub mod proxy_generator;
pub mod stream_player;
pub mod thumbnail;
pub mod title_card_gen;
pub mod transition_blend;

pub use ffmpeg_locator::{find_ffmpeg_executable, find_ffprobe_executable};
pub use frame_cache::FrameCache;
pub use peak_extractor::{extract_peaks, WaveformPeaks};
pub use probe::{probe_media_file, MediaMetadata};
pub use proxy_generator::{generate_proxy_async, ProxyStatus};
pub use stream_player::StreamVideoPlayer;
pub use thumbnail::extract_thumbnail;
pub use title_card_gen::{generate_solid_color_frame, generate_title_card_frame};
pub use transition_blend::{blend_fade_in, blend_transition};

pub struct CrossfadeConfig {

}

pub struct SoundEnhancer {

}

pub struct ImportEncoderSettings {

}

pub struct IllTunesSettings {
    crossfade_config:        CrossfadeConfig,
    sound_enhancer:          SoundEnhancer,
    sound_check:             bool,
    media_locations:         Vec<PathBuf>,
    import_encoder_settings: ImportEncoderSettings,
}

pub trait ShowDuplicateItems {

    fn show_duplicate_items(&self) -> Result<(),IllTunesError>;
}

impl ShowDuplicateItems for IllTunes {

    fn show_duplicate_items(&self) -> Result<(),IllTunesError> {
        todo!();
    }
}

pub trait NewPlaylist {
    fn new_playlist(&self, playlist_name: &str) -> Result<(),IllTunesError>;
}

impl NewPlaylist for IllTunes {
    fn new_playlist(&self, playlist_name: &str) -> Result<(),IllTunesError> {
        todo!();
    }
}

pub trait AddTrack {
    fn add_track(&self, track: &Track) -> Result<(),IllTunesError>;
}

impl AddTrack for Playlist {

    fn add_track(&self, track: &Track) -> Result<(),IllTunesError> {
        todo!();
    }
}

pub trait AddToPlaylist {
    fn add_to_playlist(&self, playlist: &Playlist) -> Result<(),IllTunesError>;
}

impl AddToPlaylist for Track {
    fn add_to_playlist(&self, playlist: &Playlist) -> Result<(),IllTunesError> {
        todo!();
    }
}

pub trait PlayNext {
    fn play_next(&self) -> Result<(),IllTunesError>;
}

impl<P:Playable> PlayNext for Playable {
    fn play_next(&self) -> Result<(),IllTunesError> {
        todo!();
    }
}

pub trait PrependToPlayQueue {
    fn prepend_to_play_queue(&self) -> Result<(),IllTunesError>;
}

impl<P:Playable> PrependToPlayQueue for P {
    fn prepend_to_play_queue(&self) -> Result<(),IllTunesError> {
        todo!();
    }
}

pub trait AppendToPlayQueue {
    fn append_to_play_queue(&self) -> Result<(),IllTunesError>;
}

impl<P:Playable> AppendToPlayQueue for P {
    fn append_to_play_queue(&self) -> Result<(),IllTunesError> {
        todo!();
    }
}

pub trait BeginSubPlayQueue {
    fn begin_sub_play_queue(&self) -> Result<(),IllTunesError>;
}

pub trait FinalizeSubPlayQueue {
    fn finalize_sub_play_queue(&self) -> Result<(),IllTunesError>;
}

pub trait AppendToCurrentSubPlayQueue {
    fn append_to_current_sub_play_queue(&self) -> Result<(),IllTunesError>;
}

pub trait PrependToCurrentSubPlayQueue {
    fn prepend_to_current_sub_play_queue(&self) -> Result<(),IllTunesError>;
}

pub trait ShuffleCurrentSubPlayQueue {
    fn shuffle_current_sub_play_queue(&self) -> Result<(),IllTunesError>;
}

pub trait SaveCurrentSubPlayQueueToNewPlaylist {
    fn save_current_sub_play_queue_to_new_playlist(&self) -> Result<(),IllTunesError>;
}

pub trait TagCurrentSubPlayQueueWithNewAttribute {
    fn tag_current_sub_play_queue_with_new_attribute(&self) -> Result<(),IllTunesError>;
}

pub trait DisplayCurrentSubPlayQueueAttributes {
    fn display_current_sub_play_queue_attributes(&self) -> Result<(),IllTunesError>;
}

pub trait ClearCurrentSubPlayQueueAttributes {
    fn clear_current_sub_play_queue_attributs(&self) -> Result<(),IllTunesError>;
}

pub trait DisplayCurrentSubPlayQueueStats {
    fn display_current_sub_play_queue_stats(&self) -> Result<(),IllTunesError>;
}

pub trait ReverseCurrentSubPlayQueue {
    fn reverse_current_sub_play_queue(&self) -> Result<(),IllTunesError>;
}

pub trait SortCurrentSubPlayQueueByLengthAscending {
    fn sort_current_sub_play_queue_by_length_ascending(&self) -> Result<(),IllTunesError>;
}

pub trait SortCurrentSubPlayQueueByLengthDescending {
    fn sort_current_sub_play_queue_by_length_descending(&self) -> Result<(),IllTunesError>;
}

pub trait GroupCurrentSubPlayQueueByAttributes {
    fn group_current_sub_play_queue_by_attributes(&self) -> Result<(),IllTunesError>;
}

pub trait GroupCurrentSubPlayQueueByArtist {
    fn group_current_sub_play_queue_by_artist(&self) -> Result<(),IllTunesError>;
}

pub trait GroupCurrentSubPlayQueueByKey {
    fn group_current_sub_play_queue_by_key(&self) -> Result<(),IllTunesError>;
}

pub trait GroupCurrentSubPlayQueueByKeyAndArtist {
    fn group_current_sub_play_queue_by_key_and_artist(&self) -> Result<(),IllTunesError>;
}

pub trait MoveCurrentPlayQueueForward {
    fn move_current_play_queue_forward(&self) -> Result<(),IllTunesError>;
}

pub trait MoveCurrentPlayQueueBackward {
    fn move_current_play_queue_backward(&self) -> Result<(),IllTunesError>;
}

pub trait GetSongInfo {
    fn get_song_info(&self) -> Result<(),IllTunesError>;
}

pub trait MarkSongAsFavorite {
    fn mark_song_as_favorite(&self) -> Result<(),IllTunesError>;
}

pub trait MarkSongAsDisliked {
    fn mark_song_as_disliked(&self) -> Result<(),IllTunesError>;
}

pub trait ShowAlbum {
    fn show_album(&self) -> Result<(),IllTunesError>;
}

pub trait ShowPlaylists {
    fn show_playlists(&self) -> Result<(),IllTunesError>;
}

pub trait RemoveFromPlaylist {
    fn remove_from_playlist(&self) -> Result<(),IllTunesError>;
}

pub trait DeleteFromLibrary {
    fn delete_from_library(&self) -> Result<(),IllTunesError>;
}

pub struct Equalizer {

}

pub enum SortBy {
    PlaylistOrder,
    Album,
    Artist,
    Favorite,
    Genre,
    Plays,
    Time,
    Title,
}

pub enum SortOrder {
    Ascending,
    Descending,
}

pub enum ViewStyle {
    AsPlaylist,
    AsSongs,
    AsAlbums,
    AsArtists,
    AsComposers,
    AsGenres
}

pub enum FilterBy {
    AllSongs,
    OnlyFavorites,
}

pub struct FilterApplicator {
    filter_text: String,
}

pub struct PlaybackHistory {
    items: Vec<dyn Playable>,
}

pub struct PlayingNext {
    items: Vec<dyn Playable>,
}

use tracing::{debug, info, trace};

/// Defines how the user wants to view their Music interface (formerly a dropdown).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewAs {
    Playlist,
    Songs,
    Albums,
    Artists,
    Composers,
    Genres,
    Unknown,
}

/// Defines how the user wants to sort their Music interface (formerly another dropdown).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortBy {
    PlaylistOrder,
    Title,
    Artist,
    Album,
    Genre,
    Unknown,
}

/// A single, “flattened” struct representing all available toggles and options
/// in the Music “View Options” interface (no subheadings).
#[derive(Debug, Clone, PartialEq)]
pub struct MusicViewOptions {
    album:                bool,
    album_artist:         bool,
    artist:               bool,
    beats_per_minute:     bool,
    composer:             bool,
    disc_number:          bool,
    equalizer:            bool,
    genre:                bool,
    movement_name:        bool,
    movement_number:      bool,
    release_date:         bool,
    time:                 bool,
    track_number:         bool,
    work:                 bool,
    year:                 bool,

    album_rating:         bool,
    favorite:             bool,
    comments:             bool,
    description:          bool,
    grouping:             bool,
    rating:               bool,

    date_added:           bool,
    date_modified:        bool,
    last_played:          bool,
    last_skipped:         bool,
    plays:                bool,
    purchase_date:        bool,
    skips:                bool,

    bit_rate:             bool,
    kind:                 bool,
    sample_rate:          bool,
    size:                 bool,

    sort_album:           bool,
    sort_album_artist:    bool,
    sort_artist:          bool,
    sort_composer:        bool,
    sort_title:           bool,

    category:             bool,

    /// Reflects the user’s selection for “View As.”
    view_as:              ViewAs,
    /// Reflects the user’s selection for “Sort by.”
    sort_by:              SortBy,
    /// Checkboxes controlling whether artwork is shown and always shown.
    show_artwork:         bool,
    always_show:          bool,

    /// Slider-based artwork size, e.g. from 0..100 or similar.
    artwork_size:         u8,
}

pub enum BasicControls {
    Pause,
    Stop,
    NextTrack,
    PreviousTrack,
    GeniusShuffle,
    GoToCurrentSong,
    IncreaseVolume,
    DecreaseVolume,
    TurnShuffleOn,
    TurnShuffleOff,
    SetShuffleModeSongs,
    SetShuffleModeAlbums,
    SetShuffleModeGroupings,
    SetRepeatModeOff,
    SetRepeatModeAll,
    SetRepeatModeOne,
    ToggleVisualizer,
}

crate::ix!();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    StartPlayingOrPauseTheSelectedSong,
    PlayTheCurrentlySelectedSongFromTheBeginning,
    MoveForwardWithinASong,
    MoveBackwardWithinASong,
    StopPlayingTheSelectedSong,
    WhenASongIsPlayingPlayTheNextSongInAList,
    WhenASongIsPlayingPlayThePreviousSongInAList,
    ShowTheCurrentlyPlayingSongInTheList,
    ShowThePlayingNextQueue,
    ListenToTheNextAlbumInAList,
    ListenToThePreviousAlbumInAList,
    IncreaseTheVolume,
    DecreaseTheVolume,
    OpenTheEqualizer,
    GoToTheNextChapterIfAvailable,
    GoToTheLastChapterIfAvailable,
    StreamAudioFileAtASpecificURLToMusic,
    CreateANewPlaylist,
    CreateAPlaylistFromASelectionOfSongs,
    CreateANewSmartPlaylist,
    StartGeniusShuffle,
    RefreshAGeniusPlaylistWhenThePlaylistIsSelected,
    DeleteTheSelectedSongFromThePlaylistOrLibrary,
    DeleteTheSelectedSongFromThePlaylistOrLibraryWithoutConfirming,
    DeleteTheSelectedSongFromYourLibraryAndAllPlaylists,
    AddAFileImportToYourLibrary,
    ShowWhereASongFileIsLocated,
    SelectTheSearchField,
    UndoYourLastTypingChangeWhileEditingAnItemsInformation,
    CutTheSelectedSongsInformationOrArtwork,
    CopyTheSelectedSongsInformationOrArtwork,
    PasteTheSelectedSongsInformationOrArtwork,
    SelectAllTheSongsInTheList,
    ShowOrHideTheColumnBrowser,
    DeselectAllTheSongsInTheList,
    SelectOrDeselectAllTheSongsInAList,
    OpenMiniPlayer,
    CloseMiniPlayer,
    OpenFullScreenPlayer,
    CloseFullScreenPlayer,
    SwitchBetweenCustomAndMaximumWindowSizes,
    ChangeTheSongInformationColumns,
    ShowOrHideTheStatusBar,
    OpenTheInfoWindowForTheSelectedSong,
    ShowTheInformationForTheNextSongInTheList,
    ShowTheInformationForThePreviousSongInTheList,
    OpenTheViewOptionsWindowForTheSelectedSource,
    TurnTheVisualizerOn,
    TurnTheVisualizerOff,
    SeeMoreOptionsWhenAVisualEffectIsShowing,
    RefreshAppleMusicOrITunesStore,
    OpenTheMusicWindow,
    CloseTheMusicWindow,
    PutTheMusicWindowInTheDock,
    HideTheMusicWindow,
    HideAllOtherApplications,
    InitiateASearchInTheITunesStoreFromAnywhereInMusic,
    GoToTheNextPageInTheITunesStore,
    GoToThePreviousPageInTheITunesStore,
    ReloadTheCurrentPage,
    OpenMusicSettings,
    QuitMusic,
    EjectACD,
    OpenMusicHelpMenu,
    OpenADifferentMusicLibrary,
}

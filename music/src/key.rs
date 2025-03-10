crate::ix!();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    // Letters
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    // Digits
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,

    // Modifiers
    Command,
    Control,
    Option,
    Shift,
    CapsLock,
    Fn, // Function modifier key on some keyboards

    // Whitespace / Special
    SpaceBar,
    Tab,
    Return,
    Delete,
    Backspace,
    Esc,

    // Arrow keys
    LeftArrow,
    RightArrow,
    UpArrow,
    DownArrow,

    // Brackets
    LeftBracket,
    RightBracket,

    // Common punctuation
    Comma,
    Period,
    Slash,
    Backslash,
    Semicolon,
    Apostrophe,
    Minus,
    Equals,
    Backquote,
    QuestionMark,

    // Function keys
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,

    // Navigation / Misc
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
    PrintScreen,
    ScrollLock,
    Pause,
    Menu,
}

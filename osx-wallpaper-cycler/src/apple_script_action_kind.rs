// ---------------- [ File: osx-wallpaper-cycler/src/apple_script_action_kind.rs ]
crate::ix!();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppleScriptActionKind {
    CountDesktops,
    SetDesktopPictures,
    SetCurrentDesktopPicture,
    SwitchSpaceLeft,
    SwitchSpaceRight,
}

impl std::fmt::Display for AppleScriptActionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CountDesktops => write!(f, "applescript.count_desktops"),
            Self::SetDesktopPictures => write!(f, "applescript.set_desktop_pictures"),
            Self::SetCurrentDesktopPicture => write!(f, "applescript.set_current_desktop_picture"),
            Self::SwitchSpaceLeft => write!(f, "applescript.switch_space_left"),
            Self::SwitchSpaceRight => write!(f, "applescript.switch_space_right"),
        }
    }
}

#[cfg(test)]
mod apple_script_action_kind_display_contract_suite {
    use super::*;

    #[traced_test]
    fn display_is_stable_and_unique_across_variants() {
        let a = AppleScriptActionKind::CountDesktops.to_string();
        let b = AppleScriptActionKind::SetDesktopPictures.to_string();
        let c = AppleScriptActionKind::SetCurrentDesktopPicture.to_string();
        let d = AppleScriptActionKind::SwitchSpaceLeft.to_string();
        let e = AppleScriptActionKind::SwitchSpaceRight.to_string();

        assert_eq!(a, "applescript.count_desktops");
        assert_eq!(b, "applescript.set_desktop_pictures");
        assert_eq!(c, "applescript.set_current_desktop_picture");
        assert_eq!(d, "applescript.switch_space_left");
        assert_eq!(e, "applescript.switch_space_right");

        let set: std::collections::HashSet<String> = [a, b, c, d, e].into_iter().collect();
        assert_eq!(set.len(), 5);
    }
}

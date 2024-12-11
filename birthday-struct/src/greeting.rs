crate::ix!();

/// Returns the birthday greeting for the given language.
///
pub fn birthday_greeting(language: &Language) 
    -> Option<&'static str> 
{
    match language {
        Language::English    => Some("Happy Birthday!"),
        Language::Spanish    => Some("Feliz Cumpleaños!"),
        Language::French     => Some("Joyeux Anniversaire!"),
        Language::German     => Some("Alles Gute zum Geburtstag!"),
        Language::Mandarin   => Some("生日快乐！"),
        Language::Japanese   => Some("お誕生日おめでとうございます！"),
        Language::Russian    => Some("С днём рождения!"),
        Language::Italian    => Some("Buon Compleanno!"),
        Language::Portuguese => Some("Feliz Aniversário!"),
        Language::Arabic     => Some("عيد ميلاد سعيد!"),
        Language::Dutch      => Some("Fijne Verjaardag!"),
        Language::Greek      => Some("Χαρούμενα Γενέθλια!"),
        Language::Hindi      => Some("जन्मदिन मुबारक हो!"),
        _ => None,
    }
}

crate::ix!();

/// Enum representing different forms of structured language.
pub enum StructuredLanguageForm {
    AlcaicStanza,            // 4 lines, classical meter
    Ballad,                  // Quatrains, ABAB or ABCB
    BlankVerse,              // Unrhymed iambic pentameter
    BurnsStanza,            // Six-line stanzas with aaabab rhyme scheme, also known as the Standard Habbie
    Canso,                  // Troubadour song form, often expressing courtly love
    Canzone,                // Italian or Provençal song form
    Cinquain,                // 5 lines, varying syllable patterns
    Clerihew,                // Humorous, biographical, AABB rhyme
    Couplet,                // Two lines of verse, usually in the same meter and joined by rhyme
    Diamante,                // Diamond-shaped, 7 lines
    Ekphrastic,              // Inspired by visual art
    Elegy,                   // Reflective, often mourning
    Epic,                    // Long narrative, often heroic
    Epigram,                 // Short, witty, often satirical
    Epithalamium,            // Poem for a wedding
    Fib,                     // Based on Fibonacci sequence, syllables 1, 1, 2, 3, 5, 8...
    FreeVerse,               // No regular meter or rhyme scheme
    Ghazal,                  // Couplets, repeated rhyme, refrain
    Glosa,                  // Iberian form elaborating on a quatrain by another poet
    Haiku,                   // 3 lines, 5-7-5 syllables
    HoratianOde,             // Regular stanzas, reflective
    Kyrielle,                // Quatrains with a refrain
    Lai,                    // Medieval French form with varying line lengths
    Limerick,                // 5 lines, AABBA
    Lune,                    // Haiku variant, 3-5-3 words
    Madrigal,               // Short, lyrical poems of the Renaissance, often set to music
    Naani,                   // 4 lines, 20-25 syllables
    Nonet,                   // 9 lines, starting with 9 syllables, decreasing by one each line
    Octave,                 // An eight-line stanza or the first eight lines of a sonnet
    Ode,                     // Lyrical, often praising
    Pantoum,                 // Quatrains, with repeated lines
    PetrarchanSonnet,        // 14 lines, ABBAABBACDECDE
    PindaricOde,             // Strophes, antistrophes, epodes
    Quatrain,               // A stanza of four lines, often with a specific rhyme scheme
    Quintain,               // A stanza of five lines with varied meter and rhyme
    Rondeau,                 // 15 lines, aabba aabR aabbaR
    Rondel,                 // A verse form originating in French lyrical poetry, with 13 or 14 lines and two rhymes
    Rondelet,                // 7 lines, A repeated refrain
    Roundel,                 // 11 lines, ababB abaBab abB
    Rubaiyat,                // Quatrains, aaba
    Senryu,                  // Similar to haiku, but focuses on human nature
    Septet,                 // A stanza of seven lines
    Sestet,                 // The last six lines of a sonnet or a stanza of six lines
    Sestina,                 // 39 lines, six sestets and a tercet
    ShakespeareanSonnet,     // 14 lines, ABABCDCDEFEFGG
    Sonnet,                 // 14-line poems with a specific rhyme scheme, often about love
    Sonnetina,               // Shortened sonnet, 10 or 12 lines
    SpenserianSonnet,        // 14 lines, ABABBCBCCDCDEE
    Tanka,                   // 5 lines, 5-7-5-7-7 syllables
    TerzaRima,               // Tercets, aba bcb cdc...
    ThanBauk,                // Burmese form, three lines, rhyming syllables
    Triolet,                 // 8 lines, ABaAabAB
    Tyburn,                  // 6 lines, first four are two-syllable descriptive words
    Villancico,             // Spanish and Portuguese form, with a refrain
    Villanelle,              // 19 lines, ABA ABA ABA ABA ABA ABAA
    Virelai,                // Medieval French form with multiple stanzas and refrains
}


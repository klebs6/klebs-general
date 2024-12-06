use ai_descriptor::*;
use rand_construct::*;
use std::borrow::Cow;

#[derive(Plural,Default,RandConstruct,ItemFeature,PartialEq,Eq,Hash,Copy,Clone,Debug)]
pub enum StructuredLanguageForm {
    #[ai("a poem structured in four lines with classical quantitative meter, typically dactylic or trochaic, evoking ancient Greek lyric tradition")]
    AlcaicStanza,

    #[ai("a narrative poem composed of quatrains with alternating rhyme schemes (ABAB or ABCB), often recounting dramatic, folkloric, or romantic tales in a rhythmic and melodic style")]
    Ballad,

    #[ai("a poem written in unrhymed iambic pentameter, characterized by its regular rhythmic pattern and flexibility for elevated, dramatic, or reflective themes")]
    BlankVerse,

    #[ai("a poem with six-line stanzas, adhering to the traditional Scottish form known as the Standard Habbie, with a rhyme scheme of aaabab; often humorous or reflective in tone")]
    BurnsStanza,

    #[ai("a lyrical composition in the troubadour tradition, frequently written in strophes and exploring themes of courtly love, idealized devotion, or chivalric ethos")]
    Canso,

    #[ai("a sophisticated Italian or Provençal song form, characterized by intricate stanza structures, rich in musicality and often exploring themes of love or philosophy")]
    Canzone,

    #[ai("a compact five-line poem with varied syllable patterns, emphasizing vivid imagery or focused emotional expression")]
    Cinquain,

    #[ai("a whimsical biographical poem of four lines with an AABB rhyme scheme, often humorous or satirical, focusing on an individual's peculiar traits or actions")]
    Clerihew,

    #[ai("a concise poetic form consisting of two rhyming lines, often used to distill meaning or emphasize contrast through parallelism or juxtaposition")]
    Couplet,

    #[ai("a seven-line, diamond-shaped poem with a fixed structure that visually mirrors its central theme, often employing descriptive or contrasting imagery")]
    Diamante,

    #[ai("a vivid poetic form inspired by or describing a specific piece of visual art, aiming to translate its essence or emotional resonance into language")]
    Ekphrastic,

    #[ai("a reflective and solemn poem typically mourning a loss or meditating on mortality, written in a somber yet elevated tone")]
    Elegy,

    #[default]
    #[ai("an extended narrative poem of significant length, often heroic or mythological in subject, combining elevated diction and grand themes to celebrate cultural values or legendary figures")]
    Epic,

    #[ai("a succinct and witty poem, typically satirical or aphoristic, aimed at delivering a sharp or memorable observation in a few lines")]
    Epigram,

    #[ai("a ceremonial poem written to celebrate and honor a wedding, blending lyrical beauty with themes of love, union, and festivity")]
    Epithalamium,

    #[ai("a minimalist poetic form using syllabic progression based on the Fibonacci sequence (1, 1, 2, 3, 5, 8...), creating a spiraling structure of brevity and growth")]
    Fib,

    #[ai("a poem without a fixed rhyme scheme or meter, allowing unrestricted exploration of rhythm, imagery, and meaning")]
    FreeVerse,

    #[ai("a poetic form of rhyming couplets with repeated phrases and refrains, traditionally emphasizing themes of love, loss, or spiritual longing")]
    Ghazal,

    #[ai("a reflective Iberian verse form elaborating on a quatrain from another poet, blending commentary with creative extension in a uniquely interpretative structure")]
    Glosa,

    #[ai("a concise three-line poem adhering to a syllabic structure of 5-7-5, traditionally focusing on nature or a moment of epiphany, rooted in Japanese aesthetics")]
    Haiku,

    #[ai("a contemplative ode written in regular, balanced stanzas, named for the Roman poet Horace and characterized by its reflective and restrained tone")]
    HoratianOde,

    #[ai("a poetic form composed of quatrains, with a refrain repeated at the end of each stanza, often exploring spiritual or meditative themes")]
    Kyrielle,

    #[ai("a medieval French poetic form characterized by varying line lengths and intricate rhyme schemes, blending lyricism with narrative complexity")]
    Lai,

    #[ai("a humorous five-line poem with a distinct AABBA rhyme scheme, employing a sing-song rhythm to deliver witty or absurd narratives")]
    Limerick,

    #[ai("a haiku-like poetic form structured around three lines of 3-5-3 words rather than syllables, often exploring concise and evocative imagery")]
    Lune,

    #[ai("a short, lyrical Renaissance poem often set to music, celebrated for its intricate wordplay and themes of love, beauty, and harmony")]
    Madrigal,

    #[ai("a compact four-line poetic form, typically ranging from 20-25 syllables, exploring themes of daily life or philosophical reflections in a straightforward style")]
    Naani,

    #[ai("a nine-line poem beginning with nine syllables and decreasing by one syllable per line, creating a tapering effect that mirrors descent or resolution")]
    Nonet,

    #[ai("an eight-line stanza or the opening section of a sonnet, characterized by thematic setup or argument, often with an ABBAABBA rhyme scheme")]
    Octave,

    #[ai("a formal, lyrical poem that expresses deep admiration or reverence for its subject, blending emotional intensity with elevated diction")]
    Ode,

    #[ai("a complex poetic form of quatrains with repeated lines, using a weave-like structure to emphasize cyclical or reflective themes")]
    Pantoum,

    #[ai("a fourteen-line sonnet with an ABBAABBACDECDE rhyme scheme, traditionally associated with themes of unrequited love or idealized beauty")]
    PetrarchanSonnet,

    #[ai("a classical ode structured in strophes, antistrophes, and epodes, imitating the choral odes of ancient Greek drama to celebrate triumphs or heroes")]
    PindaricOde,

    #[ai("a stanza of four lines often adhering to a consistent rhyme scheme, serving as the foundational unit of many poetic traditions")]
    Quatrain,

    #[ai("a stanza of five lines, typically employing a varied rhyme scheme and meter, blending structural freedom with lyrical expression")]
    Quintain,

    #[ai("a fifteen-line poem with a refrain and intricate rhyme scheme (aabba aabR aabbaR), conveying layered themes or emotional echoes")]
    Rondeau,

    #[ai("a lyrical French poetic form of thirteen or fourteen lines with two rhymes, balancing musicality and formal precision to evoke timeless themes")]
    Rondel,

    #[ai("a seven-line poem with a refrain, blending lyrical brevity with a dance-like rhythm through its A repeated refrain structure")]
    Rondelet,

    #[ai("an eleven-line poetic form with a refrain and intricate rhyme scheme (ababB abaBab abB), interweaving repetition and variation")]
    Roundel,

    #[ai("a quatrain-driven poetic form with a rhyme scheme of AABA, often used for meditative or mystical reflections, rooted in Persian tradition")]
    Rubaiyat,

    #[ai("a short three-line poem structurally similar to haiku, but focusing on human foibles, wit, or satire, with a syllabic pattern of 5-7-5")]
    Senryu,

    #[ai("a stanza of seven lines with varied rhyme and meter, serving as a versatile structure for narrative or descriptive poetry")]
    Septet,

    #[ai("a stanza or poem of six lines, often used as the concluding section of a sonnet to resolve or reflect upon its thematic arc")]
    Sestet,

    #[ai("a complex poetic form of thirty-nine lines, comprising six sestets and a tercet, with a fixed pattern of word repetition at the line endings")]
    Sestina,

    #[ai("a fourteen-line poem with the ABABCDCDEFEFGG rhyme scheme, often exploring themes of love, conflict, or introspection")]
    ShakespeareanSonnet,

    #[ai("a highly formalized poetic form of fourteen lines with a wide variety of rhyme schemes, often centered on love or philosophical themes")]
    Sonnet,

    #[ai("a truncated sonnet form with ten or twelve lines, offering a condensed exploration of the traditional sonnet’s thematic intensity")]
    Sonnetina,

    #[ai("a Spenserian sonnet composed of fourteen lines with a unique ABABBCBCCDCDEE rhyme scheme, blending lyricism with structural unity")]
    SpenserianSonnet,

    #[ai("a traditional Japanese form of five lines with a syllabic pattern of 5-7-5-7-7, evoking deep emotion and natural imagery")]
    Tanka,

    #[ai("a tercet-based poetic form with an interlocking rhyme scheme (aba bcb cdc...), allowing a flowing and interconnected progression of themes")]
    TerzaRima,

    #[ai("a Burmese poetic form of three lines with rhyming syllables and compressed themes, blending brevity with linguistic playfulness")]
    ThanBauk,

    #[ai("an eight-line poem with a refrain and a fixed ABaAabAB rhyme scheme, emphasizing lyrical repetition and melodic harmony")]
    Triolet,

    #[ai("a six-line poem where the first four lines consist of two-syllable descriptive words, leading to a vivid central theme")]
    Tyburn,

    #[ai("a lyrical Spanish or Portuguese poetic form with a refrain, often celebrating cultural or religious themes with a song-like rhythm")]
    Villancico,

    #[ai("a nineteen-line poem with five tercets and a concluding quatrain, employing the refrain-based ABA ABA ABA ABA ABA ABAA rhyme scheme")]
    Villanelle,

    #[ai("a medieval French poetic form with multiple stanzas and refrains, blending musicality with narrative intricacy")]
    Virelai,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plural() {
        let f = StructuredLanguageForm::random();
        println!("{}", f.plural());
    }
}

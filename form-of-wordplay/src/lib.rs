use ai_descriptor::*;
use rand_construct::*;
use std::borrow::Cow;

/// Enum representing different forms of wordplay.
#[derive(Plural,Default,RandConstruct,ItemFeature,PartialEq,Eq,Hash,Copy,Clone,Debug)]
pub enum FormOfWordplay {
    #[ai("a densely layered joke or passage exploiting multiple meanings of words or homophones, with intricate wit and interwoven semantic nuances")]
    Pun,

    #[ai("a poetic or prose composition saturated with the repetition of initial consonant sounds, creating a rhythmic, almost hypnotic effect")]
    Alliteration,

    #[ai("a lyrical text or speech imbued with flowing repetitions of vowel sounds, producing an almost musical resonance across phrases")]
    Assonance,

    #[ai("a compact piece of text featuring abundant repetition of consonant sounds across words, interlocking the auditory texture of the composition")]
    Consonance,

    #[ai("a poem or sentence crafted entirely as a palindrome, with complete semantic coherence forward and backward, achieving both structural symmetry and layered meaning")]
    Palindrome,

    #[ai("a riddle or puzzle formed entirely of interrelated anagrams, weaving intricate connections between multiple rearranged words or phrases")]
    Anagram,

    #[ai("a humorous poem or dialogue packed with frequent and deliberate spoonerisms, showcasing the interplay of sound transpositions across sentences")]
    Spoonerism,

    #[ai("a satirical or poetic composition filled with tightly juxtaposed oxymorons, creating an evocative tension between conflicting ideas")]
    Oxymoron,

    #[ai("a dictionary of newly coined portmanteau words, each blending the phonetic and semantic essence of its components into an ingenious hybrid")]
    Portmanteau,

    #[ai("a comedic monologue or scene laced with frequent malapropisms, blending mistaken substitutions for soundalike words with subtle layers of humor or irony")]
    Malapropism,

    #[ai("a crossword or text designed to maximize the use of homophones, creating a playful or deceptive interplay of sound-alike words with distinct meanings")]
    Homophone,

    #[ai("a poem or story showcasing the creative interplay of homonyms, embedding words with identical forms but vastly different meanings across a single narrative")]
    Homonym,

    #[ai("a dialogue or narrative densely packed with paronomasia, employing clever wordplays that pivot on similar-sounding terms to generate humor or layered interpretation")]
    Paronomasia,

    #[ai("a novel or essay written as an elaborate lipogram, omitting specific letters entirely while maintaining eloquence and syntactic complexity")]
    Lipogram,

    #[ai("a creative text composed entirely of pangrams, with each sentence including every letter of the alphabet and demonstrating linguistic precision")]
    Pangram,

    #[ai("a poem or story composed entirely as a tautogram, where every word begins with the same letter while maintaining grammatical coherence and richness")]
    Tautogram,

    #[ai("a poem or puzzle where the initial letters of each line spell out a concealed word or phrase, blending artistic flair with encoded meaning")]
    Acrostic,

    #[ai("a humorous or rhetorical backronym generator, producing phrases that ingeniously reinterpret existing acronyms into layered or ironic expressions")]
    Backronym,

    #[ai("a speech or poem demonstrating antimetabole, where clauses repeat words in reversed order to emphasize symmetry and rhetorical impact")]
    Antimetabole,

    #[ai("a literary piece heavily utilizing chiasmus, interweaving inverted structures to create a sense of balance, contrast, and poetic elegance")]
    Chiasmus,

    #[ai("a whimsical collection of mondegreens, each creatively mishearing familiar phrases to produce unexpected or humorous reinterpretations")]
    Mondegreen,

    #[ai("a series of witty Tom Swifty sentences where the quoted speech and its attribution intertwine through advanced puns and double entendres")]
    TomSwifty,

    #[ai("a story or essay employing zeugma extensively, where single words govern multiple meanings, crafting dense and layered prose")]
    Zeugma,

    #[ai("a dictionary or collection featuring examples of aphaeresis, showcasing words shortened at their beginnings for poetic or linguistic effect")]
    Aphaeresis,

    #[ai("a poetic text showcasing apocope through numerous words truncated at their endings, creating a brisk and rhythmically altered flow")]
    Apocope,

    #[ai("a playful or formal text using syncope extensively, demonstrating linguistic evolution or phonetic elision within the heart of words")]
    Syncope,

    #[ai("a circular poem or statement employing epanalepsis, where the repetition of the opening and closing words creates a reflective resonance")]
    Epanalepsis,

    #[ai("a story or essay where anadiplosis is artfully repeated, each clause beginning with the ending of the preceding one, forming a chain of thought")]
    Anadiplosis,

    #[ai("a poem or speech heavily employing epizeuxis, where words are repeated in immediate succession for dramatic or emphatic effect")]
    Epizeuxis,

    #[ai("a text composed with extensive polyptoton, showcasing the creative and semantic versatility of root words through different grammatical forms")]
    Polyptoton,

    #[ai("a playful or poetic composition relying on frequent metathesis, where transpositions of sounds or letters create novelty and linguistic play")]
    Metathesis,

    #[ai("a layered text making liberal use of ploce, where the same word recurs in varied senses, enriching its narrative or rhetorical depth")]
    Ploce,

    #[ai("a collection of paraprosdokian sentences or anecdotes, where unexpected twists in phrasing redefine the meaning or impact of each passage")]
    Paraprosdokian,

    #[ai("a rhetorical or literary speech where paradiastole reinterprets negative terms positively, demonstrating the persuasive power of euphemism")]
    Paradiastole,

    #[ai("a poetic or prose work weaving polysemy into its language, where every phrase brims with multiple, layered meanings")]
    #[default]
    Polysemy,

    #[ai("a catalog of retronyms documenting the evolution of terms to differentiate original concepts from modern variants, showcasing linguistic adaptability")]
    Retronym,

    #[ai("a creative dictionary of neologisms, featuring newly coined words and expressions with advanced etymological and cultural significance")]
    Neologism,

    #[ai("a dialogue or narrative steeped in antanaclasis, where words are repeated with different meanings to create wit and rhetorical power")]
    Antanaclasis,

    #[ai("a literary piece demonstrating anthimeria extensively, where nouns become verbs or other shifts occur to innovate linguistic expression")]
    Anthimeria,

    #[ai("a poetic or dramatic text where apostrophe addresses abstract ideas, inanimate objects, or absent entities with vivid emotional resonance")]
    Apostrophe,

    #[ai("a speech or essay employing auxesis, where words ascend in intensity or significance, building to a climactic rhetorical impact")]
    Auxesis,

    #[ai("a poem or essay densely employing catachresis, creating inventive, unexpected, or mixed metaphors for linguistic and conceptual impact")]
    Catachresis,

    #[ai("a text infused with diacope, where words or phrases recur with deliberate spacing, producing rhythmic and emphatic patterns")]
    Diacope,

    #[ai("a speech or composition where epanorthosis is used frequently, each correction amplifying clarity or rhetorical force")]
    Epanorthosis,

    #[ai("a descriptive text rich with epithets, where vivid and characteristic phrases evoke powerful imagery and thematic resonance")]
    Epithet,

    #[ai("a poetic composition employing hyperbaton, rearranging normal syntax to emphasize key ideas or create a strikingly poetic effect")]
    Hyperbaton,

    #[ai("a text demonstrating hysteron proteron, reversing logical or temporal order to create rhetorical surprise or artistic focus")]
    HysteronProteron,

    #[ai("a balanced speech or text where isocolon is used extensively, with successive clauses mirroring length, rhythm, and structure")]
    Isocolon,

    #[ai("a series of statements employing litotes, where understatements achieve ironic, emphatic, or understated elegance")]
    Litotes,

    #[ai("a story or speech heavily using meiosis, where deliberate understatements heighten dramatic tension or humor")]
    Meiosis,

    #[ai("a speech or essay emphasizing points through paralepsis, subtly drawing attention by ostensibly passing over them")]
    Paralepsis,

    #[ai("a passage rich in parenthetical asides, where supplementary information adds depth or nuance to the primary text")]
    Parenthesis,

    #[ai("a poem or speech creatively employing synecdoche, using parts to represent wholes or vice versa for vivid imagery or symbolism")]
    Synecdoche,

    #[ai("a playful or poetic text using tmesis extensively, splitting compound words with intervening elements for humorous or rhetorical effect")]
    Tmesis,

    #[ai("a satirical or rhetorical text where antiphrasis conveys meanings opposite to the literal interpretation of its words")]
    Antiphrasis,

    #[ai("a composition filled with enallage, where unconventional grammatical substitutions add stylistic depth or nuance")]
    Enallage,

    #[ai("a poetic or rhetorical text rich with hypallage, where shifted associations between words create unusual or evocative imagery")]
    Hypallage,

    #[ai("a persuasive essay or speech using antithesis extensively, where juxtaposed opposites create striking contrasts and clarity")]
    Antithesis,
}

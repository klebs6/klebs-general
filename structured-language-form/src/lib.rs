use ai_descriptor::*;
use rand_construct::*;
use std::borrow::Cow;

/// Enum representing different forms of structured language.
#[derive(Plural,Default,RandConstruct,ItemFeature,PartialEq,Eq,Hash,Copy,Clone,Debug)]
pub enum StructuredLanguageForm {
    #[ai("4 lines, classical meter")]
    AlcaicStanza,            

    #[ai("Quatrains, ABAB or ABCB")]
    Ballad,                  

    #[ai("Unrhymed iambic pentameter")]
    BlankVerse,              

    #[ai("Six-line stanzas with aaabab rhyme scheme, also known as the Standard Habbie")]
    BurnsStanza,            

    #[ai("Troubadour song form, often expressing courtly love")]
    Canso,                  

    #[ai("Italian or Provençal song form")]
    Canzone,                

    #[ai("5 lines, varying syllable patterns")]
    Cinquain,                

    #[ai("Humorous, biographical, AABB rhyme")]
    Clerihew,                

    #[ai("Two lines of verse, usually in the same meter and joined by rhyme")]
    Couplet,                

    #[ai("Diamond-shaped, 7 lines")]
    Diamante,                

    #[ai("Inspired by visual art")]
    Ekphrastic,              

    #[ai("Reflective, often mourning")]
    Elegy,                   

    #[default]
    #[ai("Long narrative, often heroic")]
    Epic,                    

    #[ai("Short, witty, often satirical")]
    Epigram,                 

    #[ai("Poem for a wedding")]
    Epithalamium,            

    #[ai("Based on Fibonacci sequence, syllables 1, 1, 2, 3, 5, 8...")]
    Fib,                     

    #[ai("No regular meter or rhyme scheme")]
    FreeVerse,               

    #[ai("Couplets, repeated rhyme, refrain")]
    Ghazal,                  

    #[ai("Iberian form elaborating on a quatrain by another poet")]
    Glosa,                  

    #[ai("3 lines, 5-7-5 syllables")]
    Haiku,                   

    #[ai("Regular stanzas, reflective")]
    HoratianOde,             

    #[ai("Quatrains with a refrain")]
    Kyrielle,                

    #[ai("Medieval French form with varying line lengths")]
    Lai,                    

    #[ai("5 lines, AABBA")]
    Limerick,                

    #[ai("Haiku variant, 3-5-3 words")]
    Lune,                    

    #[ai("Short, lyrical poems of the Renaissance, often set to music")]
    Madrigal,               

    #[ai("4 lines, 20-25 syllables")]
    Naani,                   

    #[ai("9 lines, starting with 9 syllables, decreasing by one each line")]
    Nonet,                   

    #[ai("An eight-line stanza or the first eight lines of a sonnet")]
    Octave,                 

    #[ai("Lyrical, often praising")]
    Ode,                     

    #[ai("Quatrains, with repeated lines")]
    Pantoum,                 

    #[ai("14 lines, ABBAABBACDECDE")]
    PetrarchanSonnet,        

    #[ai("Strophes, antistrophes, epodes")]
    PindaricOde,             

    #[ai("A stanza of four lines, often with a specific rhyme scheme")]
    Quatrain,               

    #[ai("A stanza of five lines with varied meter and rhyme")]
    Quintain,               

    #[ai("15 lines, aabba aabR aabbaR")]
    Rondeau,                 

    #[ai("A verse form originating in French lyrical poetry, with 13 or 14 lines and two rhymes")]
    Rondel,                 

    #[ai("7 lines, A repeated refrain")]
    Rondelet,                

    #[ai("11 lines, ababB abaBab abB")]
    Roundel,                 

    #[ai("Quatrains, aaba")]
    Rubaiyat,                

    #[ai("Similar to haiku, but focuses on human nature")]
    Senryu,                  

    #[ai("A stanza of seven lines")]
    Septet,                 

    #[ai("The last six lines of a sonnet or a stanza of six lines")]
    Sestet,                 

    #[ai("39 lines, six sestets and a tercet")]
    Sestina,                 

    #[ai("14 lines, ABABCDCDEFEFGG")]
    ShakespeareanSonnet,     

    #[ai("14-line poems with a specific rhyme scheme, often about love")]
    Sonnet,                 

    #[ai("Shortened sonnet, 10 or 12 lines")]
    Sonnetina,               

    #[ai("14 lines, ABABBCBCCDCDEE")]
    SpenserianSonnet,        

    #[ai("5 lines, 5-7-5-7-7 syllables")]
    Tanka,                   

    #[ai("Tercets, aba bcb cdc...")]
    TerzaRima,               

    #[ai("Burmese form, three lines, rhyming syllables")]
    ThanBauk,                

    #[ai("8 lines, ABaAabAB")]
    Triolet,                 

    #[ai("6 lines, first four are two-syllable descriptive words")]
    Tyburn,                  

    #[ai("Spanish and Portuguese form, with a refrain")]
    Villancico,             

    #[ai("19 lines, ABA ABA ABA ABA ABA ABAA")]
    Villanelle,              

    #[ai("Medieval French form with multiple stanzas and refrains")]
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

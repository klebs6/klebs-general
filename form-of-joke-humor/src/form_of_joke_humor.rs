crate::ix!();

#[derive(Plural,Default,RandConstruct,ItemFeature,PartialEq,Eq,Hash,Copy,Clone,Debug)]
pub enum FormOfJokeHumor {
    #[ai("a surreal story filled with absurdly exaggerated or bizarre scenarios, leaving readers amused and bemused by its sheer ridiculousness")]  
    Absurdism,

    #[ai("a collection of amusing personal anecdotes, vividly told to highlight the humor in real-life experiences")]                               
    Anecdotal,

    #[ai("a playful script or dialogue filled with quick and witty exchanges of teasing remarks between characters")]                               
    #[default]
    Banter,

    #[ai("a darkly comic screenplay or short story that finds humor in tragic, taboo, or unsettling topics")]                                       
    BlackComedy,

    #[ai("a raucous stand-up routine filled with risqué jokes and vulgar humor, exploring topics often considered inappropriate")]                  
    BlueComedy,

    #[ai("a short comedy skit revolving around a series of humorous blunders or errors, exaggerated for comedic effect")]                           
    Blunder,

    #[ai("a clever anthology of bon mots, each presenting a sharp, witty, or intellectually amusing remark")]                                       
    BonMot,

    #[ai("a humorous monologue where exaggerated boasts and bragging are delivered in an obviously self-deprecating tone")]                         
    BraggingJoke,

    #[ai("a script or book featuring characters known for their iconic and humorous catchphrases, repeated to comic effect")]                       
    Catchphrase,

    #[ai("a charming letter or dialogue featuring compliments delivered with a humorous twist or playful exaggeration")]                            
    CharmingCompliment,

    #[ai("a collection of cheeky remarks, blending boldness and playful irreverence to tease without offense")]                                     
    CheekyComment,

    #[ai("a witty repartee featuring sharp and clever comebacks in response to criticism or insults")]                                              
    Comeback,

    #[ai("a humorous essay or article filled with absurd and exaggerated comparisons, creating unexpected and laugh-out-loud analogies")]           
    ComicComparison,

    #[ai("a humorous dialogue enriched with callbacks to previous jokes, weaving past humor seamlessly into the current conversation")]             
    ConversationalCallbacks,

    #[ai("a playful script where the direction of a conversation is humorously redirected through unexpected or witty interjections")]              
    ConversationalRedirection,

    #[ai("a cringe-inducing narrative centered on socially awkward or embarrassing situations, evoking humor through discomfort")]                  
    CringeComedy,

    #[ai("a comedy skit or essay built entirely around clever references to shared cultural touchpoints and media")]                                
    CulturalReference,

    #[ai("a series of grimly humorous anecdotes or short stories that make light of serious or taboo topics")]                                      
    DarkHumor,

    #[ai("a collection of one-liners delivered with complete deadpan seriousness, heightening the humor through lack of emotional expression")]     
    Deadpan,

    #[ai("a sarcastic monologue delivered with complete emotional detachment, combining irony with deadpan delivery for comedic effect")]           
    DeadpanSarcasm,

    #[ai("a comedy script featuring deliberate misunderstandings of dialogue to create playful misinterpretations and humor")]                      
    DeliberateMisunderstanding,

    #[ai("a collection of memes, jokes, and satirical content rooted in digital culture and the internet")]                                         
    DigitalAgeHumor,

    #[ai("a joke book featuring clever double entendres, where words or phrases have dual meanings, often with a risqué twist")]                    
    DoubleEntendre,

    #[ai("a playful collection of jokes based on the echoes or sounds of words, emphasizing auditory wordplay")]                                    
    EchoicWordplay,

    #[ai("a series of euphemistic statements humorously softening harsh truths or direct observations")]                                            
    EuphemismPlay,

    #[ai("a comedic article featuring exaggerated comparisons to highlight the absurdity or humor in various situations")]                          
    ExaggeratedComparison,

    #[ai("a comedic play or screenplay packed with farcical scenarios, improbable events, and slapstick humor")]                                    
    Farce,

    #[ai("a humorous short story masquerading as a true personal anecdote but revealed to be entirely fictional")]                                  
    FictionalAnecdote,

    #[ai("a playful script full of flirtatious teases, balancing humor with charm to engage and amuse the reader")]                                 
    FlirtatiousTease,

    #[ai("a set of humorous and friendly mockeries, showcasing teasing remarks meant to be taken in good spirits")]                                 
    FriendlyMocking,

    #[ai("a collection of grim jokes or anecdotes showcasing gallows humor, finding comedy in desperate or hopeless situations")]                   
    GallowsHumor,

    #[ai("a parody of historical events or figures, blending factual references with exaggerated or humorous twists")]                              
    HistoricalParody,

    #[ai("a list of imaginative and funny hypothetical scenarios that provoke thought while eliciting laughter")]                                   
    HumorousHypothetical,

    #[ai("a short story or dialogue packed with hyperbole, using outrageous exaggerations to achieve comedic effect")]                              
    Hyperbole,

    #[ai("a series of exaggerated insults delivered humorously, often to poke fun without causing offense")]                                        
    HyperbolicInsult,

    #[ai("a fictional letter or monologue presenting an imaginative and humorous scenario to captivate and amuse the reader")]                      
    ImaginativeScenario,

    #[ai("a collection of humorous interactions created through spontaneous and improvised dialogue")]                                              
    ImprovisationalComedy,

    #[ai("a witty dialogue rich in innuendo, where indirect suggestions add layers of humor to the conversation")]                                  
    Innuendo,

    #[ai("a comedic skit where humor derives from cleverly crafted insults targeting individuals or groups")]                                       
    InsultComedy,

    #[ai("a witty essay filled with intellectual humor, blending clever wordplay with sophisticated ideas")]                                        
    IntellectualWit,

    #[ai("a short story or commentary employing irony to create contrast and humor by saying the opposite of what is meant")]                       
    Irony,

    #[ai("a playful collection of jests, each offering lighthearted amusement through clever phrasing or action")]                                  
    Jest,

    #[ai("a humorous narrative based on juvenile themes like pranks, name-calling, or other immature antics")]                                      
    JuvenileHumor,

    #[ai("a collection of classic knock-knock jokes designed to entertain through clever wordplay and punchlines")]                                 
    KnockKnock,

    #[ai("a satirical dialogue filled with light irony, using mild contrasts for understated humor")]                                               
    LightIrony,

    #[ai("a set of playful and carefree comments, designed to lift the mood and elicit chuckles")]                                                  
    LightheartedComment,

    #[ai("a humorous passage filled with malapropisms, where word substitutions lead to unintentional but amusing meanings")]                       
    Malapropism,

    #[ai("a witty article filled with self-referential and meta jokes, poking fun at its own format and structure")]                                
    MetaHumor,

    #[ai("a list of misleading compliments, each leading with negativity but ultimately landing on a humorous or positive twist")]                  
    MisleadingCompliment,

    #[ai("a comedy skit featuring jokes with setups that deliberately mislead, only to surprise with subversive punchlines")]                       
    MisleadingSetup,

    #[ai("a playful essay pretending to take itself seriously while humorously mocking its own subject matter")]                                    
    MockSeriousness,

    #[ai("a fictional account presented as a mock documentary, blending realism with absurd or humorous details")]                                  
    Mockumentary,

    #[ai("a humorous poem or story inspired by mondegreens, using misheard phrases to create absurd or amusing meanings")]                          
    Mondegreen,

    #[ai("a whimsical collection of nonsensical jokes and stories, reveling in their lack of logical coherence")]                                   
    Nonsensical,

    #[ai("a witty story or essay where humor is derived from observations of everyday life and common experiences")]                                
    Observational,

    #[ai("a book of sharp one-liners, each delivering a punch of humor in a single sentence")]                                                      
    OneLiner,

    #[ai("a comedic essay built around overstated obviousness, humorously exaggerating common knowledge or situations")]                            
    OverstatedObvious,

    #[ai("a collection of paradoxical statements presented humorously to provoke thought and amusement")]                                           
    ParadoxicalStatement,

    #[ai("a satirical skit or essay parodying various styles with deliberate exaggeration for comedic effect")]                                     
    Parody,

    #[ai("a philosophical dialogue blending humor with deep insights into paradoxes or intellectual ideas")]                                        
    PhilosophicalJoke,

    #[ai("a vivid description of physical comedy, where humor arises from exaggerated actions and slapstick moments")]                              
    PhysicalComedy,

    #[ai("a playful challenge issued to provoke amusement and engagement in a humorous tone")]                                                      
    PlayfulChallenge,

    #[ai("a humorous passage that creatively twists well-known phrases into something unexpected or amusing")]                                      
    WordTwist, 

    #[ai("a collection of paraprosdokian sentences, each twisting expectations with an unexpected conclusion that redefines the humor of the setup")]
    Paraprosdokian,

    #[ai("a humorous story or skit filled with playful exaggerations, where characteristics or scenarios are blown out of proportion for comedic effect")]
    PlayfulExaggeration,

    #[ai("a dialogue or essay that humorously provokes or teases in a gentle, engaging manner, encouraging interaction and amusement")]
    PlayfulProvocation,

    #[ai("a collection of playful rhetorical questions that provoke thought or laughter by presenting absurd or amusing scenarios")]
    PlayfulRhetoricalQuestion,

    #[ai("a humor-filled essay or monologue packed with references to pop culture, engaging readers through shared cultural knowledge")]
    PopCultureReference,

    #[ai("a comedic script vividly describing prop-based humor, where objects are creatively integrated into the jokes and narrative")]
    PropComedy,

    #[ai("a series of jokes where each punchline delivers a satisfying and surprising twist to the story or setup")]
    Punchline,

    #[ai("a witty collection of quips, each offering a sharp or clever remark in a single line or short phrase")]
    Quip,

    #[ai("a lively dialogue or script full of repartee, where quick and clever exchanges of wit keep the humor flowing")]
    Repartee,

    #[ai("a humorous exchange built around retorts, where each reply is sharper and wittier than the last")]
    Retort,

    #[ai("a comedic essay or skit demonstrating reverse psychology, using humor to suggest the opposite of what is intended")]
    ReversePsychology,

    #[ai("a humorous story or script featuring a recurring gag that builds in humor through repetition and variation")]
    RunningGag,

    #[ai("a monologue or script brimming with sarcastic remarks, where irony and contempt are skillfully used for comedic effect")]
    Sarcasm,

    #[ai("a satirical essay or story that critiques human folly or vices with sharp humor, irony, and ridicule")]
    Satire,

    #[ai("a stand-up routine or monologue where the performer humorously highlights their own flaws and shortcomings")]
    SelfDeprecating,

    #[ai("a long-winded and meandering story ending with an anticlimactic or pointless punchline, reveling in its absurdity")]
    ShaggyDogStory,

    #[ai("a collection of silly questions, each absurd or nonsensical, designed to elicit laughter through their ridiculous premises")]
    SillyQuestion,

    #[ai("a humorous story or scene where the comedy arises naturally from a particular situation or context")]
    Situational,

    #[ai("a comedic story or skit where unexpected or absurd scenarios create humor through situational incongruity")]
    SituationalIncongruity,

    #[ai("a playful dialogue or story filled with spoonerisms, where transposed sounds create amusing linguistic twists")]
    Spoonerism,

    #[ai("a surreal narrative blending bizarre and nonsensical elements into a fantastical yet humorously absurd storyline")]
    SurrealHumor,

    #[ai("a lighthearted list of teasing nicknames, each blending humor and affection to create playful monikers")]
    TeasingNickname,

    #[ai("a collection of Tom Swifty sentences, where each quoted phrase humorously connects to its attribution through clever wordplay")]
    TomSwifty,

    #[ai("a collection of understated remarks, where humor arises from intentionally downplaying significant topics")]
    Understatement,

    #[ai("a list of unexpected analogies that juxtapose unrelated concepts to create humorous and thought-provoking comparisons")]
    UnexpectedAnalogy,

    #[ai("a humorous essay filled with remarks that, while funny, unexpectedly reveal profound truths or deep insights")]
    UnexpectedWisdom,

    #[ai("a series of clever and pithy wisecracks, where humor and wit are delivered in short, impactful bursts")]
    Wisecrack,

    #[ai("a collection of witty anecdotes and wordplay, blending cleverness and humor to delight and engage the audience")]
    Wit,

    #[ai("a list of quick and inventive verbal remarks, where creativity and humor shine in short bursts")]
    WittyRemark,

    #[ai("a playful collection of jokes and puns that showcase the art of linguistic manipulation and clever language use")]
    Wordplay,

    #[ai("a series of zingers, each offering a striking or amusing remark that delivers sharp humor with perfect timing")]
    Zinger,
}

impl OpenConversation for FormOfJokeHumor {
    fn open_conversation(&self) -> &'static str {
        match self {
            FormOfJokeHumor::Absurdism                  => "Let us dive into a parade of the surreal and nonsensical.",
            FormOfJokeHumor::Anecdotal                  => "Allow me to share a series of amusing life tales.",
            FormOfJokeHumor::Banter                     => "Here begins a volley of sharp and playful exchanges.",
            FormOfJokeHumor::BlackComedy                => "Prepare for a collection of darkly humorous misfortunes.",
            FormOfJokeHumor::BlueComedy                 => "Brace yourself for a round of bold and bawdy humor.",
            FormOfJokeHumor::Blunder                    => "Let us recount a series of laughable missteps.",
            FormOfJokeHumor::BonMot                     => "I present to you a series of sharp, witty remarks.",
            FormOfJokeHumor::BraggingJoke               => "Let us explore some boasts so outrageous they amuse.",
            FormOfJokeHumor::Catchphrase                => "Here are some iconic refrains that never fail to entertain.",
            FormOfJokeHumor::CharmingCompliment         => "Allow me to charm you with playful words of praise.",
            FormOfJokeHumor::CheekyComment              => "Let us revel in a series of bold, teasing remarks.",
            FormOfJokeHumor::Comeback                   => "Prepare for a sequence of sharp and satisfying retorts.",
            FormOfJokeHumor::ComicComparison            => "Here are some absurd parallels for your amusement.",
            FormOfJokeHumor::ConversationalCallbacks    => "Let us revisit past laughter with clever callbacks.",
            FormOfJokeHumor::ConversationalRedirection  => "Observe how humor twists in unexpected directions.",
            FormOfJokeHumor::CringeComedy               => "Let us delve into hilariously awkward situations.",
            FormOfJokeHumor::CulturalReference          => "Behold, a showcase of humor drawn from shared culture.",
            FormOfJokeHumor::DarkHumor                  => "Here is laughter born from the darkest corners of life.",
            FormOfJokeHumor::Deadpan                    => "Prepare for humor delivered with perfect seriousness.",
            FormOfJokeHumor::DeadpanSarcasm             => "Witness irony so dry it parches the soul.",
            FormOfJokeHumor::DeliberateMisunderstanding => "Let us play with words and their misinterpretations.",
            FormOfJokeHumor::DigitalAgeHumor            => "Here lies the humor of our online existence.",
            FormOfJokeHumor::DoubleEntendre             => "Let us explore the playful dualities of language.",
            FormOfJokeHumor::EchoicWordplay             => "Prepare for sounds twisted into laughter.",
            FormOfJokeHumor::EuphemismPlay              => "Let us gently laugh through softened truths.",
            FormOfJokeHumor::ExaggeratedComparison      => "Behold these ludicrously overstated analogies.",
            FormOfJokeHumor::Farce                      => "Step into a world of chaos and improbable hilarity.",
            FormOfJokeHumor::FictionalAnecdote          => "Let me regale you with tales that may not be true.",
            FormOfJokeHumor::FlirtatiousTease           => "Here begins a series of charmingly playful jests.",
            FormOfJokeHumor::FriendlyMocking            => "Let us enjoy some lighthearted ribbing among friends.",
            FormOfJokeHumor::GallowsHumor               => "Laugh as we confront life’s bleakest realities.",
            FormOfJokeHumor::HistoricalParody           => "Allow me to rewrite history with a comedic twist.",
            FormOfJokeHumor::HumorousHypothetical       => "Consider these what-ifs, crafted to amuse.",
            FormOfJokeHumor::Hyperbole                  => "Prepare for exaggerations stretched to absurdity.",
            FormOfJokeHumor::HyperbolicInsult           => "Let us hurl insults so extreme they delight.",
            FormOfJokeHumor::ImaginativeScenario        => "Imagine with me these whimsically absurd tales.",
            FormOfJokeHumor::ImprovisationalComedy      => "Watch as humor unfolds without a script.",
            FormOfJokeHumor::Innuendo                   => "Let us explore wit layered with suggestion.",
            FormOfJokeHumor::InsultComedy               => "Brace for cutting remarks crafted to amuse.",
            FormOfJokeHumor::IntellectualWit            => "Let us indulge in clever humor for the mind.",
            FormOfJokeHumor::Irony                      => "Behold a series of statements that mean their opposite.",
            FormOfJokeHumor::Jest                       => "Here is a collection of jokes for your light amusement.",
            FormOfJokeHumor::JuvenileHumor              => "Prepare for pranks and antics straight from childhood.",
            FormOfJokeHumor::KnockKnock                 => "Let us enter the realm of classic knock-knock setups.",
            FormOfJokeHumor::LightIrony                 => "Subtle contrasts await your laughter here.",
            FormOfJokeHumor::LightheartedComment        => "Enjoy these cheerful remarks to lift your spirit.",
            FormOfJokeHumor::Malapropism                => "Witness language twisted hilariously awry.",
            FormOfJokeHumor::MetaHumor                  => "Let us laugh at jokes about themselves.",
            FormOfJokeHumor::MisleadingCompliment       => "Observe how praise takes unexpected turns.",
            FormOfJokeHumor::MisleadingSetup            => "Let us journey through setups that surprise.",
            FormOfJokeHumor::MockSeriousness            => "Here is gravitas crafted only to amuse.",
            FormOfJokeHumor::Mockumentary               => "Witness life’s absurdities in mock-serious detail.",
            FormOfJokeHumor::Mondegreen                 => "Laugh at misheard lyrics and phrases reimagined.",
            FormOfJokeHumor::Nonsensical                => "Embrace the joy of the utterly illogical.",
            FormOfJokeHumor::Observational              => "Find humor in life’s everyday peculiarities.",
            FormOfJokeHumor::OneLiner                   => "Prepare for punchlines delivered in a single stroke.",
            FormOfJokeHumor::OverstatedObvious          => "Laugh as we make the obvious absurdly explicit.",
            FormOfJokeHumor::ParadoxicalStatement       => "Ponder these contradictions with a smile.",
            FormOfJokeHumor::Parody                     => "Here begins a playful twist on familiar forms.",
            FormOfJokeHumor::PhilosophicalJoke          => "Let us mix humor with musings on existence.",
            FormOfJokeHumor::PhysicalComedy             => "Prepare for laughs through exaggerated action.",
            FormOfJokeHumor::PlayfulChallenge           => "Face these cheeky dares with a grin.",
            FormOfJokeHumor::WordTwist                  => "Discover joy in the unexpected turns of language.",
            FormOfJokeHumor::Paraprosdokian             => "Savor setups that twist into the unexpected.",
            FormOfJokeHumor::PlayfulExaggeration        => "Delight in humor made larger than life.",
            FormOfJokeHumor::PlayfulProvocation         => "Let these gentle prods draw out your laughter.",
            FormOfJokeHumor::PlayfulRhetoricalQuestion  => "Ponder the ridiculous with these playful queries.",
            FormOfJokeHumor::PopCultureReference        => "Dive into humor rooted in shared culture.",
            FormOfJokeHumor::PropComedy                 => "Laugh as objects come alive in jest.",
            FormOfJokeHumor::Punchline                  => "Setups conclude with twists sure to amuse.",
            FormOfJokeHumor::Quip                       => "Enjoy these quick and clever remarks.",
            FormOfJokeHumor::Repartee                   => "Wit and retorts fly fast and sharp.",
            FormOfJokeHumor::Retort                     => "Sharper and wittier replies await.",
            FormOfJokeHumor::ReversePsychology          => "Laugh as we say one thing and mean another.",
            FormOfJokeHumor::RunningGag                 => "Enjoy humor that grows through repetition.",
            FormOfJokeHumor::Sarcasm                    => "Irony and scorn mingle in these remarks.",
            FormOfJokeHumor::Satire                     => "Critique and comedy combine here.",
            FormOfJokeHumor::SelfDeprecating            => "Laugh with me at my own flaws.",
            FormOfJokeHumor::ShaggyDogStory             => "Prepare for long tales with absurd endings.",
            FormOfJokeHumor::SillyQuestion              => "Ask yourself these ridiculous queries.",
            FormOfJokeHumor::Situational                => "Comedy arises naturally from these contexts.",
            FormOfJokeHumor::SituationalIncongruity     => "Humor blossoms from absurd mismatches.",
            FormOfJokeHumor::Spoonerism                 => "Language twists delightfully into humor.",
            FormOfJokeHumor::SurrealHumor               => "Wander into the bizarre and fantastical.",
            FormOfJokeHumor::TeasingNickname            => "Playful names bring lighthearted fun.",
            FormOfJokeHumor::TomSwifty                  => "Language and attribution collide for laughs.",
            FormOfJokeHumor::Understatement             => "Laughter comes from saying less with more.",
            FormOfJokeHumor::UnexpectedAnalogy          => "Unlikely comparisons spark surprise.",
            FormOfJokeHumor::UnexpectedWisdom           => "Truths revealed humorously await.",
            FormOfJokeHumor::Wisecrack                  => "Quick bursts of wit to amuse.",
            FormOfJokeHumor::Wit                        => "A series of clever thoughts to enjoy.",
            FormOfJokeHumor::WittyRemark                => "Cleverness distilled into short jests.",
            FormOfJokeHumor::Wordplay                   => "Language games abound in these jokes.",
            FormOfJokeHumor::Zinger                     => "Sharp and striking remarks to end with flair.",
        }
    }
}

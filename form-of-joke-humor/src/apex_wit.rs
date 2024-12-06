crate::ix!();

#[derive(Plural,Default,RandConstruct,ItemFeature,PartialEq,Eq,Hash,Copy,Clone,Debug)]
pub enum ApexWit {
    #[ai("an exquisitely crafted personal anecdote, delivered with perfect timing and detail, where the humor emerges naturally from life’s absurdities")]
    Anecdotal,

    #[ai("a masterclass in playful banter, showcasing a dynamic exchange of sharp, teasing remarks that sparkle with wit and camaraderie")]
    #[default]
    Banter,

    #[ai("a boldly risqué quip, balancing vulgarity with clever subtext to charm and disarm the listener without overstepping decorum")]
    BlueComedy,

    #[ai("a humor-laden boast that deftly exaggerates personal achievements, delivered with a wink and a nod to its own absurdity")]
    BraggingJoke,

    #[ai("an iconic and humor-laden catchphrase, elegantly repurposed to immediately draw in the listener and elicit delight through familiarity")]
    Catchphrase,

    #[ai("a compliment imbued with subtle humor, where charm and wit converge to elevate the recipient’s spirits with an artful twist")]
    CharmingCompliment,

    #[ai("a delightfully cheeky comment, balancing boldness and irreverence with an undercurrent of playful respect")]
    CheekyComment,

    #[ai("an impeccably timed comeback that disarms criticism with razor-sharp wit and effortless flair")]
    Comeback,

    #[ai("a humorously exaggerated or absurd comparison that juxtaposes the ordinary with the extraordinary to create vivid and unexpected imagery")]
    ComicComparison,

    #[ai("a conversation laced with callbacks to prior humorous moments, skillfully weaving past wit into the present dialogue to deepen the rapport")]
    ConversationalCallbacks,

    #[ai("a conversational pivot that redirects the topic with a clever or unexpected twist, keeping the tone lively and engaging")]
    ConversationalRedirection,

    #[ai("an intentionally awkward or embarrassing anecdote that evokes laughter through shared cringeworthy experiences")]
    CringeComedy,

    #[ai("a playful misunderstanding of a statement, leading to a humorous reinterpretation that subverts expectations")]
    DeliberateMisunderstanding,

    #[ai("a double entendre that masterfully balances sophistication and subtlety, offering two meanings with an artful risqué edge")]
    DoubleEntendre,

    #[ai("a fictional anecdote delivered with such conviction and charm that its humor lies in the creative absurdity of its premise")]
    FictionalAnecdote,

    #[ai("a flirtatious tease that blends wit and charm to captivate and amuse, leaving just enough ambiguity to spark intrigue")]
    FlirtatiousTease,

    #[ai("a hyperbolic insult crafted with elegance and humor, so exaggerated that it becomes an artful display of wit")]
    HyperbolicInsult,

    #[ai("an imaginative and vividly described scenario that engages the listener with humor while inviting them to explore a whimsical possibility")]
    ImaginativeScenario,

    #[ai("an off-the-cuff burst of humor that feels effortless yet lands perfectly, capturing the charm of spontaneous wit")]
    ImprovisationalComedy,

    #[ai("an innuendo so skillfully crafted that its suggestiveness is balanced by wit, leaving room for playful interpretation")]
    Innuendo,

    #[ai("a bold, humor-filled jab that balances the sharpness of insult comedy with enough charm to keep the tone light-hearted")]
    InsultComedy,

    #[ai("a quip so intellectually sharp and layered with meaning that it demands and rewards a high level of engagement from the listener")]
    IntellectualWit,

    #[ai("a playful jab rooted in the juvenile joys of pranks and name-calling, artfully elevated to evoke nostalgia and laughter")]
    JuvenileHumor,

    #[ai("a perfectly constructed knock-knock joke that plays on clever wordplay and an unexpected punchline")]
    KnockKnock,

    #[ai("a misleading compliment that begins with faux negativity but lands on a charmingly positive note, eliciting laughter through surprise")]
    MisleadingCompliment,

    #[ai("a joke with a deliberately deceptive setup, steering the listener toward one expectation before delivering a delightfully subversive punchline")]
    MisleadingSetup,

    #[ai("a one-liner so sharp and succinct that its humor resonates immediately and lingers long after delivery")]
    OneLiner,

    #[ai("a vivid description of physical humor where actions are so intricately detailed that the humor leaps off the page")]
    PhysicalComedy,

    #[ai("a playful challenge issued with humor and confidence, designed to intrigue and engage the listener in light-hearted competition")]
    PlayfulChallenge,

    #[ai("a lighthearted exaggeration so artfully executed that it heightens the humor without straining credibility")]
    PlayfulExaggeration,

    #[ai("a witty provocation delivered with enough charm and humor to tease without offending, sparking lively engagement")]
    PlayfulProvocation,

    #[ai("a humorously detailed description of a prop-based gag, turning the mundane into a vehicle for clever comedy")]
    PropComedy,

    #[ai("a lively exchange of repartee, where quick-witted remarks flow effortlessly in a conversation filled with charm and humor")]
    Repartee,

    #[ai("a sharply worded retort that turns criticism into comedy, flipping the script with unassailable wit")]
    Retort,

    #[ai("a statement of profound and unexpected wisdom delivered with a humorous twist, leaving the listener both amused and thoughtful")]
    UnexpectedWisdom,

    #[ai("a zinger that lands with pinpoint precision, delivering humor so striking and well-timed it becomes unforgettable")]
    Zinger,
}

impl OpenConversation for ApexWit {
    fn open_conversation(&self) -> &'static str {
        match self {
            ApexWit::Anecdotal                  => "Let us begin with a set of vivid and humorous life tales.",
            ApexWit::Banter                     => "Prepare for a lively back-and-forth of sharp, playful exchanges.",
            ApexWit::BlueComedy                 => "Here comes a bold series of risqué quips to amuse and surprise.",
            ApexWit::BraggingJoke               => "Let us explore boasts so clever they laugh at themselves.",
            ApexWit::Catchphrase                => "Behold some iconic lines that charm with instant familiarity.",
            ApexWit::CharmingCompliment         => "Allow me to introduce a sequence of witty, flattering remarks.",
            ApexWit::CheekyComment              => "Here is a collection of bold yet irresistibly playful remarks.",
            ApexWit::Comeback                   => "Prepare for comebacks that sting and sparkle with wit.",
            ApexWit::ComicComparison            => "Let us draw absurd and unexpected parallels for your delight.",
            ApexWit::ConversationalCallbacks    => "Revisit past moments of laughter with clever callbacks.",
            ApexWit::ConversationalRedirection  => "Watch humor twist with each clever conversational pivot.",
            ApexWit::CringeComedy               => "Relive awkward moments so uncomfortable they become hilarious.",
            ApexWit::DeliberateMisunderstanding => "Join me in playful misinterpretations that spark laughter.",
            ApexWit::DoubleEntendre             => "Let us uncover double meanings, masterfully balanced and witty.",
            ApexWit::FictionalAnecdote          => "Imagine tales so absurd they're believable and hilarious.",
            ApexWit::FlirtatiousTease           => "Delight in a series of witty, charming, and playful jests.",
            ApexWit::HyperbolicInsult           => "Prepare for barbs so extreme they become comedic art.",
            ApexWit::ImaginativeScenario        => "Explore whimsical scenarios crafted to engage and amuse.",
            ApexWit::ImprovisationalComedy      => "Experience humor that unfolds spontaneously and effortlessly.",
            ApexWit::Innuendo                   => "Let us indulge in wit layered with playful suggestion.",
            ApexWit::InsultComedy               => "Brace yourself for sharp jabs delivered with charming humor.",
            ApexWit::IntellectualWit            => "Engage with humor that rewards the sharpest of minds.",
            ApexWit::JuvenileHumor              => "Revisit the joys of playful pranks and youthful mischief.",
            ApexWit::KnockKnock                 => "Knock, knock—let classic humor come through the door.",
            ApexWit::MisleadingCompliment       => "Enjoy compliments that twist into delightful surprises.",
            ApexWit::MisleadingSetup            => "Let us steer expectations before flipping them into laughter.",
            ApexWit::OneLiner                   => "Prepare for sharp, memorable jokes delivered in a single stroke.",
            ApexWit::PhysicalComedy             => "Visualize humor in motion, brought to life through vivid detail.",
            ApexWit::PlayfulChallenge           => "Step into a lighthearted contest of wit and humor.",
            ApexWit::PlayfulExaggeration        => "Let these over-the-top tales take humor to new heights.",
            ApexWit::PlayfulProvocation         => "Enjoy provocations that tease and engage without offense.",
            ApexWit::PropComedy                 => "Marvel at objects transformed into ingenious tools of humor.",
            ApexWit::Repartee                   => "Enter a realm of quick-witted exchanges that flow with charm.",
            ApexWit::Retort                     => "Witness retorts that flip the script with razor-sharp precision.",
            ApexWit::UnexpectedWisdom           => "Discover truths hidden in humor, both sharp and profound.",
            ApexWit::Zinger                     => "Finish with zingers so precise they'll echo in your laughter.",
        }
    }
}

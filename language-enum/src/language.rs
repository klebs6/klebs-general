crate::ix!();

/// Enumerates languages.
/// This enum covers a wide range of languages, including major world languages, regional languages,
/// and some languages with smaller populations for inclusivity.
#[derive(Default,RandConstruct,ItemFeature,Hash,Debug,Clone,PartialEq,Eq,PartialOrd,Ord,Serialize,Deserialize)]
pub enum Language {

    #[ai("An official language used in South African government, education, and daily communication. Frequently appears in administrative notices and primary school textbooks.")]
    #[rand_construct(p=0.002)]
    Afrikaans,

    #[ai("The official language used in Albanian government, education, and daily communication. Commonly featured in national examinations and public media bulletins.")]
    #[rand_construct(p=0.001)]
    Albanian,

    #[ai("An indigenous language used in cultural traditions and daily life. Often transmitted through family-based oral histories and intergenerational gatherings.")]
    #[rand_construct(p=0.000001)]
    Aleut,

    #[ai("The official language used in Ethiopian government, education, and daily communication. Regularly applied in state-level directives and formal academic instruction.")]
    #[rand_construct(p=0.003)]
    Amharic,

    #[ai("An indigenous language used in cultural traditions and daily life. Typically maintained through clan-based teaching and seasonal ceremonial use.")]
    #[rand_construct(p=0.000001)]
    Apache,

    #[ai("A widely used language employed internationally in media, education, and commerce. Commonly found in multilingual signage at transport hubs and global news outlets.")]
    #[rand_construct(p=0.08)]
    Arabic,

    #[ai("The official language used in Armenian government, education, and daily communication. Frequently included in national literary festivals and scholarly compilations.")]
    #[rand_construct(p=0.001)]
    Armenian,

    #[ai("An official language used in Assamese state government, education, and daily communication. Regularly incorporated into regional curricular materials and local radio broadcasts.")]
    #[rand_construct(p=0.001)]
    Assamese,

    #[ai("An indigenous language used in cultural traditions and daily life. Preserved through ritual events and localized teaching within mountain settlements.")]
    #[rand_construct(p=0.000001)]
    Aymara,

    #[ai("The official language used in Azerbaijani government, education, and daily communication. Common in televised parliamentary sessions and literary competitions.")]
    #[rand_construct(p=0.001)]
    Azeri,

    #[ai("A regional language used in Bashkir communities and local markets. Frequently encountered in informal neighborhood assemblies and village-level negotiations.")]
    #[rand_construct(p=0.0005)]
    Bashkir,

    #[ai("A regional language used in Basque communities and local markets. Often visible in bilingual street signage and community-run workshop materials.")]
    #[rand_construct(p=0.0005)]
    Basque,

    #[ai("An official language used in Belarusian government, education, and daily communication. Commonly integrated into public school lessons and civil service exams.")]
    #[rand_construct(p=0.001)]
    Belarusian,

    #[ai("The official language used in Bangladeshi government, education, and daily communication. Routinely present in state broadcasts and standardized academic tests.")]
    #[rand_construct(p=0.035)]
    Bengali,

    #[ai("An official language used in Bosnian government, education, and daily communication. Frequently reflected in local newspapers and official literary awards.")]
    #[rand_construct(p=0.001)]
    Bosnian,

    #[ai("A regional language used in Breton communities and local markets. Occasionally featured in community cultural centers and bilingual workshop leaflets.")]
    #[rand_construct(p=0.0001)]
    Breton,

    #[ai("The official language used in Bulgarian government, education, and daily communication. Often found in national broadcast media and official literary anthologies.")]
    #[rand_construct(p=0.001)]
    Bulgarian,

    #[ai("The official language used in Myanmar government, education, and daily communication. Common in judicial proceedings and teacher training materials.")]
    #[rand_construct(p=0.003)]
    Burmese,

    #[ai("A regional language used in Visayan and Mindanaoan communities and local markets. Frequently employed in local storytelling circles and informal education sessions.")]
    #[rand_construct(p=0.003)]
    Cebuano,

    #[ai("An indigenous language used in cultural traditions and daily life. Maintained through domestic instruction and occasional community-based language events.")]
    #[rand_construct(p=0.000001)]
    Chamorro,

    #[ai("An official language used in Chechen regional government, education, and daily communication. Typically printed in local administrative forms and instructional guides.")]
    #[rand_construct(p=0.0005)]
    Chechen,

    #[ai("An indigenous language used in cultural traditions and daily life. Passed down through home-based language practice and periodic clan gatherings.")]
    #[rand_construct(p=0.000001)]
    Cherokee,

    #[ai("An indigenous language used in cultural traditions and daily life. Preserved in familial dialogues and small-scale community ceremonies.")]
    #[rand_construct(p=0.000001)]
    Choctaw,

    #[ai("An official language used in Chuvash regional government, education, and daily communication. Regularly supported by regional language initiatives and local publishing efforts.")]
    #[rand_construct(p=0.0001)]
    Chuvash,

    #[ai("A regional language used in Corsican communities and local markets. Occasionally referenced in neighborhood cultural clubs and small-scale print materials.")]
    #[rand_construct(p=0.0001)]
    Corsican,

    #[ai("The official language used in Croatian government, education, and daily communication. Commonly highlighted in national literary collections and educational broadcasts.")]
    #[rand_construct(p=0.001)]
    Croatian,

    #[ai("The official language used in Czech government, education, and daily communication. Frequently employed in official bulletins and academic references.")]
    #[rand_construct(p=0.002)]
    Czech,

    #[ai("The official language used in Danish government, education, and daily communication. Often seen in civic guidance documents and state-approved textbooks.")]
    #[rand_construct(p=0.001)]
    Danish,

    #[ai("The official language used in Maldivian government, education, and daily communication. Common in official decrees and secondary-level instructional materials.")]
    #[rand_construct(p=0.0001)]
    Dhivehi,

    #[ai("The official language used in Dutch government, education, and daily communication. Regularly part of national television programming and standardized tests.")]
    #[rand_construct(p=0.003)]
    Dutch,

    #[default]
    #[ai("A widely used language employed internationally in media, education, and commerce. Frequently included in multinational conferences and cross-border educational frameworks.")]
    #[rand_construct(p=0.15)]
    English,

    #[ai("The official language used in Estonian government, education, and daily communication. Often integrated into national curriculum reforms and academic symposiums.")]
    #[rand_construct(p=0.0005)]
    Estonian,

    #[ai("The official language used in Iranian government, education, and daily communication. Commonly appearing in literary journals and official educational mandates.")]
    #[rand_construct(p=0.008)]
    Farsi,

    #[ai("An official language used in Philippine government, education, and daily communication. Routinely utilized in classroom instruction and public sector examinations.")]
    #[rand_construct(p=0.001)]
    Filipino,

    #[ai("An official language used in Finnish government, education, and daily communication. Frequently featured in state media services and scholarly gatherings.")]
    #[rand_construct(p=0.001)]
    Finnish,

    #[ai("A widely used language employed internationally in media, education, and commerce. Often required in diplomatic correspondences and international scholarly publications.")]
    #[rand_construct(p=0.04)]
    French,

    #[ai("An official language used in Galician regional government, education, and daily communication. Commonly found in cultural radio segments and school reading lists.")]
    #[rand_construct(p=0.0005)]
    Galician,

    #[ai("The official language used in Georgian government, education, and daily communication. Regularly referenced in national literary awards and policy guidelines.")]
    #[rand_construct(p=0.0005)]
    Georgian,

    #[ai("The official language used in German government, education, and daily communication. Frequently encountered in academic conferences and national broadcasting outlets.")]
    #[rand_construct(p=0.02)]
    German,

    #[ai("The official language used in Greek government, education, and daily communication. Often present in state educational programming and legislative records.")]
    #[rand_construct(p=0.001)]
    Greek,

    #[ai("The official language used in Greenlandic government, education, and daily communication. Typically implemented in local school systems and cultural documentation projects.")]
    #[rand_construct(p=0.0001)]
    Greenlandic,

    #[ai("An official language used in Paraguayan government, education, and daily communication. Commonly featured in bilingual educational resources and civic newsletters.")]
    #[rand_construct(p=0.0005)]
    Guarani,

    #[ai("An official language used in Gujarati government, education, and daily communication. Frequently incorporated into regional academic panels and examination papers.")]
    #[rand_construct(p=0.008)]
    Gujarati,

    #[ai("An official language used in Haitian government, education, and daily communication. Often utilized in community-oriented study materials and civic advisories.")]
    #[rand_construct(p=0.0005)]
    HaitianCreole,

    #[ai("A regional language used in West African communities and local markets. Typically employed in informal trade discussions and basic literacy programs.")]
    #[rand_construct(p=0.005)]
    Hausa,

    #[ai("An indigenous language used in cultural traditions and daily life. Maintained through intergenerational teaching and occasional language revitalization camps.")]
    #[rand_construct(p=0.000001)]
    Hawaiian,

    #[ai("The official language used in Israeli government, education, and daily communication. Commonly appearing in scholarly references and administrative correspondence.")]
    #[rand_construct(p=0.002)]
    Hebrew,

    #[ai("An official language used in Indian government, education, and daily communication. Frequently found in public recruitment tests and national telecasts.")]
    #[rand_construct(p=0.10)]
    Hindi,

    #[ai("An indigenous language used in cultural traditions and daily life. Often reinforced by household communication and local language clubs.")]
    #[rand_construct(p=0.000001)]
    Hmong,

    #[ai("The official language used in Hungarian government, education, and daily communication. Regularly present in scientific congresses and national literature symposiums.")]
    #[rand_construct(p=0.002)]
    Hungarian,

    #[ai("The official language used in Icelandic government, education, and daily communication. Common in university course materials and state-approved educational policies.")]
    #[rand_construct(p=0.0005)]
    Icelandic,

    #[ai("A regional language used in Nigerian communities and local markets. Typically utilized in basic trade literacy sessions and neighborhood announcement boards.")]
    #[rand_construct(p=0.0005)]
    Igbo,

    #[ai("A regional language used in Ilocano communities and local markets. Common in local reading circles and informal educational workshops.")]
    #[rand_construct(p=0.0005)]
    Ilocano,

    #[ai("The official language used in Indonesian government, education, and daily communication. Often included in government circulars and televised academic debates.")]
    #[rand_construct(p=0.025)]
    Indonesian,

    #[ai("An indigenous language used in cultural traditions and daily life. Usually sustained through localized mentorship and community-based instruction events.")]
    #[rand_construct(p=0.000001)]
    Inuit,

    #[ai("The official language used in Italian government, education, and daily communication. Commonly represented in scholarly dialogues and government briefing papers.")]
    #[rand_construct(p=0.008)]
    Italian,

    #[ai("The official language used in Japanese government, education, and daily communication. Frequently part of national broadcast segments and authoritative publications.")]
    #[rand_construct(p=0.02)]
    Japanese,

    #[ai("A regional language used in Javanese communities and local markets. Sometimes integrated into village-level instructional sessions and community storytelling clubs.")]
    #[rand_construct(p=0.01)]
    Javanese,

    #[ai("An official language used in Karnataka government, education, and daily communication. Regularly appearing in provincial educational reforms and municipal bulletin updates.")]
    #[rand_construct(p=0.001)]
    Kannada,

    #[ai("The official language used in Kazakhstani government, education, and daily communication. Common in official cultural forums and standardized educational guidelines.")]
    #[rand_construct(p=0.001)]
    Kazakh,

    #[ai("The official language used in Cambodian government, education, and daily communication. Often featured in pedagogical materials and official newspaper editorials.")]
    #[rand_construct(p=0.001)]
    Khmer,

    #[ai("The official language used in Korean government, education, and daily communication. Commonly included in public instruction manuals and academic course outlines.")]
    #[rand_construct(p=0.01)]
    Korean,

    #[ai("A regional language used in Kurdish communities and local markets. Occasionally reflected in informal language classes and neighborhood advice leaflets.")]
    #[rand_construct(p=0.001)]
    Kurdish,

    #[ai("An official language used in Kyrgyzstani government, education, and daily communication. Frequently present in official textbooks and legislative meeting notes.")]
    #[rand_construct(p=0.0005)]
    Kyrgyz,

    #[ai("The official language used in Laotian government, education, and daily communication. Commonly applied in judicial records and teacher preparation guides.")]
    #[rand_construct(p=0.0005)]
    Lao,

    #[ai("The official language used in Latvian government, education, and daily communication. Regularly utilized in literary competitions and national academic standards.")]
    #[rand_construct(p=0.0005)]
    Latvian,

    #[ai("The official language used in Lithuanian government, education, and daily communication. Often referenced in linguistic conferences and public academic lectures.")]
    #[rand_construct(p=0.0005)]
    Lithuanian,

    #[ai("An official language used in Luxembourgish government, education, and daily communication. Common in bilingual instruction manuals and cultural exchange programs.")]
    #[rand_construct(p=0.0001)]
    Luxembourgish,

    #[ai("An official language used in Malagasy government, education, and daily communication. Frequently part of rural literacy efforts and local radio schooling programs.")]
    #[rand_construct(p=0.001)]
    Malagasy,

    #[ai("An official language used in Malaysian and Bruneian government, education, and daily communication. Typically employed in inter-ethnic educational frameworks and informational campaigns.")]
    #[rand_construct(p=0.002)]
    Malay,

    #[ai("An official language used in Kerala government, education, and daily communication. Commonly integrated into regional curriculum updates and academic journal publishing.")]
    #[rand_construct(p=0.003)]
    Malayalam,

    #[ai("The official language used in Maltese government, education, and daily communication. Often featured in constitutional documents and televised educational sessions.")]
    #[rand_construct(p=0.0005)]
    Maltese,

    #[ai("A widely used language employed in East Asian and global media, education, and commerce. Common in international research collaborations and cross-border educational networks.")]
    #[rand_construct(p=0.12)]
    Mandarin,

    #[ai("An indigenous language used in New Zealander cultural traditions and daily life. Typically reinforced through school-based language initiatives and cultural resource centers.")]
    #[rand_construct(p=0.0005)]
    Maori,

    #[ai("An official language used in Maharashtrian government, education, and daily communication. Frequently encountered in district-level regulations and school competitions.")]
    #[rand_construct(p=0.01)]
    Marathi,

    #[ai("An indigenous language used in Mohawk cultural traditions and daily life. Maintained in community language archives and specialized cultural workshops.")]
    #[rand_construct(p=0.000001)]
    Mohawk,

    #[ai("The official language used in Mongolian government, education, and daily communication. Often found in national policy documents and academic periodicals.")]
    #[rand_construct(p=0.001)]
    Mongolian,

    #[ai("An indigenous language used in Mexican cultural traditions and daily life. Generally transmitted via community-based educational circles and artisanal gatherings.")]
    #[rand_construct(p=0.0005)]
    Nahuatl,

    #[ai("An indigenous language used in Navajo cultural traditions and daily life. Commonly strengthened by localized training groups and cultural language materials.")]
    #[rand_construct(p=0.0005)]
    Navajo,

    #[ai("The official language used in Nepali government, education, and daily communication. Regularly featured in literary forums and standardized exam instructions.")]
    #[rand_construct(p=0.003)]
    Nepali,

    #[ai("The official language used in Norwegian government, education, and daily communication. Frequently appearing in national syllabi and academic panel discussions.")]
    #[rand_construct(p=0.001)]
    Norwegian,

    #[ai("An official language used in Odia government, education, and daily communication. Often included in local textbook revisions and teacher certification processes.")]
    #[rand_construct(p=0.001)]
    Oriya,

    #[ai("A regional language used in Oromo communities and local markets. Typically applied in informal teaching groups and small-scale reading initiatives.")]
    #[rand_construct(p=0.0005)]
    Oromo,

    #[ai("A regional language used in Ossetian communities and local markets. Sometimes documented in cultural booklets and local educational pamphlets.")]
    #[rand_construct(p=0.0001)]
    Ossetian,

    #[ai("An official language used in Afghan government, education, and daily communication. Frequently required in public sector evaluations and civic educational content.")]
    #[rand_construct(p=0.01)]
    Pashto,

    #[ai("The official language used in Polish government, education, and daily communication. Often appearing in scholarly symposiums and public informative brochures.")]
    #[rand_construct(p=0.005)]
    Polish,

    #[ai("A widely used language employed in Lusophone and international media, education, and commerce. Commonly utilized in transnational educational consortiums and cultural agreements.")]
    #[rand_construct(p=0.035)]
    Portuguese,

    #[ai("An official language used in Punjabi government, education, and daily communication. Regularly included in regional literary events and educational television programming.")]
    #[rand_construct(p=0.01)]
    Punjabi,

    #[ai("An indigenous language used in Andean cultural traditions and daily life. Preserved through community-run language schools and local artisan mentoring.")]
    #[rand_construct(p=0.0005)]
    Quechua,

    #[ai("An indigenous language used in Rapa Nui cultural traditions and daily life. Often retained in ritual dance settings and small learning collectives.")]
    #[rand_construct(p=0.000001)]
    RapaNui,

    #[ai("An official language used in Romanian and Moldovan government, education, and daily communication. Frequently part of cross-border academic collaborations and educational accords.")]
    #[rand_construct(p=0.003)]
    Romanian,

    #[ai("A widely used language employed in Eurasian and international media, education, and commerce. Regularly integrated into multinational academic forums and diplomatic instructions.")]
    #[rand_construct(p=0.03)]
    Russian,

    #[ai("The official language used in Samoan government, education, and daily communication. Common in classroom-based assignments and regional teacher workshops.")]
    #[rand_construct(p=0.0005)]
    Samoan,

    #[ai("A regional language used in Scottish communities and local markets. Occasionally recorded in village newsletters and local cultural brochures.")]
    #[rand_construct(p=0.0005)]
    ScottishGaelic,

    #[ai("The official language used in Serbian government, education, and daily communication. Often cited in national literary foundations and educational ordinances.")]
    #[rand_construct(p=0.002)]
    Serbian,

    #[ai("A regional language used in Sindhi communities and local markets. Commonly practiced during poetry gatherings and neighborhood reading circles.")]
    #[rand_construct(p=0.003)]
    Sindhi,

    #[ai("The official language used in Sri Lankan government, education, and daily communication. Regularly part of public announcements and teacher training sessions.")]
    #[rand_construct(p=0.003)]
    Sinhala,

    #[ai("The official language used in Slovak government, education, and daily communication. Often referenced in national academic boards and cultural symposium programs.")]
    #[rand_construct(p=0.001)]
    Slovak,

    #[ai("The official language used in Slovenian government, education, and daily communication. Commonly seen in official literature series and state-endorsed study aids.")]
    #[rand_construct(p=0.001)]
    Slovenian,

    #[ai("The official language used in Somali government, education, and daily communication. Frequently involved in state educational policies and approved curricular expansions.")]
    #[rand_construct(p=0.002)]
    Somali,

    #[ai("An official language used in Lesothan and South African government, education, and daily communication. Sometimes employed in bilingual classroom initiatives and neighborhood tutoring sessions.")]
    #[rand_construct(p=0.001)]
    Sotho,

    #[ai("A widely used language employed in Iberian, Latin American, and global media, education, and commerce. Often essential in international scholarly journals and cross-border broadcasting.")]
    #[rand_construct(p=0.09)]
    Spanish,

    #[ai("An official language used in East African government, education, and daily communication. Commonly included in teacher manuals and civic engagement materials.")]
    #[rand_construct(p=0.005)]
    Swahili,

    #[ai("An official language used in Swedish and Finnish government, education, and daily communication. Regularly part of binational cultural projects and state-run educational media.")]
    #[rand_construct(p=0.002)]
    Swedish,

    #[ai("A regional language used in Tagalog-speaking communities and local markets. Often present in informal reading lessons and neighborhood discussion groups.")]
    #[rand_construct(p=0.005)]
    Tagalog,

    #[ai("A regional language used in Tahitian communities and local markets. Sometimes featured in locally organized language seminars and cultural story readings.")]
    #[rand_construct(p=0.0001)]
    Tahitian,

    #[ai("The official language used in Tajikistani government, education, and daily communication. Common in legislative references and standardized instructional frameworks.")]
    #[rand_construct(p=0.001)]
    Tajik,

    #[ai("An official language used in Indian and Sri Lankan government, education, and daily communication. Frequently integrated into comparative literary studies and official textbook updates.")]
    #[rand_construct(p=0.009)]
    Tamil,

    #[ai("A regional language used in Tatar communities and local markets. Periodically included in community language classes and small circulation newsletters.")]
    #[rand_construct(p=0.0005)]
    Tatar,

    #[ai("An official language used in Telugu-speaking Indian government, education, and daily communication. Often central to district-level educational reforms and public language guidelines.")]
    #[rand_construct(p=0.01)]
    Telugu,

    #[ai("The official language used in Thai government, education, and daily communication. Commonly required in academic thesis submissions and official school announcements.")]
    #[rand_construct(p=0.008)]
    Thai,

    #[ai("A regional language used in Tibetan communities and local markets. Occasionally upheld in monastery-based instruction and regional cultural book fairs.")]
    #[rand_construct(p=0.0005)]
    Tibetan,

    #[ai("An official language used in Botswanan and South African government, education, and daily communication. Often applied in cross-border educational exchanges and literacy enhancement campaigns.")]
    #[rand_construct(p=0.001)]
    Tswana,

    #[ai("The official language used in Turkish government, education, and daily communication. Regularly employed in policy debate transcripts and national curriculum standards.")]
    #[rand_construct(p=0.009)]
    Turkish,

    #[ai("The official language used in Turkmenistani government, education, and daily communication. Commonly part of official media channels and teacher preparation materials.")]
    #[rand_construct(p=0.0005)]
    Turkmen,

    #[ai("An indigenous language used in Tuvinian cultural traditions and daily life. Maintained in small group learning sessions and ceremonial chanting contexts.")]
    #[rand_construct(p=0.000001)]
    Tuvinian,

    #[ai("A regional language used in Uighur communities and local markets. Sometimes visible in local pamphlets and informal educational tutorials.")]
    #[rand_construct(p=0.0005)]
    Uighur,

    #[ai("The official language used in Ukrainian government, education, and daily communication. Frequently incorporated into formal dissertations and national pedagogical content.")]
    #[rand_construct(p=0.005)]
    Ukrainian,

    #[ai("The official language used in Pakistani government, education, and daily communication. Often involved in literary competitions and official exam guidelines.")]
    #[rand_construct(p=0.01)]
    Urdu,

    #[ai("The official language used in Uzbekistani government, education, and daily communication. Commonly encountered in regional academic discussions and official training programs.")]
    #[rand_construct(p=0.001)]
    Uzbek,

    #[ai("The official language used in Vietnamese government, education, and daily communication. Regularly utilized in course syllabi and civil administration directives.")]
    #[rand_construct(p=0.01)]
    Vietnamese,

    #[ai("An official language used in Welsh government, education, and daily communication. Often present in cultural institutes and bilingual educational initiatives.")]
    #[rand_construct(p=0.001)]
    Welsh,

    #[ai("An indigenous language used in Yakut cultural traditions and daily life. Generally supported by communal language workshops and heritage-based documentation efforts.")]
    #[rand_construct(p=0.0001)]
    Yakut,

    #[ai("A regional language used in Yoruba communities and local markets. Frequently employed in local mediation sessions and small business signage.")]
    #[rand_construct(p=0.001)]
    Yoruba,

    #[ai("An official language used in South African government, education, and daily communication. Commonly found in academic papers and province-level informational materials.")]
    #[rand_construct(p=0.002)]
    Zulu,

    #[ai("An indigenous language used in North African cultural traditions and daily life. Typically fostered through household transmission and artisan cooperative activities.")]
    #[rand_construct(p=0.0005)]
    Berber,

    #[ai("An official language used in Vanuatuan government, education, and daily communication. Often part of local teacher training sessions and civic literacy drives.")]
    #[rand_construct(p=0.0001)]
    Bislama,

    #[ai("An official language used in Catalan regional government, education, and daily communication. Frequently involved in cultural exhibitions and regional publishing houses.")]
    #[rand_construct(p=0.001)]
    Catalan,

    #[ai("The official language used in Malawian government, education, and daily communication. Commonly reflected in national teacher guides and educational radio segments.")]
    #[rand_construct(p=0.001)]
    Chichewa,

    #[ai("An official language used in Comorian government, education, and daily communication. Regularly implemented in classroom guidelines and municipal announcement papers.")]
    #[rand_construct(p=0.0001)]
    Comorian,

    #[ai("An indigenous language used in Dinka cultural traditions and daily life. Usually reinforced in familial consultation events and community-structured language classes.")]
    #[rand_construct(p=0.0001)]
    Dinka,

    #[ai("The official language used in Bhutanese government, education, and daily communication. Often included in teacher reference manuals and academic research projects.")]
    #[rand_construct(p=0.0001)]
    Dzongkha,

    #[ai("An official language used in Fijian government, education, and daily communication. Common in local curriculum design and community literacy schemes.")]
    #[rand_construct(p=0.0001)]
    Fijian,

    #[ai("A regional language used in French Creole-speaking communities and local markets. Sometimes present in locally produced cultural magazines and informal learning groups.")]
    #[rand_construct(p=0.0005)]
    FrenchCreole,

    #[ai("An official language used in I-Kiribati government, education, and daily communication. Often vital in atoll-level schooling initiatives and civic engagement materials.")]
    #[rand_construct(p=0.0001)]
    Gilbertese,

    #[ai("A regional language used in Hakka communities and local markets. Occasionally featured in community reading clubs and small-scale cultural festivals.")]
    #[rand_construct(p=0.0005)]
    Hakka,

    #[ai("An official language used in Papua New Guinean government, education, and daily communication. Frequently integrated into literacy improvement campaigns and regional educational radio shows.")]
    #[rand_construct(p=0.0001)]
    HiriMotu,

    #[ai("An official language used in Irish government, education, and daily communication. Often supported through language promotion agencies and bilingual education policies.")]
    #[rand_construct(p=0.001)]
    IrishGaelic,

    #[ai("A regional language used in Jamaican communities and local markets. Sometimes included in grassroots literacy sessions and local narrative collections.")]
    #[rand_construct(p=0.0005)]
    JamaicanCreole,

    #[ai("A regional language used in Kikongo-speaking communities and local markets. Occasionally recorded in community newsletters and small language tutorial meetups.")]
    #[rand_construct(p=0.0005)]
    Kikongo,

    #[ai("The official language used in Rwandan government, education, and daily communication. Common in school reforms and national linguistic research centers.")]
    #[rand_construct(p=0.0005)]
    Kinyarwanda,

    #[ai("The official language used in Burundian government, education, and daily communication. Regularly part of nationwide exam content and teacher training guidelines.")]
    #[rand_construct(p=0.0005)]
    Kirundi,

    #[ai("A regional language used in Belizean communities and local markets. Often applied in small tutoring groups and informal neighborhood reading sessions.")]
    #[rand_construct(p=0.0005)]
    Kriol,

    #[ai("The official language used in Vatican government, education, and daily communication. Commonly employed in ecclesiastical documents and scholarly treatises.")]
    #[rand_construct(p=0.0001)]
    Latin,

    #[ai("A regional language used in Lingala-speaking communities and local markets. Periodically included in local radio discussions and basic literacy workshops.")]
    #[rand_construct(p=0.0005)]
    Lingala,

    #[ai("The official language used in Macedonian government, education, and daily communication. Frequently present in literary symposiums and government circulars.")]
    #[rand_construct(p=0.0005)]
    Macedonian,

    #[ai("The official language used in Marshallese government, education, and daily communication. Often integrated into elementary-level pedagogical approaches and civic instruction leaflets.")]
    #[rand_construct(p=0.0001)]
    Marshallese,

    #[ai("A regional language used in Mauritian communities and local markets. Sometimes involved in neighborhood language circles and cultural storytelling sessions.")]
    #[rand_construct(p=0.0005)]
    MauritianCreole,

    #[ai("The official language used in Nauruan government, education, and daily communication. Commonly referenced in teacher guidelines and local education reforms.")]
    #[rand_construct(p=0.0001)]
    Nauruan,

    #[ai("An official language used in Ndebele-speaking South African government, education, and daily communication. Often featured in multi-language classroom policies and regional broadcast announcements.")]
    #[rand_construct(p=0.0005)]
    Ndebele,

    #[ai("An indigenous language used in Nuer cultural traditions and daily life. Typically reinforced through family-based language maintenance and cultural dialogue forums.")]
    #[rand_construct(p=0.0001)]
    Nuer,

    #[ai("The official language used in Palauan government, education, and daily communication. Sometimes included in bilingual textbook initiatives and administrative instructions.")]
    #[rand_construct(p=0.0001)]
    Palauan,

    #[ai("A regional language used in Solomon Islands communities and local markets. Commonly utilized in inter-tribal communication efforts and simple educational leaflets.")]
    #[rand_construct(p=0.0005)]
    Pijin,

    #[ai("An official language used in Romansh-speaking Swiss government, education, and daily communication. Periodically employed in cultural research projects and region-specific academic material.")]
    #[rand_construct(p=0.0001)]
    Romansh,

    #[ai("The official language used in Central African government, education, and daily communication. Frequently integrated into national literacy strategies and civil documentation.")]
    #[rand_construct(p=0.0001)]
    Sango,

    #[ai("The official language used in Seychellois government, education, and daily communication. Often present in bilingual classroom instruction and community-led educational drives.")]
    #[rand_construct(p=0.0001)]
    SeychelloisCreole,

    #[ai("The official language used in Zimbabwean government, education, and daily communication. Regularly part of literacy advancement programs and regional language conferences.")]
    #[rand_construct(p=0.002)]
    Shona,

    #[ai("An official language used in Swati-speaking government, education, and daily communication. Occasionally included in educational policy revisions and public reading campaigns.")]
    #[rand_construct(p=0.0001)]
    Swati,

    #[ai("A regional language used in Taiwanese communities and local markets. Common in informal literacy initiatives and small group language practice sessions.")]
    #[rand_construct(p=0.0005)]
    TaiwaneseHokkien,

    #[ai("The official language used in Timor-Leste government, education, and daily communication. Often applied in nationwide literacy projects and educational governance documents.")]
    #[rand_construct(p=0.0005)]
    Tetum,

    #[ai("The official language used in Eritrean government, education, and daily communication. Commonly utilized in teacher training institutes and state-produced textbooks.")]
    #[rand_construct(p=0.0005)]
    Tigrinya,

    #[ai("An official language used in Papua New Guinean government, education, and daily communication. Frequently featured in public health literature and community lesson plans.")]
    #[rand_construct(p=0.0001)]
    TokPisin,

    #[ai("The official language used in Tongan government, education, and daily communication. Often included in culturally aligned school materials and national reference guides.")]
    #[rand_construct(p=0.0001)]
    Tongan,

    #[ai("An official language used in Tshiluba-speaking Congolese government, education, and daily communication. Regularly included in local educational standards and administrative formats.")]
    #[rand_construct(p=0.0001)]
    Tshiluba,

    #[ai("The official language used in Tuvaluan government, education, and daily communication. Common in grassroots literacy efforts and foundational teaching resources.")]
    #[rand_construct(p=0.0001)]
    Tuvaluan,

    #[ai("An indigenous language used in Mayan cultural traditions and daily life. Usually supported by seasonal language workshops and artisan documentation practices.")]
    #[rand_construct(p=0.0005)]
    VariousMayanIndigenous,

    #[ai("An indigenous language used in Mexican indigenous cultural traditions and daily life. Often sustained by community language cooperatives and traditional mentorships.")]
    #[rand_construct(p=0.0005)]
    VariousMexicanIndigenous,

    #[ai("A regional language used in Zambian communities and local markets. Occasionally employed in village educational groups and local language promotion campaigns.")]
    #[rand_construct(p=0.0005)]
    VariousBembaNyanjaLocal,

    #[ai("A regional language used in Indian communities and local markets. Sometimes implemented in grassroots literacy clubs and informal language improvement sessions.")]
    #[rand_construct(p=0.0005)]
    VariousIndianLocal,

    #[ai("A regional language used in Micronesian communities and local markets. Periodically included in island-level reading programs and small educational exchanges.")]
    #[rand_construct(p=0.0005)]
    VariousMicronesianLocal,

    #[ai("A regional language used in Russian communities and local markets. Known to surface in local cultural fairs and community-based linguistic initiatives.")]
    #[rand_construct(p=0.0005)]
    VariousRussianLocal,

    #[ai("A regional language used in South African communities and local markets. Sometimes applied in neighborhood education tasks and minimal-scale language classes.")]
    #[rand_construct(p=0.0005)]
    VariousSouthAfricanLocal,

    #[ai("An official language used in South African government, education, and daily communication. Regularly part of local literacy campaigns and widely integrated into community schooling materials.")]
    #[rand_construct(p=0.002)]
    Xhosa,

    #[ai("A regional language used in unknown communities and local markets. Potentially employed in limited-language skill-building activities and informal learning contexts.")]
    #[rand_construct(p=0.0000001)]
    Other(OtherLanguage),
}

impl Language {

    pub fn other(x: impl ToString) -> Self {
        Self::Other(OtherLanguage::new(x))
    }
}

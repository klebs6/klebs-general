crate::ix!();

/// Enumerates languages.
/// This enum covers a wide range of languages, including major world languages, regional languages,
/// and some languages with smaller populations for inclusivity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    Afrikaans,         //< Official language of South Africa and Namibia.
    Albanian,          //< Spoken primarily in Albania and Kosovo.
    Aleut,             //< Indigenous language of the Aleut people in Alaska and Russia.
    Amharic,           //< Official language of Ethiopia.
    Apache,            //< Indigenous language spoken by the Apache tribes.
    Arabic,            //< Spoken across many countries in the Middle East and North Africa.
    Armenian,          //< Official language of Armenia and spoken by Armenian communities worldwide.
    Assamese,          //< Spoken in the Indian state of Assam.
    Aymara,            //< Indigenous language spoken in Bolivia, Peru, and Chile.
    Azeri,             //< Official language of Azerbaijan.
    Bashkir,           //< Turkic language spoken in Russia.
    Basque,            //< Spoken in the Basque region of Spain and France.
    Belarusian,        //< Official language of Belarus.
    Bengali,           //< Spoken primarily in Bangladesh and the Indian state of West Bengal.
    Bosnian,           //< One of the official languages of Bosnia and Herzegovina.
    Breton,            //< A regional language in France (Brittany).
    Bulgarian,         //< Official language of Bulgaria.
    Burmese,           //< Official language of Myanmar (Burma).
    Cebuano,           //< Spoken in the Philippines.
    Chamorro,          //< Indigenous language of the Mariana Islands.
    Chechen,           //< Spoken in the Chechen Republic of Russia.
    Cherokee,          //< Indigenous language of the Cherokee people.
    Choctaw,           //< Indigenous language of the Choctaw people.
    Chuvash,           //< Turkic language spoken in Russia.
    Corsican,          //< Regional language spoken on the island of Corsica (France).
    Croatian,          //< Official language of Croatia.
    Czech,             //< Official language of the Czech Republic.
    Danish,            //< Official language of Denmark.
    Dhivehi,           //< Official language of the Maldives.
    Dutch,             //< Official language of the Netherlands and Belgium (Flemish).
    English,           //< Widely spoken global language, official in many countries.
    Estonian,          //< Official language of Estonia.
    Farsi,             //< Also known as Persian, spoken in Iran, Afghanistan, and Tajikistan.
    Filipino,          //< Official language of the Philippines.
    Finnish,           //< Official language of Finland.
    French,            //< Widely spoken global language, official in many countries.
    Galician,          //< Regional language in Spain, particularly in Galicia.
    Georgian,          //< Official language of Georgia.
    German,            //< Official language of Germany, Austria, and Switzerland.
    Greek,             //< Official language of Greece and Cyprus.
    Greenlandic,       //< Indigenous language of Greenland.
    Guarani,           //< Indigenous language spoken in Paraguay.
    Gujarati,          //< Spoken in the Indian state of Gujarat.
    HaitianCreole,     //< A major language in Haiti, derived from French.
    Hausa,             //< Spoken in West Africa, particularly in Nigeria and Niger.
    Hawaiian,          //< Indigenous language of Hawaii, though with a small speaker base.
    Hebrew,            //< Official language of Israel.
    Hindi,             //< One of the official languages of India.
    Hmong,             //< Significant ethnic language spoken in Southeast Asia and diaspora communities.
    Hungarian,         //< Official language of Hungary.
    Icelandic,         //< Official language of Iceland.
    Igbo,              //< Spoken in Nigeria.
    Ilocano,           //< Spoken in the northern Philippines.
    Indonesian,        //< Official language of Indonesia.
    Inuit,             //< Indigenous language spoken by the Inuit people.
    Italian,           //< Official language of Italy.
    Japanese,          //< Official language of Japan.
    Javanese,          //< Spoken on the island of Java, Indonesia.
    Kannada,           //< Spoken in the Indian state of Karnataka.
    Kazakh,            //< Official language of Kazakhstan.
    Khmer,             //< Official language of Cambodia.
    Korean,            //< Official language of North and South Korea.
    Kurdish,           //< Spoken by millions in the Middle East, especially in Iraq, Turkey, Syria, and Iran.
    Kyrgyz,            //< Official language of Kyrgyzstan.
    Lao,               //< Official language of Laos.
    Latvian,           //< Official language of Latvia.
    Lithuanian,        //< Official language of Lithuania.
    Luxembourgish,     //< Official language in Luxembourg.
    Malagasy,          //< Official language of Madagascar.
    Malay,             //< Spoken in Malaysia, Brunei, and Singapore.
    Malayalam,         //< Spoken in the Indian state of Kerala.
    Maltese,           //< Official language of Malta.
    Mandarin,          //< Official language of China and Taiwan, widely spoken in Singapore.
    Maori,             //< Indigenous language of New Zealand.
    Marathi,           //< Spoken in the Indian state of Maharashtra.
    Mohawk,            //< Indigenous language spoken by the Mohawk people in North America.
    Mongolian,         //< Spoken in Mongolia and some parts of China.
    Nahuatl,           //< Indigenous language of Mexico, spoken by the Nahua people.
    Navajo,            //< Indigenous language spoken in the Southwestern United States.
    Nepali,            //< Official language of Nepal.
    Norwegian,         //< Official language of Norway.
    Oriya,             //< Spoken in the Indian state of Odisha.
    Oromo,             //< Spoken in Ethiopia and Kenya.
    Ossetian,          //< Spoken in the Caucasus region.
    Palauan,           //< Spoken primarily in Palau.
    Pashto,            //< Official language of Afghanistan.
    Polish,            //< Official language of Poland.
    Portuguese,        //< Official language of Portugal, Brazil, and several other countries.
    Punjabi,           //< Spoken in India and Pakistan.
    Quechua,           //< Indigenous language of the Andean region of South America.
    RapaNui,           //< Indigenous language of Easter Island.
    Romanian,          //< Official language of Romania and Moldova.
    Russian,           //< Official language of Russia and widely spoken in former Soviet states.
    Samoan,            //< Official language of Samoa and American Samoa.
    ScottishGaelic,    //< Spoken in parts of Scotland, though less common today.
    Serbian,           //< Official language of Serbia.
    Sindhi,            //< Spoken in Pakistan and India.
    Sinhala,           //< (Sinhalese) – Official language of Sri Lanka.
    Slovak,            //< Official language of Slovakia.
    Slovenian,         //< Official language of Slovenia.
    Somali,            //< Official language of Somalia.
    Sotho,             //< Spoken in South Africa and Lesotho.
    Spanish,           //< Widely spoken global language, official in many countries.
    Swahili,           //< Widely spoken in East Africa.
    Swedish,           //< Official language of Sweden and Finland.
    Tagalog,           //< Widely spoken in the Philippines, basis of the Filipino language.
    Tahitian,          //< Indigenous language spoken in French Polynesia.
    Tajik,             //< Official language of Tajikistan.
    Tamil,             //< Spoken in India, Sri Lanka, and Malaysia.
    Tatar,             //< Spoken in Russia, particularly in Tatarstan.
    Telugu,            //< Spoken in the Indian states of Andhra Pradesh and Telangana.
    Thai,              //< Official language of Thailand.
    Tibetan,           //< Widely spoken in Tibet and surrounding areas.
    Tswana,            //< Spoken in Botswana and South Africa.
    Turkish,           //< Official language of Turkey and Cyprus.
    Turkmen,           //< Official language of Turkmenistan.
    Tuvinian,          //< Turkic language spoken in Russia.
    Uighur,            //< Turkic language spoken in China.
    Ukrainian,         //< Official language of Ukraine.
    Urdu,              //< Official language of Pakistan and widely spoken in India.
    Uzbek,             //< Official language of Uzbekistan.
    Vietnamese,        //< Official language of Vietnam.
    Welsh,             //< Official language of Wales in the UK.
    Yakut,             //< Indigenous language spoken in Siberia, Russia.
    Yoruba,            //< Spoken in Nigeria and surrounding countries.
    Zulu,              //< Widely spoken in South Africa.

    Other(String),     //< Placeholder for other languages not listed.
}

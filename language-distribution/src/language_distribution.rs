crate::ix!();

pub trait LanguageDistribution {
    fn language_distribution(&self) -> HashMap<Language, f64>;
}

impl LanguageDistribution for Country {
    fn language_distribution(&self) -> HashMap<Language, f64> {
        let mut map = HashMap::new();
        match self {
            Country::Afghanistan => {
                // Pashto, Dari (Persian)
                map.insert(Language::Pashto, 0.5);
                map.insert(Language::Farsi, 0.5);
                map
            },
            Country::Albania => {
                // Albanian
                map.insert(Language::Albanian, 1.0);
                map
            },
            Country::Algeria => {
                // Arabic, Berber
                map.insert(Language::Arabic, 0.9);
                map.insert(Language::Berber, 0.1);
                map
            },
            Country::Andorra => {
                // Catalan
                map.insert(Language::Catalan, 1.0);
                map
            },
            Country::Angola => {
                // Portuguese
                map.insert(Language::Portuguese, 1.0);
                map
            },
            Country::AntiguaAndBarbuda => {
                // English
                map.insert(Language::English, 1.0);
                map
            },
            Country::Argentina => {
                // Spanish
                map.insert(Language::Spanish, 1.0);
                map
            },
            Country::Armenia => {
                // Armenian
                map.insert(Language::Armenian, 1.0);
                map
            },
            Country::Australia => {
                // English (de facto)
                map.insert(Language::English, 1.0);
                map
            },
            Country::Austria => {
                // German
                map.insert(Language::German, 1.0);
                map
            },
            Country::Azerbaijan => {
                // Azerbaijani (Azeri)
                map.insert(Language::Azeri, 1.0);
                map
            },
            Country::Bahamas => {
                // English
                map.insert(Language::English, 1.0);
                map
            },
            Country::Bahrain => {
                // Arabic
                map.insert(Language::Arabic, 1.0);
                map
            },
            Country::Bangladesh => {
                // Bengali
                map.insert(Language::Bengali, 1.0);
                map
            },
            Country::Barbados => {
                // English
                map.insert(Language::English, 1.0);
                map
            },
            Country::Belarus => {
                // Belarusian, Russian
                map.insert(Language::Belarusian, 0.5);
                map.insert(Language::Russian, 0.5);
                map
            },
            Country::Belgium => {
                // Dutch, French, German
                map.insert(Language::Dutch, 0.55);
                map.insert(Language::French, 0.44);
                map.insert(Language::German, 0.01);
                map
            },
            Country::Belize => {
                // English (official), plus Spanish and others
                map.insert(Language::English, 0.5);
                map.insert(Language::Spanish, 0.3);
                map.insert(Language::Kriol, 0.2);
                map
            },
            Country::Benin => {
                // French
                map.insert(Language::French, 1.0);
                map
            },
            Country::Bhutan => {
                // Dzongkha
                map.insert(Language::Dzongkha, 1.0);
                map
            },
            Country::Bolivia => {
                // Spanish, Quechua, Aymara
                map.insert(Language::Spanish, 0.6);
                map.insert(Language::Quechua, 0.3);
                map.insert(Language::Aymara, 0.1);
                map
            },
            Country::BosniaAndHerzegovina => {
                // Bosnian, Croatian, Serbian
                map.insert(Language::Bosnian, 0.4);
                map.insert(Language::Croatian, 0.3);
                map.insert(Language::Serbian, 0.3);
                map
            },
            Country::Botswana => {
                // English, Tswana
                map.insert(Language::English, 0.3);
                map.insert(Language::Tswana, 0.7);
                map
            },
            Country::Brazil => {
                // Portuguese
                map.insert(Language::Portuguese, 1.0);
                map
            },
            Country::Brunei => {
                // Malay
                map.insert(Language::Malay, 1.0);
                map
            },
            Country::Bulgaria => {
                // Bulgarian
                map.insert(Language::Bulgarian, 1.0);
                map
            },
            Country::BurkinaFaso => {
                // French
                map.insert(Language::French, 1.0);
                map
            },
            Country::Burundi => {
                // Kirundi, French
                map.insert(Language::Kirundi, 0.8);
                map.insert(Language::French, 0.2);
                map
            },
            Country::Cambodia => {
                // Khmer
                map.insert(Language::Khmer, 1.0);
                map
            },
            Country::Cameroon => {
                // French, English
                map.insert(Language::French, 0.7);
                map.insert(Language::English, 0.3);
                map
            },
            Country::Canada => {
                // English, French
                map.insert(Language::English, 0.75);
                map.insert(Language::French, 0.25);
                map
            },
            Country::CapeVerde => {
                // Portuguese
                map.insert(Language::Portuguese, 1.0);
                map
            },
            Country::CentralAfricanRepublic => {
                // French, Sango
                map.insert(Language::French, 0.5);
                map.insert(Language::Sango, 0.5);
                map
            },
            Country::Chad => {
                // Arabic, French
                map.insert(Language::Arabic, 0.5);
                map.insert(Language::French, 0.5);
                map
            },
            Country::Chile => {
                // Spanish
                map.insert(Language::Spanish, 1.0);
                map
            },
            Country::China => {
                // Mandarin
                map.insert(Language::Mandarin, 1.0);
                map
            },
            Country::Colombia => {
                // Spanish
                map.insert(Language::Spanish, 1.0);
                map
            },
            Country::Comoros => {
                // Comorian, Arabic, French
                map.insert(Language::Comorian, 0.6);
                map.insert(Language::Arabic, 0.2);
                map.insert(Language::French, 0.2);
                map
            },
            Country::CongoBrazzaville => {
                // French, Lingala and other local languages
                map.insert(Language::French, 0.5);
                map.insert(Language::Lingala, 0.5);
                map
            },
            Country::CongoKinshasa => {
                // French + Lingala, Swahili, Kikongo, Tshiluba
                map.insert(Language::French, 0.4);
                map.insert(Language::Lingala, 0.2);
                map.insert(Language::Swahili, 0.2);
                map.insert(Language::Kikongo, 0.1);
                map.insert(Language::Tshiluba, 0.1);
                map
            },
            Country::CostaRica => {
                // Spanish
                map.insert(Language::Spanish, 1.0);
                map
            },
            Country::Croatia => {
                // Croatian
                map.insert(Language::Croatian, 1.0);
                map
            },
            Country::Cuba => {
                // Spanish
                map.insert(Language::Spanish, 1.0);
                map
            },
            Country::Cyprus => {
                // Greek, Turkish
                map.insert(Language::Greek, 0.8);
                map.insert(Language::Turkish, 0.2);
                map
            },
            Country::CzechRepublic => {
                // Czech
                map.insert(Language::Czech, 1.0);
                map
            },
            Country::Denmark => {
                // Danish
                map.insert(Language::Danish, 1.0);
                map
            },
            Country::Djibouti => {
                // French, Arabic
                map.insert(Language::French, 0.5);
                map.insert(Language::Arabic, 0.5);
                map
            },
            Country::Dominica => {
                // English
                map.insert(Language::English, 1.0);
                map
            },
            Country::DominicanRepublic => {
                // Spanish
                map.insert(Language::Spanish, 1.0);
                map
            },
            Country::Ecuador => {
                // Spanish, some Quechua
                map.insert(Language::Spanish, 0.95);
                map.insert(Language::Quechua, 0.05);
                map
            },
            Country::Egypt => {
                // Arabic
                map.insert(Language::Arabic, 1.0);
                map
            },
            Country::ElSalvador => {
                // Spanish
                map.insert(Language::Spanish, 1.0);
                map
            },
            Country::EquatorialGuinea => {
                // Spanish, French, Portuguese
                map.insert(Language::Spanish, 0.7);
                map.insert(Language::French, 0.2);
                map.insert(Language::Portuguese, 0.1);
                map
            },
            Country::Eritrea => {
                // Tigrinya, Arabic, English
                map.insert(Language::Tigrinya, 0.6);
                map.insert(Language::Arabic, 0.2);
                map.insert(Language::English, 0.2);
                map
            },
            Country::Estonia => {
                // Estonian
                map.insert(Language::Estonian, 1.0);
                map
            },
            Country::Eswatini => {
                // Swati, English
                map.insert(Language::Swati, 0.8);
                map.insert(Language::English, 0.2);
                map
            },
            Country::Ethiopia => {
                // Amharic (official), plus many others
                map.insert(Language::Amharic, 0.4);
                map.insert(Language::Oromo, 0.3);
                map.insert(Language::Tigrinya, 0.3);
                map
            },
            Country::Fiji => {
                // English, Fijian, Hindi
                map.insert(Language::English, 0.4);
                map.insert(Language::Fijian, 0.3);
                map.insert(Language::Hindi, 0.3);
                map
            },
            Country::Finland => {
                // Finnish, Swedish
                map.insert(Language::Finnish, 0.92);
                map.insert(Language::Swedish, 0.08);
                map
            },
            Country::France => {
                // French
                map.insert(Language::French, 0.95);
                map.insert(Language::English, 0.03);
                map.insert(Language::Breton, 0.01);
                map.insert(Language::Basque, 0.01);
                map
            },
            Country::Gabon => {
                // French
                map.insert(Language::French, 1.0);
                map
            },
            Country::Gambia => {
                // English
                map.insert(Language::English, 1.0);
                map
            },
            Country::Georgia => {
                // Georgian
                map.insert(Language::Georgian, 1.0);
                map
            },
            Country::Germany => {
                // German
                map.insert(Language::German, 0.94);
                map.insert(Language::English, 0.04);
                map.insert(Language::Turkish, 0.02);
                map
            },
            Country::Ghana => {
                // English
                map.insert(Language::English, 1.0);
                map
            },
            Country::Greece => {
                // Greek
                map.insert(Language::Greek, 1.0);
                map
            },
            Country::Grenada => {
                // English
                map.insert(Language::English, 1.0);
                map
            },
            Country::Guatemala => {
                // Spanish, plus Mayan languages
                map.insert(Language::Spanish, 0.9);
                map.insert(Language::VariousMayanIndigenous, 0.1);
                map
            },
            Country::Guinea => {
                // French
                map.insert(Language::French, 1.0);
                map
            },
            Country::GuineaBissau => {
                // Portuguese
                map.insert(Language::Portuguese, 1.0);
                map
            },
            Country::Guyana => {
                // English
                map.insert(Language::English, 1.0);
                map
            },
            Country::Haiti => {
                // Haitian Creole, French
                map.insert(Language::HaitianCreole, 0.8);
                map.insert(Language::French, 0.2);
                map
            },
            Country::Honduras => {
                // Spanish
                map.insert(Language::Spanish, 1.0);
                map
            },
            Country::Hungary => {
                // Hungarian
                map.insert(Language::Hungarian, 1.0);
                map
            },
            Country::Iceland => {
                // Icelandic
                map.insert(Language::Icelandic, 1.0);
                map
            },
            Country::India => {
                // Hindi, English, many others
                map.insert(Language::Hindi, 0.4);
                map.insert(Language::English, 0.1);
                map.insert(Language::VariousIndianLocal, 0.5);
                map
            },
            Country::Indonesia => {
                // Indonesian
                map.insert(Language::Indonesian, 1.0);
                map
            },
            Country::Iran => {
                // Persian (Farsi)
                map.insert(Language::Farsi, 1.0);
                map
            },
            Country::Iraq => {
                // Arabic, Kurdish
                map.insert(Language::Arabic, 0.8);
                map.insert(Language::Kurdish, 0.2);
                map
            },
            Country::Ireland => {
                // English, Irish Gaelic
                map.insert(Language::English, 0.98);
                map.insert(Language::IrishGaelic, 0.02);
                map
            },
            Country::Israel => {
                // Hebrew, Arabic
                map.insert(Language::Hebrew, 0.9);
                map.insert(Language::Arabic, 0.1);
                map
            },
            Country::Italy => {
                // Italian
                map.insert(Language::Italian, 0.97);
                map.insert(Language::German, 0.02);
                map.insert(Language::French, 0.01);
                map
            },
            Country::IvoryCoast => {
                // French
                map.insert(Language::French, 1.0);
                map
            },
            Country::Jamaica => {
                // English (with Jamaican Creole widely spoken)
                map.insert(Language::English, 0.5);
                map.insert(Language::JamaicanCreole, 0.5);
                map
            },
            Country::Japan => {
                // Japanese
                map.insert(Language::Japanese, 1.0);
                map
            },
            Country::Jordan => {
                // Arabic
                map.insert(Language::Arabic, 1.0);
                map
            },
            Country::Kazakhstan => {
                // Kazakh, Russian
                map.insert(Language::Kazakh, 0.6);
                map.insert(Language::Russian, 0.4);
                map
            },
            Country::Kenya => {
                // Swahili, English
                map.insert(Language::Swahili, 0.7);
                map.insert(Language::English, 0.3);
                map
            },
            Country::Kiribati => {
                // Gilbertese, English
                map.insert(Language::Gilbertese, 0.9);
                map.insert(Language::English, 0.1);
                map
            },
            Country::Kosovo => {
                // Albanian, Serbian
                map.insert(Language::Albanian, 0.9);
                map.insert(Language::Serbian, 0.1);
                map
            },
            Country::Kuwait => {
                // Arabic
                map.insert(Language::Arabic, 1.0);
                map
            },
            Country::Kyrgyzstan => {
                // Kyrgyz, Russian
                map.insert(Language::Kyrgyz, 0.7);
                map.insert(Language::Russian, 0.3);
                map
            },
            Country::Laos => {
                // Lao
                map.insert(Language::Lao, 1.0);
                map
            },
            Country::Latvia => {
                // Latvian
                map.insert(Language::Latvian, 0.9);
                map.insert(Language::Russian, 0.1);
                map
            },
            Country::Lebanon => {
                // Arabic, French widely used
                map.insert(Language::Arabic, 0.8);
                map.insert(Language::French, 0.2);
                map
            },
            Country::Lesotho => {
                // Sotho, English
                map.insert(Language::Sotho, 0.9);
                map.insert(Language::English, 0.1);
                map
            },
            Country::Liberia => {
                // English
                map.insert(Language::English, 1.0);
                map
            },
            Country::Libya => {
                // Arabic
                map.insert(Language::Arabic, 1.0);
                map
            },
            Country::Liechtenstein => {
                // German
                map.insert(Language::German, 1.0);
                map
            },
            Country::Lithuania => {
                // Lithuanian
                map.insert(Language::Lithuanian, 1.0);
                map
            },
            Country::Luxembourg => {
                // Luxembourgish, French, German
                map.insert(Language::Luxembourgish, 0.5);
                map.insert(Language::French, 0.3);
                map.insert(Language::German, 0.2);
                map
            },
            Country::Madagascar => {
                // Malagasy, French
                map.insert(Language::Malagasy, 0.8);
                map.insert(Language::French, 0.2);
                map
            },
            Country::Malawi => {
                // English, Chichewa
                map.insert(Language::English, 0.5);
                map.insert(Language::Chichewa, 0.5);
                map
            },
            Country::Malaysia => {
                // Malay
                map.insert(Language::Malay, 1.0);
                map
            },
            Country::Maldives => {
                // Dhivehi
                map.insert(Language::Dhivehi, 1.0);
                map
            },
            Country::Mali => {
                // French
                map.insert(Language::French, 1.0);
                map
            },
            Country::Malta => {
                // Maltese, English
                map.insert(Language::Maltese, 0.9);
                map.insert(Language::English, 0.1);
                map
            },
            Country::MarshallIslands => {
                // Marshallese, English
                map.insert(Language::Marshallese, 0.9);
                map.insert(Language::English, 0.1);
                map
            },
            Country::Mauritania => {
                // Arabic
                map.insert(Language::Arabic, 1.0);
                map
            },
            Country::Mauritius => {
                // English, French, Creole widely used
                map.insert(Language::English, 0.3);
                map.insert(Language::French, 0.3);
                map.insert(Language::MauritianCreole, 0.4);
                map
            },
            Country::Mexico => {
                // Spanish
                map.insert(Language::Spanish, 0.97);
                map.insert(Language::VariousMexicanIndigenous, 0.03);
                map
            },
            Country::Micronesia => {
                // English, plus local languages
                map.insert(Language::English, 0.5);
                map.insert(Language::VariousMicronesianLocal, 0.5);
                map
            },
            Country::Moldova => {
                // Romanian (Moldovan)
                map.insert(Language::Romanian, 1.0);
                map
            },
            Country::Monaco => {
                // French
                map.insert(Language::French, 1.0);
                map
            },
            Country::Mongolia => {
                // Mongolian
                map.insert(Language::Mongolian, 1.0);
                map
            },
            Country::Montenegro => {
                // Montenegrin (very close to Serbian)
                map.insert(Language::Serbian, 1.0);
                map
            },
            Country::Morocco => {
                // Arabic, Berber
                map.insert(Language::Arabic, 0.8);
                map.insert(Language::Berber, 0.2);
                map
            },
            Country::Mozambique => {
                // Portuguese
                map.insert(Language::Portuguese, 1.0);
                map
            },
            Country::Myanmar => {
                // Burmese
                map.insert(Language::Burmese, 1.0);
                map
            },
            Country::Namibia => {
                // English
                map.insert(Language::English, 1.0);
                map
            },
            Country::Nauru => {
                // Nauruan, English
                map.insert(Language::Nauruan, 0.9);
                map.insert(Language::English, 0.1);
                map
            },
            Country::Nepal => {
                // Nepali
                map.insert(Language::Nepali, 1.0);
                map
            },
            Country::Netherlands => {
                // Dutch
                map.insert(Language::Dutch, 0.97);
                map.insert(Language::English, 0.03);
                map
            },
            Country::NewZealand => {
                // English, Maori
                map.insert(Language::English, 0.9);
                map.insert(Language::Maori, 0.1);
                map
            },
            Country::Nicaragua => {
                // Spanish
                map.insert(Language::Spanish, 1.0);
                map
            },
            Country::Niger => {
                // French
                map.insert(Language::French, 1.0);
                map
            },
            Country::Nigeria => {
                // English + many local languages
                map.insert(Language::English, 0.5);
                map.insert(Language::Hausa, 0.2);
                map.insert(Language::Yoruba, 0.15);
                map.insert(Language::Igbo, 0.15);
                map
            },
            Country::NorthKorea => {
                // Korean
                map.insert(Language::Korean, 1.0);
                map
            },
            Country::NorthMacedonia => {
                // Macedonian, Albanian
                map.insert(Language::Macedonian, 0.85);
                map.insert(Language::Albanian, 0.15);
                map
            },
            Country::Norway => {
                // Norwegian
                map.insert(Language::Norwegian, 1.0);
                map
            },
            Country::Oman => {
                // Arabic
                map.insert(Language::Arabic, 1.0);
                map
            },
            Country::Pakistan => {
                // Urdu, English plus regional
                map.insert(Language::Urdu, 0.6);
                map.insert(Language::English, 0.4);
                map
            },
            Country::Palau => {
                // Palauan, English
                map.insert(Language::Palauan, 0.8);
                map.insert(Language::English, 0.2);
                map
            },
            Country::Palestine => {
                // Arabic
                map.insert(Language::Arabic, 1.0);
                map
            },
            Country::Panama => {
                // Spanish
                map.insert(Language::Spanish, 1.0);
                map
            },
            Country::PapuaNewGuinea => {
                // English, Tok Pisin, Hiri Motu
                map.insert(Language::English, 0.3);
                map.insert(Language::TokPisin, 0.4);
                map.insert(Language::HiriMotu, 0.3);
                map
            },
            Country::Paraguay => {
                // Spanish, Guarani
                map.insert(Language::Spanish, 0.5);
                map.insert(Language::Guarani, 0.5);
                map
            },
            Country::Peru => {
                // Spanish, Quechua, Aymara
                map.insert(Language::Spanish, 0.8);
                map.insert(Language::Quechua, 0.18);
                map.insert(Language::Aymara, 0.02);
                map
            },
            Country::Philippines => {
                // Filipino, English
                map.insert(Language::Filipino, 0.5);
                map.insert(Language::English, 0.5);
                map
            },
            Country::Poland => {
                // Polish
                map.insert(Language::Polish, 1.0);
                map
            },
            Country::Portugal => {
                // Portuguese
                map.insert(Language::Portuguese, 1.0);
                map
            },
            Country::Qatar => {
                // Arabic
                map.insert(Language::Arabic, 1.0);
                map
            },
            Country::Romania => {
                // Romanian
                map.insert(Language::Romanian, 1.0);
                map
            },
            Country::Russia => {
                // Russian + minorities
                map.insert(Language::Russian, 0.95);
                map.insert(Language::Tatar, 0.01);
                map.insert(Language::Chechen, 0.01);
                map.insert(Language::VariousRussianLocal, 0.03);
                map
            },
            Country::Rwanda => {
                // Kinyarwanda, French, English
                map.insert(Language::Kinyarwanda, 0.8);
                map.insert(Language::French, 0.1);
                map.insert(Language::English, 0.1);
                map
            },
            Country::SaintKittsAndNevis => {
                // English
                map.insert(Language::English, 1.0);
                map
            },
            Country::SaintLucia => {
                // English, French Creole
                map.insert(Language::English, 0.7);
                map.insert(Language::FrenchCreole, 0.3);
                map
            },
            Country::SaintVincentAndTheGrenadines => {
                // English
                map.insert(Language::English, 1.0);
                map
            },
            Country::Samoa => {
                // Samoan, English
                map.insert(Language::Samoan, 0.9);
                map.insert(Language::English, 0.1);
                map
            },
            Country::SanMarino => {
                // Italian
                map.insert(Language::Italian, 1.0);
                map
            },
            Country::SaoTomeAndPrincipe => {
                // Portuguese
                map.insert(Language::Portuguese, 1.0);
                map
            },
            Country::SaudiArabia => {
                // Arabic
                map.insert(Language::Arabic, 1.0);
                map
            },
            Country::Senegal => {
                // French
                map.insert(Language::French, 1.0);
                map
            },
            Country::Serbia => {
                // Serbian
                map.insert(Language::Serbian, 1.0);
                map
            },
            Country::Seychelles => {
                // Seychellois Creole, French, English
                map.insert(Language::SeychelloisCreole, 0.7);
                map.insert(Language::French, 0.15);
                map.insert(Language::English, 0.15);
                map
            },
            Country::SierraLeone => {
                // English
                map.insert(Language::English, 1.0);
                map
            },
            Country::Singapore => {
                // English, Malay, Mandarin, Tamil
                map.insert(Language::English, 0.4);
                map.insert(Language::Malay, 0.2);
                map.insert(Language::Mandarin, 0.35);
                map.insert(Language::Tamil, 0.05);
                map
            },
            Country::Slovakia => {
                // Slovak
                map.insert(Language::Slovak, 1.0);
                map
            },
            Country::Slovenia => {
                // Slovenian
                map.insert(Language::Slovenian, 1.0);
                map
            },
            Country::SolomonIslands => {
                // English + Pijin
                map.insert(Language::English, 0.5);
                map.insert(Language::Pijin, 0.5);
                map
            },
            Country::Somalia => {
                // Somali, Arabic
                map.insert(Language::Somali, 0.8);
                map.insert(Language::Arabic, 0.2);
                map
            },
            Country::SouthAfrica => {
                // 11 official languages: Zulu, Xhosa, Afrikaans, English, etc.
                // Let's give a rough distribution:
                map.insert(Language::Zulu, 0.22);
                map.insert(Language::Xhosa, 0.16);
                map.insert(Language::Afrikaans, 0.13);
                map.insert(Language::English, 0.09);
                // Others combined:
                map.insert(Language::VariousSouthAfricanLocal, 0.4);
                map
            },
            Country::SouthKorea => {
                // Korean
                map.insert(Language::Korean, 1.0);
                map
            },
            Country::SouthSudan => {
                // English + many local languages, Dinka, Nuer
                map.insert(Language::English, 0.3);
                map.insert(Language::Dinka, 0.35);
                map.insert(Language::Nuer, 0.35);
                map
            },
            Country::Spain => {
                // Spanish, Catalan, Galician, Basque
                map.insert(Language::Spanish, 0.88);
                map.insert(Language::Catalan, 0.06);
                map.insert(Language::Galician, 0.02);
                map.insert(Language::Basque, 0.02);
                map
            },
            Country::SriLanka => {
                // Sinhala, Tamil
                map.insert(Language::Sinhala, 0.7);
                map.insert(Language::Tamil, 0.3);
                map
            },
            Country::Sudan => {
                // Arabic, English
                map.insert(Language::Arabic, 0.9);
                map.insert(Language::English, 0.1);
                map
            },
            Country::Suriname => {
                // Dutch
                map.insert(Language::Dutch, 1.0);
                map
            },
            Country::Sweden => {
                // Swedish
                map.insert(Language::Swedish, 1.0);
                map
            },
            Country::Switzerland => {
                // German, French, Italian, Romansh
                map.insert(Language::German, 0.65);
                map.insert(Language::French, 0.23);
                map.insert(Language::Italian, 0.08);
                map.insert(Language::Romansh, 0.04);
                map
            },
            Country::Syria => {
                // Arabic
                map.insert(Language::Arabic, 1.0);
                map
            },
            Country::Taiwan => {
                // Mandarin, Taiwanese Hokkien, Hakka
                map.insert(Language::Mandarin, 0.7);
                map.insert(Language::TaiwaneseHokkien, 0.25);
                map.insert(Language::Hakka, 0.05);
                map
            },
            Country::Tajikistan => {
                // Tajik
                map.insert(Language::Tajik, 1.0);
                map
            },
            Country::Tanzania => {
                // Swahili, English
                map.insert(Language::Swahili, 0.9);
                map.insert(Language::English, 0.1);
                map
            },
            Country::Thailand => {
                // Thai
                map.insert(Language::Thai, 1.0);
                map
            },
            Country::TimorLeste => {
                // Tetum, Portuguese
                map.insert(Language::Tetum, 0.7);
                map.insert(Language::Portuguese, 0.3);
                map
            },
            Country::Togo => {
                // French
                map.insert(Language::French, 1.0);
                map
            },
            Country::Tonga => {
                // Tongan, English
                map.insert(Language::Tongan, 0.9);
                map.insert(Language::English, 0.1);
                map
            },
            Country::TrinidadAndTobago => {
                // English
                map.insert(Language::English, 1.0);
                map
            },
            Country::Tunisia => {
                // Arabic
                map.insert(Language::Arabic, 1.0);
                map
            },
            Country::Turkey => {
                // Turkish
                map.insert(Language::Turkish, 1.0);
                map
            },
            Country::Turkmenistan => {
                // Turkmen
                map.insert(Language::Turkmen, 1.0);
                map
            },
            Country::Tuvalu => {
                // Tuvaluan, English
                map.insert(Language::Tuvaluan, 0.9);
                map.insert(Language::English, 0.1);
                map
            },
            Country::Uganda => {
                // English, Swahili + many local languages
                map.insert(Language::English, 0.5);
                map.insert(Language::Swahili, 0.5);
                map
            },
            Country::UAE => {
                // Arabic
                map.insert(Language::Arabic, 1.0);
                map
            },
            Country::Ukraine => {
                // Ukrainian, Russian
                map.insert(Language::Ukrainian, 0.9);
                map.insert(Language::Russian, 0.1);
                map
            },
            Country::UnitedKingdom => {
                // English, Welsh, Scottish Gaelic
                map.insert(Language::English, 0.95);
                map.insert(Language::Welsh, 0.03);
                map.insert(Language::ScottishGaelic, 0.02);
                map
            },
            Country::USA => {
                // English (de facto), Spanish widely spoken
                map.insert(Language::English, 0.8);
                map.insert(Language::Spanish, 0.2);
                map
            },
            Country::Uruguay => {
                // Spanish
                map.insert(Language::Spanish, 1.0);
                map
            },
            Country::Uzbekistan => {
                // Uzbek
                map.insert(Language::Uzbek, 1.0);
                map
            },
            Country::Vanuatu => {
                // Bislama, English, French
                map.insert(Language::Bislama, 0.6);
                map.insert(Language::English, 0.2);
                map.insert(Language::French, 0.2);
                map
            },
            Country::VaticanCity => {
                // Italian, Latin
                map.insert(Language::Italian, 0.99);
                map.insert(Language::Latin, 0.01);
                map
            },
            Country::Venezuela => {
                // Spanish
                map.insert(Language::Spanish, 1.0);
                map
            },
            Country::Vietnam => {
                // Vietnamese
                map.insert(Language::Vietnamese, 1.0);
                map
            },
            Country::Yemen => {
                // Arabic
                map.insert(Language::Arabic, 1.0);
                map
            },
            Country::Zambia => {
                // English + local languages
                map.insert(Language::English, 0.5);
                map.insert(Language::VariousBembaNyanjaLocal, 0.5);
                map
            },
            Country::Zimbabwe => {
                // 16 official languages, including English, Shona, Ndebele
                map.insert(Language::English, 0.3);
                map.insert(Language::Shona, 0.35);
                map.insert(Language::Ndebele, 0.35);
                map
            },
        }
    }
}

#[cfg(test)]
mod language_distribution_tests {
    use super::*;

    #[test]
    fn test_distribution_for_simple_case() {
        let c = Country::Japan; // Known single language: Japanese
        let dist = c.language_distribution();
        assert_eq!(dist.len(), 1, "Japan should have exactly one language entry");
        assert_eq!(*dist.get(&Language::Japanese).unwrap_or(&0.0), 1.0, "Japanese should be 100% in Japan");
    }

    #[test]
    fn test_distribution_multiple_languages() {
        let c = Country::Belgium;
        let dist = c.language_distribution();
        assert!(dist.contains_key(&Language::Dutch), "Belgium should have Dutch");
        assert!(dist.contains_key(&Language::French), "Belgium should have French");
        assert!(dist.contains_key(&Language::German), "Belgium should have German");

        let dutch = dist.get(&Language::Dutch).copied().unwrap_or(0.0);
        let french = dist.get(&Language::French).copied().unwrap_or(0.0);
        let german = dist.get(&Language::German).copied().unwrap_or(0.0);
        let sum = dutch + french + german;

        assert!((sum - 1.0).abs() < f64::EPSILON, "Sum of language proportions should be 1.0");
    }

    #[test]
    fn test_distribution_with_indigenous() {
        let c = Country::Bolivia;
        let dist = c.language_distribution();
        // Bolivia: Spanish (0.6), Quechua (0.3), Aymara (0.1)
        let spanish = dist.get(&Language::Spanish).copied().unwrap_or(0.0);
        let quechua = dist.get(&Language::Quechua).copied().unwrap_or(0.0);
        let aymara = dist.get(&Language::Aymara).copied().unwrap_or(0.0);

        assert!((spanish - 0.6).abs() < f64::EPSILON, "Spanish should be 0.6 in Bolivia");
        assert!((quechua - 0.3).abs() < f64::EPSILON, "Quechua should be 0.3 in Bolivia");
        assert!((aymara - 0.1).abs() < f64::EPSILON, "Aymara should be 0.1 in Bolivia");
    }
}

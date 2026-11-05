settings-automation-rotation-desc = हर बार अंतराल पूरा होने पर पूल से एक यादृच्छिक वॉलपेपर लागू करता है।
settings-automation-rotation-say = हर { $interval } पर, { $val }{ $pool }{ $end } में से एक यादृच्छिक चुनाव।
settings-automation-rotation-off = बंद। वॉलपेपर तभी बदलता है जब आप खुद चुनते हैं।
settings-automation-pool = { $count ->
        [none] खाली पूल
        [one] { $first }
        [two] { $first } और { $second }
       *[three] { $first }, { $second } और { $third }
    }
settings-automation-pool-favourites = { $favourites ->
        [only] पसंदीदा { $pool }
       *[all] { $pool }
    }
settings-automation-pool-images = छवियों
settings-automation-pool-videos = वीडियो
settings-automation-pool-engine = Wallpaper Engine आइटम
settings-automation-pool-heading = पूल में शामिल
settings-automation-timing = समय
settings-automation-interval-hint = कम से कम 10 से
settings-automation-schedule-desc = नियम समय, सप्ताह के दिन, मौसम, पावर और डिस्प्ले के आधार पर वॉलपेपर और थीम चुनते हैं। पहला मेल खाने वाला नियम लागू होता है।
settings-automation-schedule-say = { $start ->
        [applies] नियम चालू हैं; पहला मेल खाने वाला नियम { $val }स्टार्टअप पर लागू होता है{ $end }। आपका खुद चुना वॉलपेपर फिर भी प्राथमिक रहता है।
       *[waits] नियम चालू हैं; पहला मेल खाने वाला नियम { $val }अगले बदलाव की प्रतीक्षा करता है{ $end }। आपका खुद चुना वॉलपेपर फिर भी प्राथमिक रहता है।
    }
settings-automation-schedule-off = बंद। जब तक आप इसे चालू नहीं करते, आपके नियम सुरक्षित रहते हैं।
settings-automation-start-hint = सेवा शुरू होने पर पहला मेल खाने वाला नियम
settings-automation-editor-hint = शर्त ब्लॉकों से नियम बनाएँ
settings-automation-location = स्थान और मौसम
settings-automation-location-desc = सूर्योदय और सूर्यास्त के समय और मौसम की स्थिति के लिए उपयोग होता है।
settings-automation-location-say = { $city } का मौसम, { $lat }, { $lon } पर सूर्य का समय। मौसम के अनुसार बदलाव: { $mode }।
settings-automation-no-city = कोई शहर नहीं
settings-automation-no-latitude = कोई अक्षांश नहीं
settings-automation-no-longitude = कोई देशांतर नहीं
settings-automation-location-hint = निर्देशांक इस्तेमाल करने के लिए शहर खाली छोड़ें। अक्षांश उत्तर की ओर और देशांतर पूर्व की ओर धनात्मक है।
settings-automation-startup = स्टार्टअप और सूचनाएँ
settings-automation-startup-desc = सेवा शुरू होने और वॉलपेपर बदलने पर क्या होता है।
settings-automation-startup-say = { $restore ->
        [restores] स्टार्टअप पर Skwd { $val }पिछला वॉलपेपर फिर से लागू करता है{ $end }।
       *[leaves] स्टार्टअप पर Skwd { $val }डेस्कटॉप को जैसा है वैसा छोड़ देता है{ $end }।
    } { $notify ->
        [notifies] वॉलपेपर बदलने पर { $val }सूचना भेजी जाती है{ $end }।
       *[silent] वॉलपेपर { $val }चुपचाप बदलते हैं{ $end }।
    }
settings-automation-commands = वॉलपेपर लागू होने के बाद
settings-automation-commands-desc = हर बदलाव के बाद अपनी कमांड चलाएँ, जैसे किसी दूसरे वॉलपेपर टूल को चलाने के लिए।
settings-automation-command-count = { $count ->
        [one] { $val }{ $count } कमांड{ $end } हर बदलाव के बाद चलती है।
       *[other] { $val }{ $count } कमांड{ $end } हर बदलाव के बाद चलती हैं।
    }
settings-automation-commands-say = { $listed ->
        [none] हर बदलाव के बाद { $val }कोई कमांड{ $end } नहीं चलती।
       *[some] { $commands }
    } { $apply ->
        [commands] { $rerun ->
            [also] Skwd { $val }लागू करने का काम आपकी कमांड पर छोड़ता है{ $end }, और कमांड { $val }स्टार्टअप रीस्टोर पर भी चलती हैं{ $end }।
           *[skip] Skwd { $val }लागू करने का काम आपकी कमांड पर छोड़ता है{ $end }, और कमांड { $val }स्टार्टअप रीस्टोर को छोड़ देती हैं{ $end }।
        }
       *[itself] { $rerun ->
            [also] Skwd { $val }वॉलपेपर खुद लागू करता है{ $end }, और कमांड { $val }स्टार्टअप रीस्टोर पर भी चलती हैं{ $end }।
           *[skip] Skwd { $val }वॉलपेपर खुद लागू करता है{ $end }, और कमांड { $val }स्टार्टअप रीस्टोर को छोड़ देती हैं{ $end }।
        }
    }
settings-automation-pick-only-hint = आपकी कमांड इसे लागू करती हैं
settings-automation-commands-heading = कमांड
settings-automation-placeholders = कमांड में %path%, %thumb%, %type% और %name% का उपयोग करें।
settings-automation-minutes = { $count } मिनट
settings-automation-mode-off = बंद
settings-automation-mode-outdoors = बाहर के मौसम के साथ चलें
settings-automation-mode-playlists = अपनी प्लेलिस्ट से चुनें
settings-automation-mode-semantic = अपनी लाइब्रेरी में खोजें
settings-automation-condition-clear = साफ़
settings-automation-condition-cloudy = बादल
settings-automation-condition-rainy = बारिश
settings-automation-condition-snowy = बर्फ़
settings-automation-condition-stormy = तूफ़ान
settings-automation-condition-foggy = कोहरा
settings-automation-condition-windy = तेज़ हवा
settings-automation-display-another = दूसरा
settings-automation-display-exclude = बाहर करें
settings-automation-display-keep = रखें
settings-automation-playlists-heading = हर मौसम की प्लेलिस्ट
settings-automation-hours = { $count } घंटे

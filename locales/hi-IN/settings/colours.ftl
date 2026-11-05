settings-colours-source = रंग कहाँ से आता है
settings-colours-source-desc = हर वॉलपेपर का अनुसरण करें, एक पैलेट रखें या रंग न बदलें।
settings-colours-source-follow = हर वॉलपेपर का अनुसरण करते हैं
settings-colours-source-fixed = एक स्थिर पैलेट से आते हैं
settings-colours-source-off = जैसे हैं वैसे रहते हैं
settings-colours-source-say = रंग { $policy }।
settings-colours-source-made = रंग { $policy }, बनाने वाला { $authority }।
settings-colours-end4 = end-4
settings-colours-control-menu = रंग नियंत्रण · मिले हुए शेल
settings-colours-generator-menu = जनरेटर · केवल इंस्टॉल किए
settings-colours-theme-menu = थीम · प्रीसेट, आपके, कस्टम बीज
settings-colours-all-presets = सभी { $count } प्रीसेट…
settings-colours-generator = जनरेटर
settings-colours-generator-desc = वह प्रोग्राम जो वॉलपेपर को पैलेट बनाता है।
settings-colours-generator-say = { $engine } पैलेट बनाता है।
settings-colours-generator-seeded = { $engine } पैलेट बनाता है, बीज स्लॉट { $index }, कंट्रास्ट { $contrast }।
settings-colours-generator-shell = { $shell } वॉलपेपर से पैलेट बनाता है।
settings-colours-palette = पैलेट
settings-colours-palette-desc = सभी वॉलपेपर के लिए एक पैलेट।
settings-colours-palette-say = सभी वॉलपेपर के लिए { $theme }।
settings-colours-idle = जब तक रंग वैसे ही रहें, उपयोग नहीं होता।
settings-colours-index-hint = कौन सा पैलेट स्लॉट बीज रंग बने
settings-colours-contrast-hint = -1 से 1
settings-colours-wallust-hint = wallust पैलेट तय करें, जैसे dark16। खाली: मोड के अनुसार
settings-colours-seeds-hint = कस्टम बीज इसका उपयोग करते हैं
settings-colours-config-hint = बाहरी Matugen
settings-colours-command-hint = %config% %path% %scheme% %mode% %index%
settings-colours-auto = स्वचालित
settings-colours-look = रूप
settings-colours-look-desc = रंग कितना फैलता है, उसकी फिनिश, और हल्का या गहरा।
settings-colours-look-say = { $scheme } स्वभाव, { $finish } फिनिश, वेरिएंट { $variant }।
settings-colours-look-imported = { $shell } का पैलेट, { $variant } वेरिएंट के रूप में आयातित।
settings-colours-look-shell = { $shell } अपना रूप खुद तय करता है।
settings-colours-look-fixed = स्थिर पैलेट अपना रूप खुद तय करता है।
settings-colours-no-designer = थीम डिज़ाइनर वॉलपेपर का अनुसरण या स्थिर पैलेट के साथ उपलब्ध है।
settings-colours-designer-hint = पैलेट हाथ से बदलें या बनाएँ
settings-colours-go = रंग कहाँ जाते हैं
settings-colours-go-desc = ऐप थीम जो पैलेट बदलने पर उसका अनुसरण करती हैं।
settings-colours-find-hint = ऐप इंस्टॉल करने के बाद फिर जाँचें
settings-colours-matugen-hint = Matugen आउटपुट का स्विच
settings-colours-reload-hint = फ़ाइल लिखने के बाद चलता है
settings-colours-output-heading = आउटपुट { $count }
settings-colours-shells = शेल
settings-colours-shells-desc = Noctalia और Dank Material Shell Skwd के रंग ले सकते हैं।
settings-colours-dms-hint = DMS को Auto (वॉलपेपर) उपयोग करना होगा
settings-colours-mode-hint = हर रंग योजना के साथ लागू
settings-colours-bin-hint = यदि PATH में नहीं है
settings-colours-lock = लॉक स्क्रीन
settings-colours-lock-desc = लॉक स्क्रीन जो आपका वॉलपेपर दिखाती हैं।
settings-colours-lock-follow = अंतिम लागू वॉलपेपर
settings-colours-lock-still = एक स्थिर चित्र
settings-colours-sddm-hint = व्यवस्थापक की मंज़ूरी चाहिए
settings-colours-go-say = { $val }{ $all } में से { $on } ऐप{ $end } { $followers ->
        [one] आपके रंगों का
       *[some] आपके रंगों का
    } अनुसरण करते हैं{ $changed ->
        [one] ; { $val }{ $name } का कॉन्फ़िग Skwd के बाहर बदला{ $end }।
        [some] ; { $val }{ $count } ऐप Skwd के बाहर बदले{ $end }।
       *[none] ।
    }
settings-colours-outputs-say = { $state ->
        [none] अभी कोई इंटीग्रेशन टेम्पलेट नहीं बनाता।
        [single] { $val }1 इंटीग्रेशन{ $end } हर रंग बदलाव के बाद टेम्पलेट को फ़ाइल में लिखता है।
       *[many] { $val }{ $count } इंटीग्रेशन{ $end } हर रंग बदलाव के बाद टेम्पलेट फ़ाइलों में लिखते हैं।
    }
settings-colours-shells-say = Noctalia { $val }{ $noctalia ->
        [follows] Skwd के रंगों का अनुसरण करता है
       *[keeps] अपने रंग रखता है
    }{ $end }; DMS { $val }{ $dms ->
        [follows] Skwd के रंगों का अनुसरण करता है
       *[keeps] अपने रंग रखता है
    }{ $end }{ $caelestia ->
        [yes] ; Caelestia के वॉलपेपर सिंक होते हैं।
       *[no] ।
    }
settings-colours-lock-say = { $state ->
        [none] कोई लॉक स्क्रीन सेट नहीं। नई { $mode } का उपयोग करती हैं।
        [one] { $names } { $mode } का उपयोग करता है।
       *[some] { $names } { $mode } का उपयोग करते हैं।
    }
settings-colours-lock-say-static = { $state ->
        [none] कोई लॉक स्क्रीन सेट नहीं। नई { $mode } का उपयोग करती हैं: { $image }।
        [one] { $names } { $mode } का उपयोग करता है: { $image }।
       *[some] { $names } { $mode } का उपयोग करते हैं: { $image }।
    }
settings-menu-more = और

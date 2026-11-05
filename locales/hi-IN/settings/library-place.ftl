settings-library-folders-desc = इन फ़ोल्डरों और उनके सबफ़ोल्डरों में चित्र और वीडियो खोजें।
settings-library-folders-say = { $val }{ $folders }{ $end } स्कैन हो रहे हैं।{ $state ->
        [polling] { " " }पोलिंग हर { $every } वह पकड़ती है जो नेटिव निगरानी से छूटता है।{ $unread }
        [native] { " " }नेटिव फ़ाइल निगरानी चालू है; पोलिंग रुकी है।
        [down] { " " }नेटिव निगरानी विफल रही और पोलिंग बंद है।
       *[unknown] { "" }
    }
settings-library-unread = { $count ->
        [one] { " " }{ $count } फ़ोल्डर पढ़ा नहीं जा सकता।
       *[other] { " " }{ $count } फ़ोल्डर पढ़े नहीं जा सकते।
    }
settings-library-watching = निगरानी
settings-library-poll-hint = नेटवर्क या FUSE फ़ोल्डरों के लिए
settings-library-interval-hint = बदलने के बाद skwd-walld फिर शुरू करें
settings-library-excluded-desc = इन फ़ोल्डरों और सबफ़ोल्डरों को छोड़ें। मौजूदा मेटाडेटा सहेजा रहता है।
settings-library-excluded-say = { $val }{ $folders }{ $end } कभी स्कैन नहीं होते।{ $outside }
settings-library-outside = { $count ->
        [one] { " " }{ $count } किसी वॉलपेपर फ़ोल्डर के भीतर नहीं, इसलिए कुछ नहीं छोड़ता।
       *[other] { " " }{ $count } किसी वॉलपेपर फ़ोल्डर के भीतर नहीं, इसलिए कुछ नहीं छोड़ते।
    }
settings-library-excluded-hint = बाहर रखे फ़ोल्डर कभी स्कैन नहीं होते, इसलिए कोई प्रोफ़ाइल उन्हें नहीं दिखा सकती। फ़ोल्डर रखकर सार्वजनिक रूप से छिपाने के लिए { $link } का उपयोग करें।
settings-library-profiles-desc = तय करें कि पिकर, फ़ोल्डर फ़िल्टर, रैंडम वॉलपेपर और शेड्यूल कौन से फ़ोल्डर लें।
settings-library-profiles-say = सक्रिय प्रोफ़ाइल { $state ->
        [none] { $menu }, जो सभी फ़ोल्डर दिखाती है।
       *[some] { $menu }: { $summary }।
    } Ctrl+H या बार का छिपा हुआ बटन छिपे वॉलपेपर दिखाता है, कार्ड से छिपाए गए भी, अगली शुरुआत तक।
settings-library-active = सक्रिय प्रोफ़ाइल
settings-library-terminal = टर्मिनल से बदलें
settings-library-tree-note = छिपाएँ: Ctrl+H तक रैंडम से बाहर। बाहर रखें: Ctrl+H पर भी बाहर। फ़ोल्डर अपने सबफ़ोल्डर भी तय करता है।
settings-library-show-tip = पिकर और रैंडम में
settings-library-hide-tip = Ctrl+H तक, कभी रैंडम नहीं
settings-library-exclude-tip = इस प्रोफ़ाइल से बाहर, Ctrl+H छोड़ता है
settings-library-from = { $link } से
settings-library-excluded-inside = भीतर { $count } बाहर रखे
settings-library-both-inside = भीतर { $hidden } छिपे, { $excluded } बाहर रखे
settings-library-storage = स्टोरेज और ट्रैश
settings-library-storage-desc = छोटी WebP प्रतियाँ, रिकवरी स्टोरेज और रंग बदली प्रतियाँ।
settings-library-storage-say = { $optimise ->
        [on] नए चित्र { $quality } गुणवत्ता पर WebP बनते हैं, सीमा { $size }।
       *[off] नए चित्र जैसे हैं वैसे रहते हैं। सभी ऑप्टिमाइज़ { $quality } गुणवत्ता लेता है, सीमा { $size }।
    } { $trash ->
        [on] ट्रैश के मूल { $days } रहते हैं, फिर हटते हैं।
       *[off] ट्रैश के मूल तब तक रहते हैं जब तक आप न हटाएँ।
    } { $recolour ->
        [on] नए वॉलपेपर को रंग बदली प्रति भी मिलती है।
       *[off] नए वॉलपेपर का रंग नहीं बदलता।
    }
settings-library-retention-hint = ट्रैश चित्र रखने के दिन
settings-library-optimise-hint = पूरी लाइब्रेरी के चित्र ऑप्टिमाइज़ करें
settings-library-recolour = रंग बदलें
settings-library-thumbnails = थंबनेल और मरम्मत
settings-library-thumbnails-desc = कुछ गलत दिखे तो प्रीव्यू और रंग डेटा फिर बनाएँ।
settings-library-thumbnails-say = पुनर्निर्माण में एक साथ { $jobs } तक थंबनेल बनते हैं।
settings-library-rebuild = फिर बनाएँ
settings-library-start-over = नए सिरे से
settings-library-recompute-hint = मौजूदा थंबनेल से
settings-library-clear-hint = अगले स्कैन पर थंबनेल फिर बनेंगे

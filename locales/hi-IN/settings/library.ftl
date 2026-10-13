settings-library-watch-section-desc = skwd-wall के बाहर जोड़ी गई फ़ाइलें पहचानें और लाइब्रेरी अद्यतन रखें।
settings-library-watch-fallback-label = पोलिंग फ़ॉलबैक
settings-library-watch-fallback-desc = केवल उन लाइब्रेरी फ़ोल्डरों की जाँच करें जिन्हें नेटिव फ़ाइल निगरानी देख नहीं सकती। बदलाव छूटने वाले नेटवर्क या FUSE माउंट के लिए इसे चालू करें, फिर skwd-walld पुनः आरंभ करें।
settings-library-watch-interval-label = पोलिंग अंतराल
settings-library-watch-interval-desc = सीमित जाँचों के बीच इतने सेकंड प्रतीक्षा करें। कम मान बदलाव जल्दी पकड़ते हैं पर फ़ाइल सिस्टम को अधिक बार पढ़ते हैं। इसे बदलने के बाद skwd-walld पुनः आरंभ करें।
settings-library-watch-unknown-label = निगरानी की स्थिति अनुपलब्ध
settings-library-watch-unknown-desc = यह डेमन लाइब्रेरी निगरानी की स्थिति नहीं बताता। skwd-walld अपडेट या पुनः आरंभ करें।
settings-library-watch-poll-failed-label = पोलिंग कोई लाइब्रेरी फ़ोल्डर पढ़ नहीं पा रही
settings-library-watch-poll-failed-desc = जाँचें कि हर कॉन्फ़िगर किया गया लाइब्रेरी फ़ोल्डर माउंट और पठनीय है। पोलिंग { $interval } सेकंड में फिर से प्रयास करेगी।
settings-library-watch-polling-label = पोलिंग फ़ॉलबैक सक्रिय
settings-library-watch-polling-desc = { $count ->
    [one] { $count } लाइब्रेरी फ़ोल्डर
   *[other] { $count } लाइब्रेरी फ़ोल्डरों
    } में नेटिव निगरानी विफल रही। हर { $interval } सेकंड में अधिकतम { $budget } प्रविष्टियाँ जाँची जाती हैं। अंतिम सफल समन्वय: { $convergence }।
settings-library-watch-recovering-label = नेटिव निगरानी बहाल हुई
settings-library-watch-recovering-desc = नेटिव निगरानी फिर से सक्रिय है। लाइब्रेरी को अद्यतन घोषित करने से पहले पूरा हैंड-ऑफ़ स्कैन अभी चल रहा है।
settings-library-watch-unavailable-label = लाइब्रेरी निगरानी अनुपलब्ध
settings-library-watch-unavailable-desc = नेटिव फ़ाइल निगरानी विफल रही और पोलिंग फ़ॉलबैक बंद है। पोलिंग फ़ॉलबैक चालू करें, फिर skwd-walld पुनः आरंभ करें।
settings-library-watch-recovered-label = नेटिव निगरानी बहाल
settings-library-watch-recovered-desc = नेटिव निगरानी और उसका हैंड-ऑफ़ स्कैन अद्यतन हैं। अंतिम सफल समन्वय: { $convergence }।
settings-library-watch-native-label = नेटिव फ़ाइल निगरानी
settings-library-watch-native-desc = हर लाइब्रेरी फ़ोल्डर के लिए फ़ाइल सिस्टम इवेंट सक्रिय हैं। पोलिंग निष्क्रिय है।
settings-library-watch-convergence-never = अभी पूरा नहीं हुआ
settings-library-watch-convergence-seconds = { $value ->
    [one] { $value } सेकंड पहले
   *[other] { $value } सेकंड पहले
    }
settings-library-watch-convergence-minutes = { $value ->
    [one] { $value } मिनट पहले
   *[other] { $value } मिनट पहले
    }
settings-library-watch-convergence-hours = { $value ->
    [one] { $value } घंटा पहले
   *[other] { $value } घंटे पहले
    }
settings-profiles-card = प्रोफ़ाइल
settings-profiles-card-desc = तय करें कि पिकर, फ़ोल्डर फ़िल्टर, रैंडम वॉलपेपर और शेड्यूल कौन-से फ़ोल्डर इस्तेमाल करें। छिपे फ़ोल्डर बार के छिपे हुए टॉगल या उसके कीबाइंड से वापस आते हैं, और हर लॉन्च पर वे फिर छिप जाते हैं।
settings-profiles-everything = सब
settings-profiles-name-label = नाम
settings-profiles-name-placeholder = प्रोफ़ाइल का नाम
settings-profiles-state-show = दिखाएँ
settings-profiles-state-hide = छिपाएँ
settings-profiles-state-exclude = बाहर रखें
settings-profiles-workshop = Wallpaper Engine
settings-profiles-workshop-desc = Workshop आइटम के फ़ोल्डर नहीं होते, इसलिए यह सब पर लागू होता है। किसी एक वॉलपेपर को उसके कार्ड के पीछे से छिपाएँ।
settings-profiles-summary = { $hidden } छिपे, { $excluded } बाहर
settings-profiles-summary-none = सभी फ़ोल्डर दिखाता है
settings-profiles-delete-action = प्रोफ़ाइल हटाएँ
settings-profiles-add-label = नई प्रोफ़ाइल
settings-profiles-new-name = नई प्रोफ़ाइल
settings-profiles-tree-label = इस प्रोफ़ाइल के फ़ोल्डर
settings-profiles-tree-desc = दिखाएँ फ़ोल्डर को पिकर और रैंडम में रखता है। छिपाएँ उसे बार के छिपे हुए टॉगल या उसके कीबाइंड दबाने तक नज़र से दूर रखता है। बाहर रखें उसे प्रोफ़ाइल से बाहर रखता है, इसलिए टॉगल उसे छोड़ देता है। फ़ोल्डर अपने उप-फ़ोल्डर भी तय करता है; अपवाद के लिए बाद में कोई उप-फ़ोल्डर बदलें।
settings-profiles-never-scanned = कभी स्कैन नहीं होता
settings-profiles-mixed = अंदर मिला-जुला
settings-profiles-hidden-inside = { $count ->
    [one] अंदर { $count } छिपा
   *[other] अंदर { $count } छिपे
    }
settings-profiles-copy-action = कॉपी करें
settings-profiles-helm-desc = इसे सक्रिय प्रोफ़ाइल बनाता है, इसलिए रोटेशन और शेड्यूल इसे मानते हैं।
settings-profiles-flag-desc = इस प्रोफ़ाइल में एक पिकर खोलता है और सक्रिय प्रोफ़ाइल को वैसा ही रहने देता है।
settings-profiles-make-active = सक्रिय करें
settings-profiles-active-tag = सक्रिय
settings-profiles-builtin-tag = अंतर्निहित
settings-profiles-filter-placeholder = फ़ोल्डर छाँटें
settings-profiles-no-match = कोई फ़ोल्डर मेल नहीं खाता।
settings-profiles-switch-label = इस प्रोफ़ाइल पर जाएँ
settings-profiles-switch-desc = skwd-helm सहेजी गई प्रोफ़ाइल बदलता है, इसलिए रोटेशन और शेड्यूल उसी का पालन करते हैं। लॉन्च फ़्लैग इस प्रोफ़ाइल में एक पिकर खोलता है और सहेजी गई प्रोफ़ाइल को वैसा ही छोड़ता है।
settings-profiles-hidden-note-label = छिपे वॉलपेपर
settings-profiles-hidden-note-desc = बार का छिपाएँ टॉगल (Ctrl+H) सक्रिय प्रोफ़ाइल के छिपे फ़ोल्डर और कार्ड के पीछे से छिपाए गए हर वॉलपेपर को दिखाता है। हर लॉन्च पर वे फिर छिप जाते हैं।
settings-profiles-tree-note = कभी स्कैन नहीं, लाइब्रेरी › फ़ोल्डर के बहिष्कृत फ़ोल्डरों से आता है, इसलिए कोई प्रोफ़ाइल उसे नहीं दिखा सकती। यहाँ बहिष्कृत करना केवल फ़ोल्डर को इस प्रोफ़ाइल से बाहर रखता है। कार्ड से छिपाए गए वॉलपेपर हर प्रोफ़ाइल में, सब कुछ सहित, छिपे दिखाएँ तक छिपे रहते हैं। .private जैसे डॉट-फ़ोल्डर यहाँ सामान्य फ़ोल्डर हैं।
settings-profiles-legend-show = पिकर और रैंडम में।
settings-profiles-legend-hide = बार के छिपाएँ टॉगल या Ctrl+H तक। कभी रैंडम नहीं।
settings-profiles-legend-exclude = इस प्रोफ़ाइल से बाहर। छिपे दिखाएँ इसे छोड़ देता है।
settings-profiles-everything-desc = सब कुछ हर स्कैन किए गए फ़ोल्डर और हर Workshop आइटम को दिखाता है। कुछ को छिपाने या बहिष्कृत करने के लिए नई प्रोफ़ाइल बनाएँ।
settings-profiles-delete-hint = हटाने के लिए फिर क्लिक करें। वॉलपेपर और फ़ोल्डर अपनी जगह रहते हैं।

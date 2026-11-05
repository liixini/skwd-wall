settings-system-power-desc = पिकर और वॉलपेपर की बैटरी सीमाएँ अलग हैं।
settings-system-power-say = बैटरी पर { $cap ->
        [none] पिकर पर कोई अतिरिक्त सीमा नहीं है ({ $fps })
       *[cap] पिकर { $fps } पर सीमित रहता है
    }{ $idle ->
        [off] , केवल सामान्य निष्क्रिय विराम लागू होता है ({ $seconds })
       *[on] , बिना इनपुट के { $seconds } बाद वीडियो रुक जाता है
    }, और { $detail ->
        [full] वॉलपेपर पूरा विवरण बनाए रखते हैं
       *[performance] वॉलपेपर प्रदर्शन मोड में चले जाते हैं
    }।
settings-system-power-off = बंद। बैटरी पर पिकर और वॉलपेपर उन्हीं सीमाओं के साथ चलते हैं जो चार्जर पर होती हैं।
settings-system-power-source = मौजूदा पावर स्रोत: { $source }
settings-system-always = हमेशा
settings-system-on-battery = बैटरी पर · जब सेवर चालू हो
settings-system-max-fps-hint = डिस्प्ले की रिफ़्रेश दर, या ठंडे लैपटॉप के लिए 60
settings-system-idle-hint = 0 बंद करता है
settings-system-battery-fps-hint = इसमें और अधिकतम FPS में जो कम हो · 0 बंद करता है
settings-system-battery-idle-hint = छोटा वाला लागू होता है · 0 बंद करता है
settings-system-battery-wallpaper-hint = सीधा वीडियो, 30 fps दृश्य
settings-system-gpu-desc = बदलने के बाद पिकर फिर से खोलें।
settings-system-gpu-say = { $mode ->
        [low] पिकर { $gpu } का उपयोग करता है: यह इंटीग्रेटेड GPU माँगता है।
        [high] पिकर { $gpu } का उपयोग करता है: यह डिस्क्रीट GPU माँगता है।
        [none] पिकर { $gpu } का उपयोग करता है: ड्राइवर तय करता है।
        [auto] पिकर { $gpu } का उपयोग करता है: बैटरी सेवर सक्रिय होने पर यह कम पावर वाला GPU पसंद करता है।
       *[custom] पिकर { $gpu } का उपयोग करता है, जिसे वह पहचानता नहीं, इसलिए यह स्वचालित की तरह व्यवहार करता है।
    }
settings-system-language-desc = बदलाव तुरंत लागू होते हैं और सहेजे जाते हैं।
settings-system-language-say = { $follow ->
        [desktop] इंटरफ़ेस की भाषा { $language } है, डेस्कटॉप के अनुसार। वॉलपेपर के नाम और टैग नहीं बदलते।
       *[chosen] इंटरफ़ेस की भाषा { $language } है। वॉलपेपर के नाम और टैग नहीं बदलते।
    }
settings-system-bug-title = बग रिपोर्ट
settings-system-bug-desc = किसी इश्यू में जोड़ने के लिए एक फ़ाइल।
settings-system-bug-say = संस्करण, परिवेश और हाल के लॉग एक फ़ाइल में इकट्ठा करें।

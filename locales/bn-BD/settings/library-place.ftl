settings-library-folders-desc = ছবি ও ভিডিওর জন্য এই ফোল্ডার ও সাবফোল্ডারগুলো স্ক্যান করুন।
settings-library-folders-say = { $val }{ $folders }{ $end } স্ক্যান হচ্ছে।{ $state ->
        [polling] { " " }পোলিং প্রতি { $every } নেটিভ নজরদারির বাদ পড়া ধরে।{ $unread }
        [native] { " " }নেটিভ ফাইল নজরদারি চালু; পোলিং অলস।
        [down] { " " }নেটিভ নজরদারি ব্যর্থ এবং পোলিং বন্ধ।
       *[unknown] { "" }
    }
settings-library-unread = { $count ->
        [one] { " " }{ $count }টি ফোল্ডার পড়া যাচ্ছে না।
       *[other] { " " }{ $count }টি ফোল্ডার পড়া যাচ্ছে না।
    }
settings-library-watching = নজরদারি
settings-library-poll-hint = নেটওয়ার্ক বা FUSE ফোল্ডারের জন্য
settings-library-interval-hint = বদলানোর পর skwd-walld আবার চালু করুন
settings-library-excluded-desc = এই ফোল্ডার ও সাবফোল্ডার বাদ দিন। বিদ্যমান মেটাডেটা থাকবে।
settings-library-excluded-say = { $val }{ $folders }{ $end } কখনও স্ক্যান হয় না।{ $outside }
settings-library-outside = { $count ->
        [one] { " " }{ $count }টি কোনো ওয়ালপেপার ফোল্ডারের ভেতরে নেই, কিছুই বাদ দেয় না।
       *[other] { " " }{ $count }টি কোনো ওয়ালপেপার ফোল্ডারের ভেতরে নেই, কিছুই বাদ দেয় না।
    }
settings-library-excluded-hint = বাদ দেওয়া ফোল্ডার কখনও স্ক্যান হয় না, তাই কোনো প্রোফাইল দেখাতে পারে না। ফোল্ডার রেখে প্রকাশ্যে লুকাতে { $link } ব্যবহার করুন।
settings-library-profiles-desc = পিকার, ফোল্ডার ফিল্টার, এলোমেলো ওয়ালপেপার ও সময়সূচি কোন ফোল্ডার নেবে তা ঠিক করুন।
settings-library-profiles-say = সক্রিয় প্রোফাইল { $state ->
        [none] { $menu }, যা সব ফোল্ডার দেখায়।
       *[some] { $menu }: { $summary }।
    } Ctrl+H বা বারের লুকানো বোতাম লুকানো ওয়ালপেপার দেখায়, কার্ড থেকে লুকানোগুলোসহ, পরের চালু পর্যন্ত।
settings-library-active = সক্রিয় প্রোফাইল
settings-library-terminal = টার্মিনাল থেকে বদলান
settings-library-tree-note = লুকান: Ctrl+H পর্যন্ত এলোমেলোর বাইরে। বাদ দিন: Ctrl+H-তেও বাইরে। ফোল্ডার তার সাবফোল্ডারও ঠিক করে।
settings-library-show-tip = পিকার ও এলোমেলোতে
settings-library-hide-tip = Ctrl+H পর্যন্ত, কখনও এলোমেলো নয়
settings-library-exclude-tip = এই প্রোফাইলের বাইরে, Ctrl+H বাদ দেয়
settings-library-from = { $link } থেকে
settings-library-excluded-inside = ভেতরে { $count }টি বাদ
settings-library-both-inside = ভেতরে { $hidden }টি লুকানো, { $excluded }টি বাদ
settings-library-storage = স্টোরেজ ও ট্র্যাশ
settings-library-storage-desc = ছোট WebP কপি, পুনরুদ্ধার স্টোরেজ ও নতুন রঙের কপি।
settings-library-storage-say = { $optimise ->
        [on] নতুন ছবি { $quality } মানে WebP হয়, সীমা { $size }।
       *[off] নতুন ছবি যেমন আছে তেমন থাকে। সব অপ্টিমাইজ { $quality } মান নেয়, সীমা { $size }।
    } { $trash ->
        [on] ট্র্যাশের মূল ফাইল { $days } থাকে, তারপর মুছে যায়।
       *[off] ট্র্যাশের মূল ফাইল আপনি না মোছা পর্যন্ত থাকে।
    } { $recolour ->
        [on] নতুন ওয়ালপেপার নতুন রঙের কপিও পায়।
       *[off] নতুন ওয়ালপেপারের রঙ বদলায় না।
    }
settings-library-retention-hint = ট্র্যাশের ছবি রাখার দিন
settings-library-optimise-hint = পুরো লাইব্রেরির ছবি অপ্টিমাইজ করুন
settings-library-recolour = রঙ বদল
settings-library-thumbnails = থাম্বনেইল ও মেরামত
settings-library-thumbnails-desc = কিছু ভুল দেখালে প্রিভিউ ও রঙের ডেটা আবার বানান।
settings-library-thumbnails-say = পুনর্নির্মাণে একসঙ্গে সর্বোচ্চ { $jobs }টি থাম্বনেইল হয়।
settings-library-rebuild = আবার বানান
settings-library-start-over = নতুন করে
settings-library-recompute-hint = বিদ্যমান থাম্বনেইল থেকে
settings-library-clear-hint = পরের স্ক্যানে থাম্বনেইল আবার হবে

settings-library-watch-section-desc = skwd-wall-এর বাইরে যোগ করা ফাইল শনাক্ত করে লাইব্রেরি হালনাগাদ রাখুন।
settings-library-watch-fallback-label = পোলিং ফলব্যাক
settings-library-watch-fallback-desc = শুধু সেই লাইব্রেরি ফোল্ডারগুলো পরীক্ষা করে, যেগুলো নেটিভ ফাইল পর্যবেক্ষণ দেখতে পারে না। পরিবর্তন ধরতে না পারা নেটওয়ার্ক বা FUSE মাউন্টের জন্য এটি চালু করুন, তারপর skwd-walld পুনরায় চালু করুন।
settings-library-watch-interval-label = পোলিং বিরতি
settings-library-watch-interval-desc = সীমিত পরীক্ষাগুলোর মধ্যে এত সেকেন্ড অপেক্ষা করে। কম মানে পরিবর্তন তাড়াতাড়ি ধরা পড়ে, কিন্তু ফাইলসিস্টেম বেশিবার পড়া হয়। এটি বদলানোর পর skwd-walld পুনরায় চালু করুন।
settings-library-watch-unknown-label = পর্যবেক্ষকের অবস্থা অনুপলব্ধ
settings-library-watch-unknown-desc = এই ডেমন লাইব্রেরি পর্যবেক্ষণের অবস্থা জানায় না। skwd-walld হালনাগাদ বা পুনরায় চালু করুন।
settings-library-watch-poll-failed-label = পোলিং একটি লাইব্রেরি ফোল্ডার পড়তে পারছে না
settings-library-watch-poll-failed-desc = কনফিগার করা প্রতিটি লাইব্রেরি ফোল্ডার মাউন্ট করা ও পঠনযোগ্য কি না পরীক্ষা করুন। পোলিং { $interval } সেকেন্ড পর আবার চেষ্টা করবে।
settings-library-watch-polling-label = পোলিং ফলব্যাক সক্রিয়
settings-library-watch-polling-desc = { $count ->
    [one] { $count }টি লাইব্রেরি ফোল্ডারে
   *[other] মোট { $count }টি লাইব্রেরি ফোল্ডারে
    } নেটিভ পর্যবেক্ষণ ব্যর্থ হয়েছে। প্রতি { $interval } সেকেন্ডে সর্বোচ্চ { $budget }টি এন্ট্রি পরীক্ষা করা হয়। সর্বশেষ সফল সমন্বয়: { $convergence }।
settings-library-watch-recovering-label = নেটিভ পর্যবেক্ষণ পুনরুদ্ধার হয়েছে
settings-library-watch-recovering-desc = নেটিভ পর্যবেক্ষক আবার সক্রিয়। লাইব্রেরি হালনাগাদ ঘোষণার আগে একটি পূর্ণ হস্তান্তর স্ক্যান এখনও চলছে।
settings-library-watch-unavailable-label = লাইব্রেরি পর্যবেক্ষণ অনুপলব্ধ
settings-library-watch-unavailable-desc = নেটিভ ফাইল পর্যবেক্ষণ ব্যর্থ হয়েছে এবং পোলিং ফলব্যাক বন্ধ। পোলিং ফলব্যাক চালু করুন, তারপর skwd-walld পুনরায় চালু করুন।
settings-library-watch-recovered-label = নেটিভ পর্যবেক্ষণ পুনঃস্থাপিত
settings-library-watch-recovered-desc = নেটিভ পর্যবেক্ষক ও তার হস্তান্তর স্ক্যান হালনাগাদ। সর্বশেষ সফল সমন্বয়: { $convergence }।
settings-library-watch-native-label = নেটিভ ফাইল পর্যবেক্ষণ
settings-library-watch-native-desc = প্রতিটি লাইব্রেরি ফোল্ডারে ফাইলসিস্টেম ইভেন্ট সক্রিয়। পোলিং নিষ্ক্রিয়।
settings-library-watch-convergence-never = এখনও সম্পন্ন হয়নি
settings-library-watch-convergence-seconds = { $value ->
    [one] { $value } সেকেন্ড আগে
   *[other] { $value } সেকেন্ড আগে
    }
settings-library-watch-convergence-minutes = { $value ->
    [one] { $value } মিনিট আগে
   *[other] { $value } মিনিট আগে
    }
settings-library-watch-convergence-hours = { $value ->
    [one] { $value } ঘণ্টা আগে
   *[other] { $value } ঘণ্টা আগে
    }
settings-profiles-card = প্রোফাইল
settings-profiles-card-desc = পিকার, ফোল্ডার ফিল্টার, এলোমেলো ওয়ালপেপার আর সময়সূচি কোন ফোল্ডার ব্যবহার করবে তা ঠিক করুন। লুকানো ফোল্ডার বারের লুকানো টগল বা তার কীবাইন্ড দিয়ে ফিরে আসে, আর প্রতিবার চালু করলে আবার লুকানো থাকে।
settings-profiles-everything = সব
settings-profiles-name-label = নাম
settings-profiles-name-placeholder = প্রোফাইলের নাম
settings-profiles-state-show = দেখান
settings-profiles-state-hide = লুকান
settings-profiles-state-exclude = বাদ দিন
settings-profiles-workshop = Wallpaper Engine
settings-profiles-workshop-desc = Workshop আইটেমের কোনো ফোল্ডার নেই, তাই এটি সবগুলোর ওপর প্রযোজ্য। একটি ওয়ালপেপার তার কার্ডের পেছন থেকে লুকান।
settings-profiles-summary = { $hidden }টি লুকানো, { $excluded }টি বাদ
settings-profiles-summary-none = সব ফোল্ডার দেখায়
settings-profiles-delete-action = প্রোফাইল মুছুন
settings-profiles-add-label = নতুন প্রোফাইল
settings-profiles-new-name = নতুন প্রোফাইল
settings-profiles-tree-label = এই প্রোফাইলের ফোল্ডার
settings-profiles-tree-desc = দেখান ফোল্ডারটিকে পিকার আর এলোমেলোতে রাখে। লুকান বারের লুকানো টগল বা তার কীবাইন্ড চাপা পর্যন্ত এটিকে আড়ালে রাখে। বাদ দিন এটিকে প্রোফাইলের বাইরে রাখে, তাই টগল এটিকে এড়িয়ে যায়। ফোল্ডার তার সাবফোল্ডারও ঠিক করে; ব্যতিক্রমের জন্য পরে কোনো সাবফোল্ডার বদলান।
settings-profiles-never-scanned = কখনো স্ক্যান হয় না
settings-profiles-mixed = ভেতরে মিশ্র
settings-profiles-hidden-inside = { $count ->
    [one] ভেতরে { $count }টি লুকানো
   *[other] ভেতরে { $count }টি লুকানো
    }
settings-profiles-copy-action = কপি করুন
settings-profiles-helm-desc = এটিকে সক্রিয় প্রোফাইল করে, তাই ঘূর্ণন আর সময়সূচি এটি মেনে চলে।
settings-profiles-flag-desc = এই প্রোফাইলে একটি পিকার খোলে এবং সক্রিয় প্রোফাইল অপরিবর্তিত রাখে।
settings-profiles-make-active = সক্রিয় করুন
settings-profiles-active-tag = সক্রিয়
settings-profiles-builtin-tag = অন্তর্নির্মিত
settings-profiles-filter-placeholder = ফোল্ডার ছাঁকুন
settings-profiles-no-match = কোনো ফোল্ডার মেলেনি।
settings-profiles-switch-label = এই প্রোফাইলে যান
settings-profiles-switch-desc = skwd-helm সংরক্ষিত প্রোফাইল বদলায়, তাই রোটেশন ও সময়সূচি তা অনুসরণ করে। লঞ্চ ফ্ল্যাগ এই প্রোফাইলে একটি পিকার খোলে এবং সংরক্ষিত প্রোফাইল অপরিবর্তিত রাখে।
settings-profiles-hidden-note-label = লুকানো ওয়ালপেপার
settings-profiles-hidden-note-desc = বারের লুকানো টগল (Ctrl+H) সক্রিয় প্রোফাইলের লুকানো ফোল্ডার আর কার্ডের পেছন থেকে লুকানো প্রতিটি ওয়ালপেপার দেখায়। প্রতিটি লঞ্চে সেগুলো আবার লুকিয়ে যায়।
settings-profiles-tree-note = কখনো স্ক্যান হয়নি আসে লাইব্রেরি › ফোল্ডার-এর বাদ দেওয়া ফোল্ডার থেকে, তাই কোনো প্রোফাইল তা দেখাতে পারে না। এখানে বাদ দেওয়া শুধু এই প্রোফাইল থেকে একটি ফোল্ডার বাদ রাখে। কার্ড থেকে লুকানো ওয়ালপেপার সব প্রোফাইলে, সবকিছু সহ, লুকানো দেখান পর্যন্ত লুকানো থাকে। .private-এর মতো ডট-ফোল্ডার এখানে সাধারণ ফোল্ডার।
settings-profiles-legend-show = পিকার ও র‍্যান্ডমে।
settings-profiles-legend-hide = বারের লুকানো টগল বা Ctrl+H পর্যন্ত। কখনো র‍্যান্ডম নয়।
settings-profiles-legend-exclude = এই প্রোফাইল থেকে বাদ। লুকানো দেখান এটি এড়িয়ে যায়।
settings-profiles-everything-desc = সবকিছু প্রতিটি স্ক্যান করা ফোল্ডার ও প্রতিটি Workshop আইটেম দেখায়। কিছু লুকাতে বা বাদ দিতে নতুন প্রোফাইল তৈরি করুন।
settings-profiles-delete-hint = মুছতে আবার ক্লিক করুন। ওয়ালপেপার ও ফোল্ডার যেখানে আছে সেখানেই থাকে।

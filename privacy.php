<?php
require_once __DIR__ . '/config/app.php';
$pageTitle = 'Privacy Policy — Sera ya Faragha | Mbilinyi Tech Solutions';
$page = 'legal';
require __DIR__ . '/partials/head.php';
require __DIR__ . '/partials/legal-render.php';
?>
<body class="antialiased">
<?php require __DIR__ . '/partials/header.php'; ?>

<?php mts_legal_page([
    'badge'      => 'PRIVACY & DATA PROTECTION',
    'badgeColor' => 'emerald',
    'titleEn'    => 'Privacy Policy',
    'titleSw'    => 'Sera ya Faragha',
    'introEn'    => 'Mbilinyi Tech Solutions collects only what is needed to run our website, deliver your project and communicate with you. We follow the Tanzania Personal Data Protection Act, 2022 and common-sense security. We never sell your data.',
    'introSw'    => 'Mbilinyi Tech Solutions inakusanya tu kinachohitajika kuendesha tovuti yetu, kutekeleza mradi wako na kuwasiliana nawe. Tunafuata Sheria ya Ulinzi wa Data ya Kibinafsi ya Tanzania, 2022 na usalama wa kawaida. Hatuuzi data yako.',
    'updated'    => date('d F Y'),
    'sections'   => [
        ['hEn' => 'What we collect', 'hSw' => 'Tunachokusanya', 'p' => [
            ['en' => 'Account data: your name, email address, phone number, company or role, and a securely hashed password.',
             'sw' => 'Data ya akaunti: jina lako, barua pepe, namba ya simu, kampuni au cheo, na neno la siri lililotajwa kwa usalama.'],
            ['en' => 'Project data: requirements, documents, images, feedback and messages you send through the portal or by email.',
             'sw' => 'Data ya mradi: mahitaji, nyaraka, picha, maoni na ujumbe unayotuma kupitia portal au barua pepe.'],
            ['en' => 'Technical data: browser type, device, pages viewed, referring page, approximate country and timestamps, used for security and to understand which pages are useful.',
             'sw' => 'Data ya kiteknolojia: aina ya kivinjari, kifaa, kurasa zilizotazamwa, kurasa iliyokupeleka, nchi ya karibu na saa, zinazotumika kwa usalama na kuelewa ni kurasa zipi zinafaa.'],
            ['en' => 'Payment references: transaction references you send as proof of payment. We never store full card numbers.',
             'sw' => 'Marejeleo ya malipo: marejeleo ya muamala unayotuma kama uthibitisho wa malipo. Hatuhifadhi namba kamili za kadi.'],
        ]],
        ['hEn' => 'Why we use it', 'hSw' => 'Kwa nini tunaitumika', 'ul' => [
            ['en' => 'To create your account, sign you in securely (including the 6-digit verification code) and reset your password.',
             'sw' => 'Kutengeneza akaunti yako, kukuingiza kwa usalama (ikijumuisha code ya 6 tarakimu) na kubadili neno lako la siri.'],
            ['en' => 'To prepare quotations, contracts and invoices, deliver the work and provide support.',
             'sw' => 'Kutayarisha quotation, mikataba na invoice, kutoa kazi na kutoa msaada.'],
            ['en' => 'To send service updates, security alerts and reminders about your project or payments.',
             'sw' => 'Kutuma masasisho ya huduma, tahadhari za usalama na vikumbusho kuhusu mradi wako au malipo.'],
            ['en' => 'To meet legal obligations under Tanzanian law, including BRELA and tax record-keeping.',
             'sw' => 'Kufuata wajibu wa kisheria chini ya sheria za Tanzania, ikiwemo kuhifadhi rekodi za BRELA na kodi.'],
        ]],
        ['hEn' => 'Who can see it', 'hSw' => 'Nani anaona', 'p' => [
            ['en' => 'Inside the company: only staff who need your data to do their job. Our admin portal is protected by password plus an emailed verification code.',
             'sw' => 'Ndani ya kampuni: ni wafanyakazi wanaohitaji data yako kufanya kazi yao. Portal yetu ya admin inalindwa na neno la siri pamoja na code ya uthibitisho ya barua pepe.'],
            ['en' => 'Outside the company: hosting and email providers that process data on our instructions (for example our database host and mail relay), each bound by their own confidentiality and security obligations. We do not sell, rent or trade personal data with anyone.',
             'sw' => 'Nje ya kampuni: watoa hosting na barua pepe wanaochakata data kwa maelekezo yetu (k.m. mwenyeji wetu wa database na relay ya barua pepe), kila mmoja akiwa na wajibu wake wa siri na usalama. Hatuuzi, kukodisha au kubadilisha data ya kibinafsi na mtu yeyote.'],
            ['en' => 'Legal disclosure: only when a competent Tanzanian authority requires it by law.',
             'sw' => 'Kufichua kisheria: tu wakati mamlaka ya Tanzania yenye mamlaka inahitaji kwa sheria.'],
        ]],
        ['hEn' => 'Cookies and local storage', 'hSw' => 'Cookies na local storage', 'p' => [
            ['en' => 'We use a strictly necessary session cookie to keep you signed in, plus local storage for your language choice and cart-like drafts. No third-party advertising or cross-site tracking cookies are used.',
             'sw' => 'Tunatumia session cookie muhimu tu ili kukushikilia umeingia, pamoja na local storage kwa lugha uliyochagua na ramani za rasimu. Hakuna cookies za matangazo za watu wa tatu wala ufuatiliaji wa matokeo.'],
        ]],
        ['hEn' => 'Security', 'hSw' => 'Usalama', 'ul' => [
            ['en' => 'Passwords are stored only as salted hashes — we cannot read them.',
             'sw' => 'Maneno ya siri huhifadhiwa kama hash zenye chumvi tu — hatuwezi kusoma.'],
            ['en' => 'Login requires a second step: a 6-digit code sent to your email and valid for 10 minutes.',
             'sw' => 'Kuingia kunahitaji hatua ya pili: code ya 6 tarakimu inayotumwa kwenye email yako na inadumu dakika 10.'],
            ['en' => 'All traffic runs over HTTPS/TLS. Contracts and invoices are stored server-side and visible only to you and the admin.',
             'sw' => 'Msimbo wote unaenda kupitia HTTPS/TLS. Mikataba na invoice huhifadishwa kwenye seva na huonekana na wewe na admin pekee.'],
            ['en' => 'Sensitive operations require a CSRF token; failed logins are rate-limited and logged.',
             'sw' => 'Kazi nyeti zinahitaji CSRF token; majaribio yaliyoshindwa yanapunguzwa na kurekodiwa.'],
        ]],
        ['hEn' => 'How long we keep it', 'hSw' => 'Tunaweka kwa muda gani', 'p' => [
            ['en' => 'Account and project records are kept while your account is active and for as long as Tanzanian tax and contract law requires us to keep them. Verification codes expire in 10 minutes and password-reset links after 1 hour. Visitor analytics are kept in aggregated form for trend reporting.',
             'sw' => 'Rekodi za akaunti na mradi hubaki wakati akaunti yako iko hai na kwa muda ambao sheria za kodi na mikataba ya Tanzania zinatuhitaji kuzihifadhi. Code za uthibitisho zinaisha ndani ya dakika 10 na link za kubadili neno la siri baada ya saa 1. Uchanganuzi wa wageni hufanyika kwa fomu ya muhtasari.'],
        ]],
        ['hEn' => 'Your rights', 'hSw' => 'Haki zako', 'ul' => [
            ['en' => 'Access and receive a copy of the personal data we hold about you.',
             'sw' => 'Kupata nakala ya data ya kibinafsi tunayokuwa nayo.'],
            ['en' => 'Correct anything that is wrong or out of date.',
             'sw' => 'Kurekebisha kilicho kisahihi au kilichopitwa na wakati.'],
            ['en' => 'Ask us to delete your account and personal data, where we have no overriding legal duty to keep it.',
             'sw' => 'Kutuomba kufuta akaunti yako na data yako ya kibinafsi, pale tusipo na wajibu mkubwa wa kisheria kuendelea kuifahadhi.'],
            ['en' => 'Object to a particular use and withdraw consent at any time.',
             'sw' => 'Kukataa matumizi maalum na kuondoa ruhusa wakati wowote.'],
            ['en' => 'Lodge a complaint with the Personal Data Protection Commission of Tanzania.',
             'sw' => 'Kulalamika kwa Tume ya Ulinzi wa Data ya Kibinafsi ya Tanzania.'],
        ]],
        ['hEn' => 'Children and other websites', 'hSw' => 'Watoto na tovuti nyingine', 'p' => [
            ['en' => 'Our services are aimed at businesses and adults; we do not knowingly collect data from children under 18. Links to other websites are governed by those sites\' own policies, not ours.',
             'sw' => 'Huduma zetu zinalenga biashaka na watu wazima; hatukusanyi data ya chini ya miaka 18 kwa makusudi. Viungo vya tovuti nyingine vinasimamiwa na sera za tovuti hizo wenyewe, si zetu.'],
        ]],
        ['hEn' => 'Contact us', 'hSw' => 'Wasiliana nasi', 'p' => [
            ['en' => 'Questions or requests about your data: info@mbilinyitech.co.tz, or call/WhatsApp 0796 752 645 (Mon–Sat, 08:00–20:00 EAT). Mbilinyi Tech Solutions, Dodoma, Tanzania. CEO: Jackson Mbilinyi.',
             'sw' => 'Maswali au maombi kuhusu data yako: info@mbilinyitech.co.tz, au piga/WhatsApp 0796 752 645 (Jumatatu–Jumamosi, 08:00–20:00 EAT). Mbilinyi Tech Solutions, Dodoma, Tanzania. CEO: Jackson Mbilinyi.'],
        ]],
    ],
]); ?>

<?php require __DIR__ . '/partials/footer.php'; ?>

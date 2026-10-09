<?php
require_once __DIR__ . '/config/app.php';
$pageTitle = 'Terms of Service — Masharti ya Huduma | Mbilinyi Tech Solutions';
$page = 'legal';
require __DIR__ . '/partials/head.php';
require __DIR__ . '/partials/legal-render.php';
?>
<body class="antialiased">
<?php require __DIR__ . '/partials/header.php'; ?>

<?php mts_legal_page([
    'badge'      => 'TERMS OF SERVICE',
    'badgeColor' => 'cyan',
    'titleEn'    => 'Terms of Service',
    'titleSw'    => 'Masharti ya Huduma',
    'introEn'    => 'These terms govern every engagement between Mbilinyi Tech Solutions and our clients. By requesting a service, receiving a quotation, signing a contract or paying an invoice you accept them in full. Software work follows the 50/50 payment schedule set out below; other services follow whatever both sides agree in writing.',
    'introSw'    => 'Masharti haya yanasimamia kila kati ya Mbilinyi Tech Solutions na wateja wetu. Kwa kuomba huduma, kupokea quotation, saini mkataba au kulipa invoice unayakubali kikamilifu. Kazi za software zinafuata ratiba ya malipo ya 50/50 iliyoelezwa hapa chini; huduma zingine zinafuata yale ambayo pande mbili zimekubaliana kimaandishi.',
    'updated'    => date('d F Y'),
    'sections'   => [
        ['hEn' => 'Who we are', 'hSw' => 'Sisi ni nani', 'p' => [
            ['en' => 'Mbilinyi Tech Solutions is a BRELA-registered limited company (Reg. Z-418822-77-TZ, TIN 192-147-522) owned by CEO Jackson Mbilinyi. We are based in Dodoma, Tanzania and deliver all work online / remotely, serving clients nationwide and worldwide.',
             'sw' => 'Mbilinyi Tech Solutions ni kampuni iliyosajiliwa na BRELA (Usajili Z-418822-77-TZ, TIN 192-147-522) inayomilikiwa na CEO Jackson Mbilinyi. Tuko Dodoma, Tanzania na tunatoa kazi zote mtandaoni / remote, tukiwahudumia wateja ndani ya nchi na duniani kote.'],
            ['en' => 'Contact: 0796 752 645 (Call / WhatsApp / SMS), info@mbilinyitech.co.tz, mbilinyitech@gmail.com. Hours: Monday–Saturday, 08:00–20:00 East Africa Time.',
             'sw' => 'Mawasiliano: 0796 752 645 (Simu / WhatsApp / SMS), info@mbilinyitech.co.tz, mbilinyitech@gmail.com. Saa za kazi: Jumatatu–Jumamosi, 08:00–20:00 saa za Afrika Mashariki.'],
        ]],
        ['hEn' => 'Services covered', 'hSw' => 'Huduma zilizojumuishwa', 'ul' => [
            ['en' => 'Custom software and web development, mobile apps and system analysis.',
             'sw' => 'Maendeleo ya programu tofauti na tovuti, programu za simu na uchambuzi wa mifumo.'],
            ['en' => 'Software maintenance and SLA, hosting, backups and uptime monitoring.',
             'sw' => 'Matengenezo ya programu na SLA, hosting, backup na ufuatiliaji wa muda wa kazi.'],
            ['en' => 'Cybersecurity audits, penetration testing and staff training.',
             'sw' => 'Uchunguzi wa usalama, mtihani wa uvamizi na mafunzo ya wafanyakazi.'],
            ['en' => 'AI fine-tuning, chatbots and integration work.',
             'sw' => 'Kurekebisha AI, chatbots na kazi za kuunganisha mifumo.'],
            ['en' => 'Consultancy, training and support on an agreed scope.',
             'sw' => 'Ushauri, mafunzo na msaada kwa mradi uliokubaliwa.'],
        ]],
        ['hEn' => 'Quotations and contracts', 'hSw' => 'Quotation na mikataba', 'p' => [
            ['en' => 'Every quotation is valid for 14 days from issue unless stated otherwise and is based on the requirements you provide. Anything outside that scope is quoted separately.',
             'sw' => 'Kila quotation ni ya siku 14 toka ikitolewa isiposema vinginevyo, na imejengwa juu ya mahitaji uliyotoa. Kile kilicho nje ya hiyo scope kinapewa quotation tofauti.'],
            ['en' => 'For software work a written contract is MANDATORY before any work begins. The contract is signed electronically by both parties, stored permanently in our system, emailed to both of you, and cannot be deleted by either side.',
             'sw' => 'Kwa kazi za software, mkataba wa maandishi ni LAZIMA kabla ya kazi kuanza. Mkataba unasainiwa kielektroniki na pande zote mbili, unahifadhiwa milele kwenye mfumo wetu, unatumwa kwa barua pepe kwenu nyote hawili, na hauwezi kufutwa na pande yoyote.'],
            ['en' => 'Work on services other than software follows the terms both parties agree in writing before the work starts.',
             'sw' => 'Kazi nje ya software zinafuata masharti ambayo pande mbili zimekubaliana kimaandishi kabla ya kazi kuanza.'],
        ]],
        ['hEn' => 'Payment terms — 50 / 50', 'hSw' => 'Masharti ya malipo — 50 / 50', 'p' => [
            ['en' => 'This 50/50 schedule applies to SOFTWARE work. Other services are handled on whatever payment terms both parties agree in writing.',
             'sw' => 'Ratiba hii ya 50/50 inatumika kwa kazi za SOFTWARE. Huduma zingine zinashughulikiwa kwa masharti ya malipo yaliyokubaliwa kimaandishi na pande mbili.'],
        ], 'ul' => [
            ['en' => '50% DEPOSIT — payable BEFORE the project starts. The project does not start until the deposit is received and verified by the CEO.',
             'sw' => '50% Amana — hulipwa KABLA mradi haujaanza. Mradi hauanzi hadi amana imepokelewa na CEO ameithibitisha.'],
            ['en' => '50% MID-PROJECT — payable halfway through the build, before final delivery and handover.',
             'sw' => '50% KATIKATI YA MRADI — hulipwa katikati ya ujenzi, kabla ya mwisho na kukabidhiwa.'],
            ['en' => 'Payments: NMB Bank account and Vodacom M-Pesa number shown in your portal after you sign in, plus any invoice we issue.',
             'sw' => 'Malipo: Akaunti ya NMB Bank na namba ya Vodacom M-Pesa zinazoonyeshwa kwenye portal yako baada ya kuingia, pamoja na kila invoice tunayotoa.'],
            ['en' => 'Invoices are due within 3 working days of issue. Late payment may pause the project; agreed timelines extend accordingly.',
             'sw' => 'Invoice zinapaswa kulipwa ndani ya siku 3 za kazi. Malipo ya kuchelewa yanaweza kusimamisha mradi; muda uliokubaliwa unaongezeka ipasavyo.'],
        ]],
        ['hEn' => 'Your responsibilities', 'hSw' => 'Wajibu wako', 'ul' => [
            ['en' => 'Send complete, accurate requirements, content, logos, images and access credentials on time.',
             'sw' => 'Tuma mahitaji kamili na sahihi, maudhui, nembo, picha na nenosiri kwa wakati.'],
            ['en' => 'Review each milestone and reply with feedback within 5 working days.',
             'sw' => 'Pitia kila hatua na jibu na maoni ndani ya siku 5 za kazi.'],
            ['en' => 'Keep your account password and verification codes confidential.',
             'sw' => 'Weka siri neno lako la siri na code za uthibitisho.'],
            ['en' => 'Do not use our deliverables for unlawful, deceptive or infringing purposes.',
             'sw' => 'Usitumie kile tulichokutoa kwa matumizi ya uhalifu, udanganyifu au uvamizi wa haki.'],
        ]],
        ['hEn' => 'Delivery, revisions and intellectual property', 'hSw' => 'Uwasilishaji, marekebisho na haki miliki', 'p' => [
            ['en' => 'Delivery dates depend on your timely input; delays caused by you extend the timeline fairly and in writing. Each package includes the revision rounds stated in your quotation; extra work is quoted separately.',
             'sw' => 'Tarehe za uwasilishaji zinategemea ulipotuma vitu kwa wakati; kuchelewa kutoka kwako kunaongeza muda kwa haki na kwa maandishi. Kila kifurushi kina viwango vya marekebisho vilivyomo kwenye quotation yako; kazi ya ziada inapewa quotation tofauti.'],
            ['en' => 'On final payment, full ownership of the delivered work transfers to you. We keep the right to showcase the project in our portfolio and to reuse non-proprietary components and general know-how.',
             'sw' => 'Ukimaliza kulipa, umiliki kamili wa kazi iliyowasilishwa hubadilika kwako. Tunahaki ya kuonyesha mradi kwenye portfolio yetu na kutumia tena vipengele visivya umiliki na ujua wa jumla.'],
            ['en' => 'Third-party licences, fonts, images, plugins and hosting remain subject to their own terms.',
             'sw' => 'Leseni za watu wa tatu, fonti, picha, plugins na hosting zinabaki chini ya masharti yao wenyewe.'],
        ]],
        ['hEn' => 'Confidentiality and data', 'hSw' => 'Siri na data', 'p' => [
            ['en' => 'Both parties keep each other\'s confidential information secret for the engagement and afterwards, except where the law requires disclosure. See the Privacy Policy for how personal data is handled.',
             'sw' => 'Pande mbili huhifadhi taarifa za siri za pande nyingine wakati wa ushirikiano na baada yake, isipokuwa sheria inapohitaji kufichua. Angalia Sera ya Faragha jinsi data ya kibinafsi inavyoshughulikiwa.'],
        ]],
        ['hEn' => 'Warranties and support', 'hSw' => 'Dhamana na msaada', 'p' => [
            ['en' => 'We warrant that our work will materially match the agreed specification. If you find a defect within 30 days of delivery we fix it free of charge. After that, maintenance plans and SLAs apply.',
             'sw' => 'Tunahakikisha kazi yetu inafuata muundo uliokubaliwa. Ukiona hitilafu ndani ya siku 30 baada ya uwasilishaji, tunairekebisha bila malipo. Baada ya hayo, mipango ya matengenezo na SLA inatumika.'],
            ['en' => 'We do not guarantee uninterrupted service caused by third parties, hosting providers, internet outages or force majeure.',
             'sw' => 'Hatushirikishi dhamana ya huduma isiyo na kikomo kutokana na watu wa tato, watoa hosting, kukatika kwa mtandao au force majeure.'],
        ]],
        ['hEn' => 'Termination', 'hSw' => 'Kuahirisha mkataba', 'p' => [
            ['en' => 'Either party may terminate on 14 days\' written notice. Milestones already completed remain payable and are not refundable. The deposit is non-refundable once work has begun.',
             'sw' => 'Pande yoyote inaweza kuahirisha mkataba kwa taarifa ya siku 14 za maandishi. Hatua zilizokamilika zinabaki zenye kulipwa na hazirudishwi. Amana hairudishwi mara kazi imeanza.'],
        ]],
        ['hEn' => 'Liability', 'hSw' => 'Wajibu wa kisheria', 'p' => [
            ['en' => 'Our total liability under any contract is limited to the amount actually paid for that contract. We are not liable for indirect or consequential loss, loss of profit, data or goodwill.',
             'sw' => 'Wajibu wetu wote kwa mkataba wowote umepunguzwa hadi kiasi kilicholipwa kwa mkataba husika. Hatuliphi hasara zisizo za moja kwa moja, hasara ya faida, data au sifa.'],
        ]],
        ['hEn' => 'Governing law and changes', 'hSw' => 'Sheria inayosimamia na mabadiliko', 'p' => [
            ['en' => 'These terms are governed by the laws of the United Republic of Tanzania. Disputes are first addressed through mediation in Dodoma before any court action.',
             'sw' => 'Masharti haya yanasimamiwa na sheria za Jamhuri ya Muungano wa Tanzania. Migogoro kwanza inashughulikiwa kwa mpatanisho Dodoma kabla ya kesi mahakamani.'],
            ['en' => 'We may update these terms; the current version is always published here with its date. Continued use of the site after a change means you accept the new version.',
             'sw' => 'Tunaweza kusasisha masharti haya; toleo la sasa daima linachapishwa hapa pamoja na tarehe yake. Kendelea kutumia tovuti baada ya mabadiliko inamaanisha umeyakubali mapya.'],
        ]],
    ],
]); ?>

<?php require __DIR__ . '/partials/footer.php'; ?>

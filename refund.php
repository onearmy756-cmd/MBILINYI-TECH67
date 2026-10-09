<?php
require_once __DIR__ . '/config/app.php';
$pageTitle = 'Refund Policy — Sera ya Kurudisha Pesa | Mbilinyi Tech Solutions';
$page = 'legal';
require __DIR__ . '/partials/head.php';
require __DIR__ . '/partials/legal-render.php';
?>
<body class="antialiased">
<?php require __DIR__ . '/partials/header.php'; ?>

<?php mts_legal_page([
    'badge'      => 'REFUND & PAYMENT TERMS',
    'badgeColor' => 'rose',
    'titleEn'    => 'Refund Policy',
    'titleSw'    => 'Sera ya Kurudisha Pesa',
    'introEn'    => 'We keep our refund rules simple and honest. Software projects run on a 50% deposit before start and 50% mid-project, and the deposit becomes non-refundable as soon as work begins because we reserve engineering time for you. Services that are not software are handled on whatever terms both parties agree in writing first.',
    'introSw'    => 'Tunahifadhi sheria zetu za kurudisha pesa kuwa rahisi na za kweli. Miradi ya software inaenda kwa 50% amana kabla ya kuanza na 50% katikati ya mradi, na amana inakuwa isiyorudishwa mara tu kazi imeanza kwa sababu tunaweka muda wa wahandisi kwa ajili yako. Huduma ambazo si software zinashughulikiwa kwa masharti yaliyokubaliwa kimaandishi kwanza.',
    'updated'    => date('d F Y'),
    'sections'   => [
        ['hEn' => 'The short version', 'hSw' => 'Toleo fupi', 'ul' => [
            ['en' => 'Deposit paid but we have not started work yet → full refund.',
             'sw' => 'Amana imelipwa na bado hatujaanza kazi → marejesho kamili.'],
            ['en' => 'Work has started → the 50% deposit is non-refundable; we deliver what was agreed.',
             'sw' => 'Kazi imeanza → amana ya 50% hairudishwi; tunakile kilichokubaliwa.'],
            ['en' => 'We cancel or miss an agreed deadline without cause → refund of everything paid for undelivered work.',
             'sw' => 'Sisi tunaahirisha au kukosa tarehe iliyokubaliwa bila sababu → marejesho ya kila kilicholipwa kwa kazi haijawasilishwa.'],
            ['en' => 'Hosting, domain, SMS, third-party licences and other pass-through costs are never refundable once purchased.',
             'sw' => 'Hosting, domain, SMS, leseni za watu wa tatu na gharama zingine hazirudishwi mara zimenunuliwa.'],
        ]],
        ['hEn' => 'How the 50/50 schedule works', 'hSw' => 'Ratiba ya 50/50 inafanyaje kazi', 'p' => [
            ['en' => 'For SOFTWARE work: 50% deposit is paid before the project starts, and the remaining 50% is paid MID-PROJECT — before final delivery and handover. Both payments follow the schedule written into your signed contract.',
             'sw' => 'Kwa kazi za SOFTWARE: 50% ya amana hulipwa kabla mradi haujaanza, na 50% iliyobaki hulipwa KATIKATI YA MRADI — kabla ya uwasilishaji wa mwisho. Malipo yote yanafuata ratiba iliyoandikwa kwenye mkataba wako uliosainiwa.'],
            ['en' => 'For services other than software — consulting, training, one-off design, security reviews on request — payment terms are what we both agree in writing before the work begins. Nothing is charged beyond that agreement.',
             'sw' => 'Kwa huduma nje ya software — ushauri, mafunzo, muundo wa mara moja, uchunguzi wa usalama kwa ombi — masharti ya malipo ni yale tuliyokubaliana kimaandishi kabla ya kazi kuanza. Hakuna kinacholipwa zaidi ya makubaliano hayo.'],
        ]],
        ['hEn' => 'Refund eligibility', 'hSw' => 'Masharti ya kustahili marejesho', 'p' => [
            ['en' => 'Refund requests must be made in writing to info@mbilinyitech.co.tz or through your client portal within 7 days of the payment date.',
             'sw' => 'Maombi ya kurudisha pesa yanapaswa kufanywa kwa maandishi kwa info@mbilinyitech.co.tz au kupitia portal yako ya mteja ndani ya siku 7 kutoka tarehe ya malipo.'],
            ['en' => 'Approved refunds are processed within 14 working days back to the original payment method (bank account or mobile money number used).',
             'sw' => 'Marejesho yaliyoidhinishwa hufanyika ndani ya siku 14 za kazi kurudi kwenye njia yake ya asili (akaunti ya benki au namba ya malipo ya simu zilizotumika).'],
        ]],
        ['hEn' => 'Situation by situation', 'hSw' => 'Hali kwa hali', 'ul' => [
            ['en' => 'Before we start: cancel any time before kickoff and receive 100% of what you paid.',
             'sw' => 'Kabla hatujaanza: ahirisha wakati wowote kabla ya kuanza na upate 100% ya kilicholipwa.'],
            ['en' => 'During development: the deposit covers the reserved engineering time and is retained; we still deliver every agreed milestone.',
             'sw' => 'Wakati wa maendeleo: amana inafunika muda wa wahandisi uliowekwa na inabaki nayo; bado tunakile kila kile kilichokubaliwa.'],
            ['en' => 'Fault on our side: if we deliver materially wrong work and do not correct it after being asked, you may claim a refund for the undelivered portion.',
             'sw' => 'Hitilafu yetu: tukitoa kazi isiyo sahihi kimsingi na kushindwa kuirekebisha baada ya kuombwa, unaweza kudai marejesho ya sehemu haijawasilishwa.'],
            ['en' => 'Change of mind: once development has started, changes of mind are not refundable — talk to us and we will re-scope instead.',
             'sw' => 'Badiliko la nia: mara maendeleo yameanza, kubadilisha mawazo hakirudishwi — ongea nasi na tutabadilisha scope badala yake.'],
            ['en' => 'Force majeure: events outside reasonable control are handled case by case in good faith.',
             'sw' => 'Force majeure: matukio nje ya udhibiti wetu yanashughulikiwa kwa kila kesi kwa nia njema.'],
        ]],
        ['hEn' => 'Maintenance and subscriptions', 'hSw' => 'Matengenezo na usajili', 'p' => [
            ['en' => 'Monthly maintenance and security plans renew automatically for the period you subscribed to. Cancel any time before the next renewal date and the plan simply stops — no further charge.',
             'sw' => 'Mipango ya kila mwezi ya matengenezo na usalama inaongezwa kiotomatiki kwa kipindi ulichojisajili. Ahirisha wakati wowote kabla ya tarehe ya kujazwa upya inayofuata na mpango unaisha — hakuna malipo zaidi.'],
            ['en' => 'A month already paid for and started is not refundable, but the service continues until the end of that period.',
             'sw' => 'Mwezi tayari uliolipwa na ulioanza haurudishwi, lakini huduma inaendelea hadi mwisho wa kipindi hicho.'],
        ]],
        ['hEn' => 'How to request a refund', 'hSw' => 'Jinsi ya kuomba marejesho', 'ul' => [
            ['en' => 'Email info@mbilinyitech.co.tz or open a ticket in your portal with your tracking ID and payment proof.',
             'sw' => 'Tuma barua pepe kwa info@mbilinyitech.co.tz au fungua tiketi kwenye portal yako na tracking ID na uthibitisho wa malipo.'],
            ['en' => 'We acknowledge within 1 business day and give a decision within 5 working days.',
             'sw' => 'Tunathibitisha kupokea ndani ya siku 1 ya kazi na kutoa uamuzi ndani ya siku 5 za kazi.'],
            ['en' => 'Approved amounts are sent back to the original payment method only.',
             'sw' => 'Kiasi kilichoidhinishwa hutumwa kwenye njia yake ya asili pekee.'],
            ['en' => 'Unresolved? Escalate directly to CEO Jackson Mbilinyi on 0796 752 645.',
             'sw' => 'Haujawekwa sawa? Piga kwa CEO Jackson Mbilinyi, 0796 752 645.'],
        ]],
    ],
]); ?>

<?php require __DIR__ . '/partials/footer.php'; ?>

<?php
declare(strict_types=1);
require_once __DIR__ . '/../config/app.php';

header('Content-Type: application/json; charset=utf-8');

try {
    switch (action_name()) {

        case 'config': {
            $c = ollama_config();
            j_ok([
                'enabled' => $c['enabled'],
                'model' => $c['model'],
            ]);
        }

        case 'chat': {
            /* The API key is a paid resource — never let another origin drive it
             * through a visitor's browser (TR-7.1). */
            require_csrf();
            $cfg = ollama_config();
            $messages = param('messages');
            $prompt = str_param('prompt', '');
            $history = is_array($messages) ? $messages : [];
            if ($prompt !== '' && count($history) === 0) {
                $history[] = ['role' => 'user', 'content' => $prompt];
            }
            if (count($history) === 0) j_err('Empty prompt.', 400);

            // System prompt tailored to Mbilinyi Tech — ALWAYS injected as first
            $servicesList = '';
            $svcs = [
                ['Custom Software Development', 'Web, mobile & desktop systems in React, Python, Rust, PHP, Laravel, R & HTML/CSS/JS.'],
                ['System Analysis & Design', 'SRS, database design, UML, workflows, feasibility studies.'],
                ['IT Consultancy', 'Digital strategy, system audits, CTO-as-a-service.'],
                ['Software Maintenance', 'Bug fixes, updates, backups, SLA 99.9% & 24/7 support.'],
                ['Cybersecurity Services', 'Pentest, OWASP audits, malware removal, SSL, firewall, staff training.'],
                ['AI Chatbots', 'Website & WhatsApp chatbots in English + Kiswahili.'],
                ['AI Fine-Tuning', 'LLM fine-tuning on YOUR data (LoRA, RAG) for on-brand accurate answers.'],
                ['Cloud & DevOps', 'AWS/VPS, CI/CD, Docker, domains, business emails.'],
                ['BRELA & Business Registration', 'Usajili wa Biashara, SME, Company (BRELA Z-418822-77-TZ).'],
                ['TRA & Tax Services', 'TIN, Tax returns, VAT, PAYE (TIN 192-147-522).'],
                ['Computer Troubleshooting', 'Software issues, malware removal, slow PC, crashes, data recovery.'],
                ['Windows & OS Installation', 'Windows 10/11, Linux, macOS + drivers + activation.'],
                ['Email & Communication Setup', 'G-Suite, Zoho, cPanel, SMTP, SPF/DKIM, email troubleshooting.'],
                ['Government e-Services', 'NIDA, NHIF, NSSF, Huduma Number, eCitizen & other gov portals.'],
                ['Software Installation', 'MS Office, Adobe, IDEs, Antivirus, design apps & custom setup.'],
            ];
            foreach ($svcs as $i => $s) { $servicesList .= ($i+1) . '. ' . $s[0] . ' — ' . $s[1] . "\n"; }
            $stackList = "PHP (96%), Laravel (94%), Python (98%), React (97%), Rust (90%), R (88%), HTML/CSS/JS (99%), Node.js (92%), Flutter (85%), SQL & Cloud (94%).";

            /* --- live price list (admin edits these in Admin → Settings) --- */
            $priceList = '';
            foreach (service_price_list() as $i => $p) {
                if ($p['price'] <= 0) continue;
                $priceList .= ($i + 1) . '. ' . $p['en'] . ' — ' . tzs($p['price']) . ' ' . $p['period']
                    . '  (Kiswahili: ' . $p['sw'] . ")\n";
            }
            $plansList = '';
            try {
                foreach (db_all('SELECT name, price, period, group_name FROM pricing_plans WHERE is_active = 1 ORDER BY group_name, sort_order') as $pl) {
                    $g = $pl['group_name'] === 'sec' ? 'Security service' : 'Maintenance plan';
                    $plansList .= '- ' . $pl['name'] . ' (' . $g . ') — ' . tzs((float)$pl['price']) . ' ' . $pl['period'] . "\n";
                }
            } catch (\Throwable $e) { /* pricing table may be empty */ }
            if ($priceList === '' && $plansList === '') {
                $priceList = "No published prices right now — always invite the user to submit the Request Service form for an official quote.\n";
            }

            $sysPrompt = <<<PROMPT
You are the friendly official AI assistant of MBILINYI TECH SOLUTIONS, BRELA registered (Z-418822-77-TZ), TIN 192-147-522.
Company location: DODOMA, TANZANIA. All work is delivered ONLINE / REMOTELY to clients anywhere in Tanzania and worldwide — never say Dar es Salaam.
Website: https://mbilinyitech.co.tz
CEO & Sole Administrator: JACKSON MBILINYI — phone/WhatsApp/SMS +255 796 752 645, email mbilinyitech@gmail.com, Instagram @mbilinyitech.
Company TAGLINE: "YOUR PROBLEM, OUR SOLUTION" — Kiswahili: "SHIDA YAKO, TATUZO LETU".
Working hours: Mon-Sat 08:00 — 20:00 EAT.

## PRICES (quote these in Tanzanian Shillings — they are the current published rates):
{$priceList}

### Maintenance & security plans (monthly / one-off):
{$plansList}
Prices are subject to change and the CEO confirms every final figure in an official quotation. Always remind the user that a formal quote follows within 24 hours.

## STRICT NON-NEGOTIABLE PAYMENT TERMS:
FOR SOFTWARE PROJECTS ONLY — TWO installments:
  ① 50% DEPOSIT — payable BEFORE project START, and VERIFIED by CEO Jackson Mbilinyi. Project DOES NOT START until this is paid. Deposit is non-refundable once work begins.
  ② 50% MID-PROJECT INSTALLMENT — payable when the timeline has reached 50% completion (NOT at final delivery). Source code, training, acceptance and handover only happen AFTER this is paid.
ALL OTHER SERVICES (consulting, training, BRELA/TRA filings, one-off audits) are NOT bound by 50/50 — payment terms for those are whatever both parties agree in writing first. Never apply the 50/50 rule to non-software work.
Penalty for late payments: 2% per week. Quotations are valid for the stated number of days on the quote.
Payment numbers (NMB bank account and Vodacom M-Pesa) are PRIVATE — they are shown only to signed-in clients in their portal and on invoices. Never print bank or mobile numbers in chat; instead say: "Sign in to your portal or wait for your invoice and you will see the payment details."

## SERVICES (15 total, offer only these + ask for clarifications):
{$servicesList}

## TECH STACK (languages/frameworks we use):
{$stackList}

## PAGES ON THE WEBSITE (link to them when relevant):
Terms of Service → https://mbilinyitech.co.tz/terms.php
Refund Policy   → https://mbilinyitech.co.tz/refund.php
Privacy Policy  → https://mbilinyitech.co.tz/privacy.php
Feedback        → https://mbilinyitech.co.tz/feedback.php
Contact/Support → https://mbilinyitech.co.tz/contact.php

## HOW TO BEHAVE:
1) BILINGUAL: ALWAYS reply in the SAME LANGUAGE the user wrote (English or Kiswahili). Mixed is OK when helpful. Never invent other languages.
2) Use the PRICES above ACCURATELY when asked. If the user wants an exact figure, tell them to submit the Request Service form (Quote Wizard) and CEO Jackson will reply with an official quotation within 24 hours.
3) If asked about 50/50 payment: CLARIFY it applies to SOFTWARE work and means DEPOSIT (before start) + MID-PROJECT (halfway, not at delivery). For non-software services say terms are agreed case by case.
4) If asked about refunds, privacy or terms — answer briefly from the policies and link to the relevant page above.
5) If a question is technical or beyond the info above — answer what you can honestly, then say "Jackson Mbilinyi (CEO, +255 796 752 645 / mbilinyitech@gmail.com) will confirm all details."
6) Be warm, professional, concise, and always end long replies with a short CTA to contact CEO or use the Quote Wizard.
7) NEVER offer other companies' services; NEVER offer to do work for free; NEVER provide financial, legal or medical advice outside registered scope; NEVER invent prices, staff, or delivery dates.
PROMPT;

            if (!$cfg['enabled']) {
                j_ok([
                    'reply' => "[Ollama disabled — fallback].\n\nAsante kwa kutupigia. Jackson Mbilinyi (CEO, +255 796 752 645 / mbilinyitech@gmail.com) atajibu kwa kiholela ndani ya saa 24. Unaweza kutumia kitufe cha \"Request Service\" kwenye ukurasa wa kwanza kwa ajili ya nukuu rasmi. Kumbuka: huduma zetu zina malipo ya 50% kabla ya kuanza na 50% kati ya mradi.",
                    'enabled' => false,
                ]);
            }

            $apiUrl = $cfg['base_url'] . '/chat/completions';
            $apiKey = $cfg['api_key'];

            /* First configured model, then every candidate — a model that is
             * renamed, unavailable or rate-limited simply falls through. */
            $order = [];
            if ($cfg['model'] !== '') $order[] = $cfg['model'];
            foreach (($cfg['candidates'] ?? []) as $m) {
                if (!in_array($m, $order, true)) $order[] = $m;
            }
            if (!$order) $order[] = 'glm-5.3-flash';

            $payload = [
                'stream' => false,
                'messages' => array_merge(
                    [['role' => 'system', 'content' => $sysPrompt]],
                    array_slice(array_map(function ($m) {
                        return [
                            'role' => in_array($m['role'] ?? '', ['user','assistant','system']) ? $m['role'] : 'user',
                            'content' => (string)($m['content'] ?? ''),
                        ];
                    }, $history), -18)
                ),
                'temperature' => 0.35,
                'max_tokens' => 700,
            ];

            $raw = false; $status = 0; $err = ''; $model = $order[0];
            foreach ($order as $candidate) {
                $model = $candidate;
                $payload['model'] = $candidate;

                $ch = curl_init($apiUrl);
                curl_setopt_array($ch, [
                    CURLOPT_RETURNTRANSFER => true,
                    CURLOPT_TIMEOUT => 75,
                    CURLOPT_CONNECTTIMEOUT => 15,
                    CURLOPT_POST => true,
                    CURLOPT_POSTFIELDS => json_encode($payload),
                    CURLOPT_HTTPHEADER => [
                        'Content-Type: application/json',
                        'Authorization: Bearer ' . $apiKey,
                    ],
                ]);
                $raw = curl_exec($ch);
                $status = curl_getinfo($ch, CURLINFO_HTTP_CODE);
                $err = curl_error($ch);
                curl_close($ch);

                if ($raw !== false && $err === '' && $status < 400) break;
                error_log("[ollama] model {$candidate} unusable (HTTP {$status} {$err}) — trying the next one");
                $raw = false;
            }

            if ($raw === false || $err !== '') {
                error_log('[ollama] all models failed, last error: ' . $err);
                j_ok([
                    'reply' => "[Ollama network issue — fallback].\n\nKwa hivyo mawasiliano na AI kwa sasa yana tatizo. Tafadhali wasiliana na CEO Jackson Mbilinyi kwa moja: +255 796 752 645 (WhatsApp/SMS) au mbilinyitech@gmail.com. Nitajaribu kutoa huduma vizuri baada ya mawasiliano. Asante!",
                    'enabled' => true,
                    'error' => 'network',
                ]);
            }

            $dec = json_decode($raw, true);
            if (!is_array($dec)) {
                error_log('[ollama] bad json: ' . substr((string)$raw, 0, 500));
                j_err('AI service returned invalid response. Try again.', 502);
            }
            if ($status >= 400) {
                error_log('[ollama] http ' . $status . ' body: ' . substr((string)$raw, 0, 1000));
                j_err($dec['error']['message'] ?? 'AI service error.', 502);
            }

            $content = $dec['choices'][0]['message']['content'] ?? '[No response]';

            j_ok([
                'reply' => is_string($content) ? $content : json_encode($content),
                'enabled' => true,
                'model' => $model,
            ]);
        }

        default:
            j_err('Unknown ollama action.', 404);
    }
} catch (Throwable $e) {
    error_log('[api/ollama] ' . $e->getMessage());
    j_err('Server error. Please try again.', 500);
}

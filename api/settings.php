<?php
declare(strict_types=1);
require_once __DIR__ . '/../config/app.php';

header('Content-Type: application/json; charset=utf-8');

/**
 * Settings + pricing API.
 *
 *  list          signed-in user  → every setting (clients get the payment block)
 *  save          admin           → upsert one key
 *  save_all      admin           → upsert several keys at once
 *  plans         anyone          → active price list
 *  plan_save     admin           → upsert one price
 *  feedback      anyone          → leave feedback
 *  feedback_list admin           → read feedback
 */
try {
    switch (action_name()) {

        case 'list': {
            require_user();
            j_ok([
                'settings'      => settings_all(),
                'payment'       => payment_details(),
                'servicePrices' => service_price_list(),
            ]);
        }

        case 'save': {
            require_admin();
            require_csrf();
            $key = str_param('key');
            if ($key === '' || !preg_match('/^[a-z0-9_]{1,80}$/i', $key)) j_err('Invalid setting key.');
            $value = (string)param('value', '');
            setting_set($key, $value);
            j_ok(['saved' => ['key' => $key, 'value' => $value]]);
        }

        case 'save_all': {
            require_admin();
            require_csrf();
            $rows = param('settings');
            if (!is_array($rows) || !$rows) j_err('No settings supplied.');
            $saved = [];
            foreach ($rows as $k => $v) {
                $k = (string)$k;
                if (!preg_match('/^[a-z0-9_]{1,80}$/i', $k)) continue;
                setting_set($k, is_scalar($v) ? (string)$v : '');
                $saved[$k] = is_scalar($v) ? (string)$v : '';
            }
            j_ok(['saved' => $saved, 'count' => count($saved)]);
        }

        /* ---------------- pricing ---------------- */

        case 'plans': {
            $rows = db_all('SELECT * FROM pricing_plans WHERE is_active = 1 ORDER BY group_name, sort_order');
            $plans = array_map(static function (array $p): array {
                $feat = json_decode($p['features'] ?? '[]', true);
                return [
                    'id'    => $p['id'],
                    'code'  => $p['code'],
                    'group' => $p['group_name'],
                    'name'  => $p['name'],
                    'price' => (float)$p['price'],
                    'currency' => $p['currency'],
                    'period'   => $p['period'],
                    'features' => is_array($feat) ? $feat : [],
                    'sort'  => (int)$p['sort_order'],
                ];
            }, $rows);
            j_ok(['plans' => $plans]);
        }

        case 'plan_save': {
            require_admin();
            require_csrf();
            $id = str_param('id');
            $name = str_param('name');
            $price = (float)param('price', 0);
            $group = str_param('group', '');
            $period = str_param('period');
            $features = param('features');
            /* group is optional — when it is not sent (partial edit from the
             * price table) the plan must stay in the group it already has. */
            $groupGiven = in_array($group, ['maint', 'sec'], true);
            if ($name === '') j_err('Plan name is required.');
            $featJson = json_encode(is_array($features) ? array_values($features) : [], JSON_UNESCAPED_UNICODE);

            if ($id === '' || $id === 'new') {
                $id = new_id('P-');
                if (!$groupGiven) $group = 'maint';
                $code = strtolower(str_replace(' ', '-', preg_replace('/[^A-Za-z0-9 ]/', '', $name)));
                db_run(
                    'INSERT INTO pricing_plans (id, code, group_name, name, price, currency, period, features, sort_order)
                     VALUES (?,?,?,?,?,?,?,?,0)',
                    [$id, substr($code, 0, 60) ?: $id, $group, $name, $price, setting('pay_currency', 'TZS'), $period, $featJson]
                );
            } else {
                if ($groupGiven) {
                    db_run(
                        'UPDATE pricing_plans SET name = ?, price = ?, group_name = ?, period = ?, features = ? WHERE id = ?',
                        [$name, $price, $group, $period, $featJson, $id]
                    );
                } else {
                    db_run(
                        'UPDATE pricing_plans SET name = ?, price = ?, period = ?, features = ? WHERE id = ?',
                        [$name, $price, $period, $featJson, $id]
                    );
                }
            }
            j_ok(['id' => $id]);
        }

        case 'plan_toggle': {
            require_admin();
            require_csrf();
            $id = str_param('id');
            db_run('UPDATE pricing_plans SET is_active = IF(is_active = 1, 0, 1) WHERE id = ?', [$id]);
            $row = db_one('SELECT is_active FROM pricing_plans WHERE id = ?', [$id]);
            j_ok(['id' => $id, 'isActive' => (bool)($row['is_active'] ?? 1)]);
        }

        /* ---------------- feedback ---------------- */

        case 'feedback': {
            require_csrf();
            $name = str_param('name');
            $email = str_param('email');
            $message = str_param('message');
            if ($message === '') j_err('Andika ujumbe wako / Please write your message.');
            if ($email !== '' && !filter_var($email, FILTER_VALIDATE_EMAIL)) j_err('Email si sahihi / Invalid email.');
            $id = new_id('FB-');
            db_run(
                'INSERT INTO feedback (id, name, email, category, subject, message, rating, status, created_at)
                 VALUES (?,?,?,?,?,?,?,\'New\', NOW())',
                [
                    $id,
                    mb_substr($name, 0, 160),
                    mb_substr($email, 0, 190),
                    mb_substr(str_param('category', 'general'), 0, 60),
                    mb_substr(str_param('subject'), 0, 255),
                    $message,
                    max(0, min(5, (int)param('rating', 0))),
                ]
            );
            j_ok(['id' => $id, 'thanks' => 'Asante kwa maoni yako! / Thanks for your feedback!']);
        }

        case 'feedback_list': {
            require_admin();
            $rows = db_all('SELECT * FROM feedback ORDER BY created_at DESC LIMIT 200');
            j_ok(['feedback' => array_map(static fn(array $r): array => [
                'id' => $r['id'], 'name' => $r['name'], 'email' => $r['email'],
                'category' => $r['category'], 'subject' => $r['subject'], 'message' => $r['message'],
                'rating' => (int)$r['rating'], 'status' => $r['status'], 'createdAt' => iso($r['created_at'] ?? null),
            ], $rows)]);
        }

        case 'feedback_status': {
            require_admin();
            require_csrf();
            $id = str_param('id');
            $status = str_param('status', 'New');
            if (!in_array($status, ['New', 'Read', 'Replied', 'Archived'], true)) j_err('Bad status.');
            db_run('UPDATE feedback SET status = ? WHERE id = ?', [$status, $id]);
            j_ok(['id' => $id, 'status' => $status]);
        }

        default:
            j_err('Unknown settings action.', 404);
    }
} catch (\Throwable $e) {
    error_log('[api/settings] ' . $e->getMessage());
    j_err('Server error. Please try again.', 500);
}

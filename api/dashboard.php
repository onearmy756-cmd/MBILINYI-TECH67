<?php
declare(strict_types=1);
require_once __DIR__ . '/../config/app.php';

header('Content-Type: application/json; charset=utf-8');

try {
    switch (action_name()) {

        case 'stats': {
            require_admin();

            $paid = (float)(db_one("SELECT coalesce(sum(amount),0) AS s FROM invoices WHERE status = 'Paid'")['s'] ?? 0);
            $unpaid = (float)(db_one("SELECT coalesce(sum(amount),0) AS s FROM invoices WHERE status <> 'Paid'")['s'] ?? 0);
            $pipeline = (float)(db_one("SELECT coalesce(sum(total),0) AS s FROM quotes WHERE status <> 'Rejected'")['s'] ?? 0);
            $pending = (int)(db_one(
                "SELECT count(*) AS c FROM requests WHERE status IN ('Pending','Under Review')"
            )['c'] ?? 0);
            $requestCount = (int)(db_one('SELECT count(*) AS c FROM requests')['c'] ?? 0);
            $userCount = (int)(db_one('SELECT count(*) AS c FROM users')['c'] ?? 0);

            $byService = [];
            foreach (db_all('SELECT service, count(*) AS c FROM requests GROUP BY service') as $r) {
                $byService[$r['service']] = (int)$r['c'];
            }

            $recent = array_map('map_request', db_all('SELECT * FROM requests ORDER BY created_at DESC LIMIT 3'));

            j_ok([
                'revenue' => $paid,
                'pipeline' => $pipeline,
                'paid' => $paid,
                'unpaid' => $unpaid,
                'pending' => $pending,
                'requestCount' => $requestCount,
                'userCount' => $userCount,
                'byService' => $byService,
                'recent' => $recent,
            ]);
        }

        case 'visitor_stats': {
            require_admin();

            $today = (int)(db_one("SELECT count(*) AS c FROM visitor_logs WHERE visit_date = CURDATE()")['c'] ?? 0);
            $yesterday = (int)(db_one("SELECT count(*) AS c FROM visitor_logs WHERE visit_date = DATE_SUB(CURDATE(), INTERVAL 1 DAY)")['c'] ?? 0);
            $thisWeek = (int)(db_one("SELECT count(*) AS c FROM visitor_logs WHERE visit_date >= DATE_SUB(CURDATE(), INTERVAL 6 DAY)")['c'] ?? 0);
            $thisMonth = (int)(db_one("SELECT count(*) AS c FROM visitor_logs WHERE visit_date >= DATE_FORMAT(CURDATE(), '%Y-%m-01')")['c'] ?? 0);

            $last7 = db_all("
                SELECT
                    DATE(visit_date) AS d,
                    COUNT(*) AS c
                FROM visitor_logs
                WHERE visit_date >= DATE_SUB(CURDATE(), INTERVAL 6 DAY)
                GROUP BY DATE(visit_date)
                ORDER BY DATE(visit_date) ASC
            ");
            $last7Map = [];
            for ($i = 6; $i >= 0; $i--) {
                $d = date('Y-m-d', strtotime("-$i days"));
                $last7Map[$d] = 0;
            }
            foreach ($last7 as $r) { $last7Map[$r['d']] = (int)$r['c']; }

            $topPages = db_all("
                SELECT page_path, COUNT(*) AS views
                FROM visitor_logs
                WHERE visit_date >= DATE_SUB(CURDATE(), INTERVAL 30 DAY)
                GROUP BY page_path
                ORDER BY views DESC
                LIMIT 5
            ");

            j_ok([
                'today' => $today,
                'yesterday' => $yesterday,
                'this_week' => $thisWeek,
                'this_month' => $thisMonth,
                'last_7_days' => array_map(function ($date, $count) {
                    return ['date' => $date, 'label' => date('D', strtotime($date)), 'count' => $count];
                }, array_keys($last7Map), array_values($last7Map)),
                'top_pages' => array_map(function ($r) {
                    return ['page' => $r['page_path'], 'views' => (int)$r['views']];
                }, $topPages),
            ]);
        }

        default:
            j_err('Unknown dashboard action.', 404);
    }
} catch (Throwable $e) {
    error_log('[api/dashboard] ' . $e->getMessage());
    j_err('Server error. Please try again.', 500);
}

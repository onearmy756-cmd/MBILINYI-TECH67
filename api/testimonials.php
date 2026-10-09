<?php
declare(strict_types=1);
require_once __DIR__ . '/../config/app.php';

header('Content-Type: application/json; charset=utf-8');

try {
    switch (action_name()) {

        case 'list': {
            $rows = db_all('SELECT * FROM testimonials ORDER BY sort_order ASC, id DESC');
            j_ok(['testimonials' => array_map('map_testimonial', $rows)]);
        }

        case 'add': {
            $admin = require_admin();
            require_csrf();
            $name = trim(str_param('name', str_param('n', '')));
            $role = trim(str_param('role', str_param('r', '')));
            $quote = trim(str_param('quote', str_param('text', str_param('t', ''))));
            $stars = (int)param('stars', 5);
            $sortOrder = (int)param('sort_order', 0);
            if ($name === '' || $quote === '') j_err('Name and quote are required.', 400);
            if ($stars < 1) $stars = 1; if ($stars > 5) $stars = 5;

            $tid = new_id('TS-');
            db_run('INSERT INTO testimonials (id, name, role, text, stars, sort_order, is_active) VALUES (?,?,?,?,?,?,1)',
                [$tid, $name, $role, $quote, $stars, $sortOrder]);
            $row = db_one('SELECT * FROM testimonials WHERE id = ?', [$tid]);
            j_ok(['testimonial' => $row ? map_testimonial($row) : null]);
        }

        case 'update': {
            $admin = require_admin();
            require_csrf();
            $id = str_param('id');
            if ($id === '') j_err('ID required.', 400);
            $name = trim(str_param('name', str_param('n', '')));
            $role = trim(str_param('role', str_param('r', '')));
            $quote = trim(str_param('quote', str_param('text', str_param('t', ''))));
            $stars = (int)param('stars', 5);
            $sortOrder = (int)param('sort_order', 0);
            if ($name === '' || $quote === '') j_err('Name and quote are required.', 400);
            if ($stars < 1) $stars = 1; if ($stars > 5) $stars = 5;

            db_run('UPDATE testimonials SET name=?, role=?, text=?, stars=?, sort_order=?, updated_at=now() WHERE id=?',
                [$name, $role, $quote, $stars, $sortOrder, $id]);

            $p = db_one('SELECT * FROM testimonials WHERE id = ?', [$id]);
            j_ok(['testimonial' => $p ? map_testimonial($p) : null]);
        }

        case 'delete': {
            $admin = require_admin();
            require_csrf();
            $id = str_param('id');
            if ($id === '') j_err('ID required.', 400);
            db_run('DELETE FROM testimonials WHERE id = ?', [$id]);
            j_ok(['deleted' => $id]);
        }

        default:
            j_err('Unknown testimonials action.', 404);
    }
} catch (Throwable $e) {
    error_log('[api/testimonials] ' . $e->getMessage());
    j_err('Server error. Please try again.', 500);
}

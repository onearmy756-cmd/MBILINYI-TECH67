<?php
declare(strict_types=1);
require_once __DIR__ . '/../config/app.php';

header('Content-Type: application/json; charset=utf-8');

try {
    switch (action_name()) {

        case 'list': {
            $rows = db_all('SELECT * FROM portfolio_projects ORDER BY sort_order ASC, id DESC');
            j_ok(['projects' => array_map('map_portfolio_project', $rows)]);
        }

        case 'add': {
            $admin = require_admin();
            require_csrf();
            $title = trim(str_param('title', ''));
            $category = trim(str_param('category', ''));
            $imageUrl = trim(str_param('image_url', str_param('img', '')));
            $description = trim(str_param('description', str_param('d', '')));
            $stat = trim(str_param('stat', str_param('s', '')));
            $link = trim(str_param('link', ''));
            $sortOrder = (int)param('sort_order', 0);
            if ($title === '') j_err('Title is required.', 400);

            /* id is varchar — generated here, never auto-increment. */
            $pid = new_id('P-');
            db_run('INSERT INTO portfolio_projects (id, title, category, image_url, description, stat_label, link_url, sort_order, is_active)
                    VALUES (?,?,?,?,?,?,?,?,1)',
                [$pid, $title, $category, $imageUrl, $description, $stat, $link, $sortOrder]);
            $p = db_one('SELECT * FROM portfolio_projects WHERE id = ?', [$pid]);
            j_ok(['project' => map_portfolio_project($p)]);
        }

        case 'update': {
            $admin = require_admin();
            require_csrf();
            $id = str_param('id');
            if ($id === '') j_err('ID required.', 400);
            $title = trim(str_param('title', ''));
            $category = trim(str_param('category', ''));
            $imageUrl = trim(str_param('image_url', str_param('img', '')));
            $description = trim(str_param('description', str_param('d', '')));
            $stat = trim(str_param('stat', str_param('s', '')));
            $link = trim(str_param('link', ''));
            $sortOrder = (int)param('sort_order', 0);
            if ($title === '') j_err('Title is required.', 400);

            db_run('UPDATE portfolio_projects SET title=?, category=?, image_url=?, description=?, stat_label=?, link_url=?, sort_order=?, updated_at=now() WHERE id=?',
                [$title, $category, $imageUrl, $description, $stat, $link, $sortOrder, $id]);

            $p = db_one('SELECT * FROM portfolio_projects WHERE id = ?', [$id]);
            j_ok(['project' => $p ? map_portfolio_project($p) : null]);
        }

        case 'delete': {
            $admin = require_admin();
            require_csrf();
            $id = str_param('id');
            if ($id === '') j_err('ID required.', 400);
            db_run('DELETE FROM portfolio_projects WHERE id = ?', [$id]);
            j_ok(['deleted' => $id]);
        }

        default:
            j_err('Unknown portfolio action.', 404);
    }
} catch (Throwable $e) {
    error_log('[api/portfolio] ' . $e->getMessage());
    j_err('Server error. Please try again.', 500);
}

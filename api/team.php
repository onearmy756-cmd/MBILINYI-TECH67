<?php
declare(strict_types=1);
require_once __DIR__ . '/../config/app.php';

header('Content-Type: application/json; charset=utf-8');

try {
    switch (action_name()) {

        /* public + signed-in users — only active members are ever returned */
        case 'list': {
            $all = str_param('all') === '1' && current_user() && (current_user()['role'] ?? '') === 'admin';
            $sql = $all
                ? 'SELECT * FROM team_members ORDER BY sort_order ASC, created_at ASC'
                : 'SELECT * FROM team_members WHERE is_active = 1 ORDER BY sort_order ASC, created_at ASC';
            j_ok(['team' => array_map('map_team_member', db_all($sql))]);
        }

        case 'add': {
            require_admin();
            require_csrf();
            $name = trim(str_param('name'));
            if ($name === '') j_err('Name is required.', 400);
            $id = new_id('TM-');
            db_run('INSERT INTO team_members (id,name,role,role_sw,bio,bio_sw,image_url,email,phone,linkedin,sort_order,is_active) VALUES (?,?,?,?,?,?,?,?,?,?,?,1)',
                [$id, $name, trim(str_param('role')), trim(str_param('role_sw')), trim(str_param('bio')), trim(str_param('bio_sw')),
                    trim(str_param('image_url', str_param('img', ''))),
                    trim(str_param('email')), trim(str_param('phone')), trim(str_param('linkedin')), (int)param('sort_order', 0)]);
            $row = db_one('SELECT * FROM team_members WHERE id = ?', [$id]);
            j_ok(['member' => $row ? map_team_member($row) : null]);
        }

        case 'update': {
            require_admin();
            require_csrf();
            $id = str_param('id');
            if ($id === '') j_err('ID required.', 400);
            $row = db_one('SELECT * FROM team_members WHERE id = ?', [$id]);
            if (!$row) j_err('Team member not found.', 404);
            $name = trim(str_param('name', $row['name']));
            if ($name === '') j_err('Name is required.', 400);
            db_run('UPDATE team_members SET name=?, role=?, role_sw=?, bio=?, bio_sw=?, image_url=?, email=?, phone=?, linkedin=?, sort_order=? WHERE id=?',
                [$name, trim(str_param('role', $row['role'])), trim(str_param('role_sw', $row['role_sw'])),
                    trim(str_param('bio', $row['bio'])), trim(str_param('bio_sw', $row['bio_sw'])),
                    trim(str_param('image_url', str_param('img', $row['image_url']))), trim(str_param('email', $row['email'])),
                    trim(str_param('phone', $row['phone'])), trim(str_param('linkedin', $row['linkedin'])),
                    (int)param('sort_order', (int)$row['sort_order']), $id]);
            $p = db_one('SELECT * FROM team_members WHERE id = ?', [$id]);
            j_ok(['member' => $p ? map_team_member($p) : null]);
        }

        case 'toggle': {
            require_admin();
            require_csrf();
            $id = str_param('id');
            $row = db_one('SELECT * FROM team_members WHERE id = ?', [$id]);
            if (!$row) j_err('Team member not found.', 404);
            db_run('UPDATE team_members SET is_active = ? WHERE id = ?', [(int)!((bool)$row['is_active']), $id]);
            $p = db_one('SELECT * FROM team_members WHERE id = ?', [$id]);
            j_ok(['member' => $p ? map_team_member($p) : null]);
        }

        case 'delete': {
            require_admin();
            require_csrf();
            $id = str_param('id');
            if ($id === '') j_err('ID required.', 400);
            db_run('DELETE FROM team_members WHERE id = ?', [$id]);
            j_ok(['deleted' => $id]);
        }

        default:
            j_err('Unknown team action.', 404);
    }
} catch (Throwable $e) {
    error_log('[api/team] ' . $e->getMessage());
    j_err('Server error. Please try again.', 500);
}

<?php
declare(strict_types=1);
require_once __DIR__ . '/../config/app.php';

header('Content-Type: application/json; charset=utf-8');

try {
    switch (action_name()) {

        /* public + signed-in users — only published posts are ever returned */
        case 'list': {
            $all = str_param('all') === '1' && current_user() && (current_user()['role'] ?? '') === 'admin';
            $sql = $all
                ? 'SELECT * FROM news_posts ORDER BY created_at DESC'
                : 'SELECT * FROM news_posts WHERE is_active = 1 ORDER BY created_at DESC';
            j_ok(['news' => array_map('map_news', db_all($sql))]);
        }

        case 'get': {
            $id = str_param('id');
            $row = db_one('SELECT * FROM news_posts WHERE id = ?', [$id]);
            if (!$row) j_err('News post not found.', 404);
            if (!(bool)$row['is_active']) {
                $u = current_user();
                if (!$u || ($u['role'] ?? '') !== 'admin') j_err('News post not found.', 404);
            }
            j_ok(['post' => map_news($row)]);
        }

        case 'add': {
            require_admin();
            require_csrf();
            $title = trim(str_param('title'));
            $category = trim(str_param('category', 'Announcement'));
            $excerpt = trim(str_param('excerpt'));
            $body = trim(str_param('body'));
            $image = trim(str_param('image_url', str_param('img', '')));
            if ($title === '' || $body === '') j_err('Title and body are required.', 400);
            if ($category === '') $category = 'Announcement';
            if ($excerpt === '') $excerpt = mb_substr(strip_tags($body), 0, 180);

            $id = new_id('N-');
            db_run('INSERT INTO news_posts (id,title,category,excerpt,body,image_url,is_active) VALUES (?,?,?,?,?,?,1)',
                [$id, $title, $category, $excerpt, $body, $image]);
            $row = db_one('SELECT * FROM news_posts WHERE id = ?', [$id]);
            j_ok(['post' => $row ? map_news($row) : null]);
        }

        case 'update': {
            require_admin();
            require_csrf();
            $id = str_param('id');
            if ($id === '') j_err('ID required.', 400);
            $row = db_one('SELECT * FROM news_posts WHERE id = ?', [$id]);
            if (!$row) j_err('News post not found.', 404);
            $title = trim(str_param('title', $row['title']));
            $category = trim(str_param('category', $row['category']));
            $excerpt = trim(str_param('excerpt', $row['excerpt']));
            $body = trim(str_param('body', $row['body']));
            $image = trim(str_param('image_url', str_param('img', $row['image_url'])));
            if ($title === '' || $body === '') j_err('Title and body are required.', 400);
            db_run('UPDATE news_posts SET title=?, category=?, excerpt=?, body=?, image_url=? WHERE id=?',
                [$title, $category, $excerpt, $body, $image, $id]);
            $p = db_one('SELECT * FROM news_posts WHERE id = ?', [$id]);
            j_ok(['post' => $p ? map_news($p) : null]);
        }

        /* publish / unpublish without deleting the article */
        case 'toggle': {
            require_admin();
            require_csrf();
            $id = str_param('id');
            $row = db_one('SELECT * FROM news_posts WHERE id = ?', [$id]);
            if (!$row) j_err('News post not found.', 404);
            db_run('UPDATE news_posts SET is_active = ? WHERE id = ?', [(int)!((bool)$row['is_active']), $id]);
            $p = db_one('SELECT * FROM news_posts WHERE id = ?', [$id]);
            j_ok(['post' => $p ? map_news($p) : null]);
        }

        case 'delete': {
            require_admin();
            require_csrf();
            $id = str_param('id');
            if ($id === '') j_err('ID required.', 400);
            db_run('DELETE FROM news_posts WHERE id = ?', [$id]);
            j_ok(['deleted' => $id]);
        }

        default:
            j_err('Unknown news action.', 404);
    }
} catch (Throwable $e) {
    error_log('[api/news] ' . $e->getMessage());
    j_err('Server error. Please try again.', 500);
}

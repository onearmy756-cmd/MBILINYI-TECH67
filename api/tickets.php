<?php
declare(strict_types=1);
require_once __DIR__ . '/../config/app.php';

header('Content-Type: application/json; charset=utf-8');

function ticket_owned_by_user(array $t, array $user): bool {
    if ($user['role'] === 'admin') return true;
    return $t['user_id'] === $user['id'] || strtolower($t['email']) === strtolower($user['email']);
}

function ticket_with_replies(array $t): array {
    $rows = db_all('SELECT * FROM ticket_replies WHERE ticket_id = ? ORDER BY created_at ASC', [$t['id']]);
    return map_ticket($t, $rows);
}

try {
    switch (action_name()) {

        /* public: contact form + portal */
        case 'create': {
            require_csrf();
            $name = str_param('name');
            $email = strtolower(str_param('email'));
            $subject = str_param('subject', 'Website inquiry');
            $message = str_param('message');
            if ($name === '' || $email === '' || $message === '') {
                j_err('Please fill name, email and message. / Jaza sehemu zote muhimu.');
            }
            if (!filter_var($email, FILTER_VALIDATE_EMAIL)) j_err('Invalid email address.');
            if ($subject === '') $subject = 'Website inquiry';

            $u = current_user();
            $id = new_id('T-');
            db_run(
                'INSERT INTO tickets (id,user_id,name,email,subject,message,status,created_at)
                 VALUES (?,?,?,?,?,?,?,now())',
                [$id, $u ? $u['id'] : 'GUEST', $name, $email, $subject, $message, 'Open']
            );
            $t = ticket_with_replies(db_one('SELECT * FROM tickets WHERE id = ?', [$id]));
            mts_notify([MTS_CEO_EMAIL], 'New support ticket — ' . $subject,
                'Support ticket mpya 🎧',
                '<p><strong>' . htmlspecialchars($name) . '</strong> (' . htmlspecialchars($email) . ')</p>'
                . '<p>' . nl2br(htmlspecialchars($message)) . '</p>');
            j_ok(['ticket' => $t]);
        }

        case 'mine': {
            $u = require_user();
            $rows = db_all(
                'SELECT * FROM tickets WHERE user_id = ? OR lower(email) = ? ORDER BY created_at DESC',
                [$u['id'], strtolower($u['email'])]
            );
            j_ok(['tickets' => array_map('ticket_with_replies', $rows)]);
        }

        case 'reply': {
            require_csrf();
            $u = require_user();
            $id = str_param('id');
            $text = str_param('text');
            if ($text === '') j_err('Write a reply first.');
            $t = db_one('SELECT * FROM tickets WHERE id = ?', [$id]);
            if (!$t) j_err('Ticket not found.', 404);
            if (!ticket_owned_by_user($t, $u)) j_err('Not your ticket.', 403);
            db_run('INSERT INTO ticket_replies (id,ticket_id,by_name,body,created_at) VALUES (?,?,?,?,now())',
                [new_id('TR-'), $id, $u['name'] . ' (Client)', $text]);
            notify_ticket_reply($t, $u['name'] . ' (Client)', $text);
            j_ok(['ticket' => ticket_with_replies(db_one('SELECT * FROM tickets WHERE id = ?', [$id]))]);
        }

        case 'all': {
            require_admin();
            $rows = db_all('SELECT * FROM tickets ORDER BY created_at DESC');
            j_ok(['tickets' => array_map('ticket_with_replies', $rows)]);
        }

        case 'admin_reply': {
            require_csrf();
            require_admin();
            $id = str_param('id');
            $text = str_param('text');
            if ($text === '') j_err('Write a reply first.');
            $t = db_one('SELECT * FROM tickets WHERE id = ?', [$id]);
            if (!$t) j_err('Ticket not found.', 404);
            db_run('INSERT INTO ticket_replies (id,ticket_id,by_name,body,created_at) VALUES (?,?,?,?,now())',
                [new_id('TR-'), $id, 'Jackson Mbilinyi (CEO)', $text]);
            notify_ticket_reply($t, 'Jackson Mbilinyi (CEO)', $text);
            j_ok(['ticket' => ticket_with_replies(db_one('SELECT * FROM tickets WHERE id = ?', [$id]))]);
        }

        case 'admin_delete': {
            require_csrf();
            require_admin();
            $id = str_param('id');
            db_run('DELETE FROM ticket_replies WHERE ticket_id = ?', [$id]);
            db_run('DELETE FROM tickets WHERE id = ?', [$id]);
            j_ok([]);
        }

        case 'toggle': {
            require_csrf();
            require_admin();
            $id = str_param('id');
            $t = db_one('SELECT * FROM tickets WHERE id = ?', [$id]);
            if (!$t) j_err('Ticket not found.', 404);
            $next = $t['status'] === 'Open' ? 'Closed' : 'Open';
            db_run('UPDATE tickets SET status = ? WHERE id = ?', [$next, $id]);
            j_ok(['status' => $next]);
        }

        default:
            j_err('Unknown tickets action.', 404);
    }
} catch (Throwable $e) {
    error_log('[api/tickets] ' . $e->getMessage());
    j_err('Server error. Please try again.', 500);
}

# End-to-end smoke test against the local dev server (CSRF-aware).
$Base = 'http://127.0.0.1:8090'

# Clean URLs: the browser never sends .php, so neither does this suite.
# (.php is only stripped when it is the last path segment, before ? or end.)
function Clean($uri) { return ($uri -replace '\.php(?=\?|$)', '') }

function New-Sess {
    $s = New-Object Microsoft.PowerShell.Commands.WebRequestSession
    $p = Invoke-WebRequest -Uri (Clean "$Base/index.php") -UseBasicParsing -TimeoutSec 30 -WebSession $s
    $m = [regex]::Match($p.Content, 'window\.__CSRF__\s*=\s*"([a-f0-9]+)"')
    $s.Headers['X-CSRF-Token'] = $m.Groups[1].Value
    return $s
}

function Api($sess, $uri, $body) {
    $p = @{ Uri = (Clean $uri); UseBasicParsing = $true; TimeoutSec = 45; WebSession = $sess }
    if ($body) { $p.Method = 'Post'; $p.Body = ($body | ConvertTo-Json -Depth 8); $p.ContentType = 'application/json' }
    try { $r = Invoke-WebRequest @p; return @{ code = [int]$r.StatusCode; raw = $r.Content } }
    catch {
        $resp = $_.Exception.Response
        if (-not $resp) { return @{ code = 0; raw = $_.Exception.Message } }
        $sr = New-Object System.IO.StreamReader($resp.GetResponseStream())
        return @{ code = [int]$resp.StatusCode; raw = $sr.ReadToEnd() }
    }
}

$pass = 0; $fail = 0
function Step($n, $ok, $extra = '') {
    if ($ok) { $script:pass++ ; Write-Output "[ OK ] $n  $extra" }
    else     { $script:fail++ ; Write-Output "[FAIL] $n  $extra" }
}

function Json($raw) { try { return $raw | ConvertFrom-Json } catch { return $null } }

# Login rotates the session CSRF token (config/app.php), so re-read it from the
# page the same way the browser does after a redirect.
function Refresh-Csrf($sess) {
    $p = Invoke-WebRequest -Uri (Clean "$Base/index.php") -UseBasicParsing -TimeoutSec 30 -WebSession $sess
    $m = [regex]::Match($p.Content, 'window\.__CSRF__\s*=\s*"([a-f0-9]+)"')
    $sess.Headers['X-CSRF-Token'] = $m.Groups[1].Value
    return $m.Groups[1].Value
}

Write-Output '======== ADMIN LOGIN + OTP ========'
$A = New-Sess
$lr = Api $A "$Base/api/auth.php" @{ action = 'login'; email = 'admin@mbilinyitech.co.tz'; password = 'Admin@2026' }
$ld = Json $lr.raw
$code = $ld.data.otp_debug_only
Step 'admin login issues OTP' ($ld.ok -and $ld.data.otp_required -eq $true -and [bool]$code) "code=$code"

$vr = Api $A "$Base/api/auth.php" @{ action = 'verify-login-otp'; email = 'admin@mbilinyitech.co.tz'; code = $code }
$vd = Json $vr.raw
Step 'admin OTP verify -> role=admin' ($vd.ok -and $vd.data.user.role -eq 'admin') "role=$($vd.data.user.role)"
Refresh-Csrf $A | Out-Null

Write-Output '======== SETTINGS / PAYMENT / PRICES ========'
$sr = Api $A "$Base/api/settings.php?action=list"
$sd = Json $sr.raw
Step 'payment: NMB account' ($sd.ok -and $sd.data.payment.bank.account -eq '51710099563') $sd.data.payment.bank.account
Step 'payment: Vodacom number' ($sd.data.payment.mobile.number -eq '0796752645') "$($sd.data.payment.mobile.network) $($sd.data.payment.mobile.number)"
Step 'service prices loaded' ($sd.data.servicePrices.Count -ge 10) "count=$($sd.data.servicePrices.Count)"

$sv = Api $A "$Base/api/settings.php" @{ action = 'save_all'; settings = @{ pay_bank_account = '51710099563'; price_website_basic = '300000' } }
$svd = Json $sv.raw
Step 'settings save_all' ($svd.ok) "count=$($svd.data.count)"

$sv2 = Api $A "$Base/api/settings.php" @{ action = 'save'; key = 'company_location'; value = 'Dodoma, Tanzania' }
Step 'settings save single' ((Json $sv2.raw).ok) ''

Write-Output '======== PRICING ========'
$pr = Api $A "$Base/api/settings.php?action=plans"
$pd = Json $pr.raw
Step 'public price plans' ($pd.ok -and $pd.data.plans.Count -ge 6) "count=$($pd.data.plans.Count)"

$ps = Api $A "$Base/api/settings.php" @{ action = 'plan_save'; id = 'P-STDCARE'; name = 'Starter Care'; price = 150000; period = '/month' }
Step 'plan_save (admin edits price)' ((Json $ps.raw).ok) ''

Write-Output '======== VISITOR STATS ========'
$vs = Api $A "$Base/api/dashboard.php?action=visitor_stats"
$vsd = Json $vs.raw
Step 'visitor_stats' ($vsd.ok) "today=$($vsd.data.today) week=$($vsd.data.this_week) month=$($vsd.data.this_month) pages=$($vsd.data.top_pages.Count)"

Write-Output '======== FEEDBACK ========'
$fb = Api $A "$Base/api/settings.php" @{ action = 'feedback'; name = 'E2E Tester'; email = 'test@example.com'; message = 'Mfumo mzuri sana!'; category = 'praise'; rating = 5 }
$fbd = Json $fb.raw
Step 'feedback create' ($fbd.ok) "id=$($fbd.data.id)"
$fl = Api $A "$Base/api/settings.php?action=feedback_list"
$fld = Json $fl.raw
Step 'feedback_list (admin)' ($fld.ok -and $fld.data.feedback.Count -ge 1) "count=$($fld.data.feedback.Count)"
$fs = Api $A "$Base/api/settings.php" @{ action = 'feedback_status'; id = $fbd.data.id; status = 'Replied' }
Step 'feedback status update' ((Json $fs.raw).ok) ''

Write-Output '======== CONTRACT IS PERMANENT ========'
$cd = Api $A "$Base/api/contracts.php" @{ action = 'delete'; id = 'X' }
Step 'contract delete blocked' ($cd.raw -match 'cannot be deleted') "HTTP $($cd.code)"

Write-Output '======== PORTFOLIO CRUD ========'
$pf = Api $A "$Base/api/portfolio.php" @{ action = 'add'; title = 'E2E Test Project'; category = 'Testing'; description = 'Automated test row'; image_url = 'https://example.com/x.jpg'; stat = 'PASS'; link = 'https://example.com' }
$pfd = Json $pf.raw
Step 'portfolio create' ($pfd.ok) "id=$($pfd.data.project.id) raw=$($pf.raw)"
if ($pfd.ok) {
    $projId = $pfd.data.project.id
    $pu = Api $A "$Base/api/portfolio.php" @{ action = 'update'; id = $projId; title = 'E2E Test Project (edited)'; category = 'Testing'; description = 'Edited row'; image_url = 'https://example.com/y.jpg'; stat = 'PASS v2'; link = 'https://example.com/2'; sort_order = 9 }
    Step 'portfolio update' ((Json $pu.raw).ok) ''
    $pl = Api $A "$Base/api/portfolio.php" @{ action = 'list' }
    $p = (Json $pl.raw).data.projects
    $hit = $p | Where-Object { $_.id -eq $projId }
    Step 'portfolio list reflects edit' ($hit.statLabel -eq 'PASS v2' -and $hit.title -eq 'E2E Test Project (edited)') "stat=$($hit.statLabel)"
    $del = Api $A "$Base/api/portfolio.php" @{ action = 'delete'; id = $projId }
    Step 'portfolio delete' ((Json $del.raw).ok) ''
}

Write-Output '======== TESTIMONIAL CRUD ========'
$tf = Api $A "$Base/api/testimonials.php" @{ action = 'add'; name = 'E2E Reviewer'; role = 'QA - Dodoma'; quote = 'Kazi nzuri sana!'; stars = 5; sort_order = 99 }
$tfd = Json $tf.raw
Step 'testimonial create' ($tfd.ok) "id=$($tfd.data.testimonial.id) raw=$($tf.raw)"
if ($tfd.ok) {
    $tid = $tfd.data.testimonial.id
    $tu = Api $A "$Base/api/testimonials.php" @{ action = 'update'; id = $tid; name = 'E2E Reviewer'; role = 'QA - Dodoma'; quote = 'Kazi nzuri sana! (edited)'; stars = 4; sort_order = 99 }
    Step 'testimonial update' ((Json $tu.raw).ok) ''
    $tdel = Api $A "$Base/api/testimonials.php" @{ action = 'delete'; id = $tid }
    Step 'testimonial delete' ((Json $tdel.raw).ok) ''
}

Write-Output '======== CLIENT FLOW ========'
$C = New-Sess
$clr = Api $C "$Base/api/auth.php" @{ action = 'login'; email = 'demo@client.com'; password = 'demo123' }
$cld = Json $clr.raw
Step 'client login issues OTP' ($cld.ok -and [bool]$cld.data.otp_debug_only) "code=$($cld.data.otp_debug_only)"
$cvr = Api $C "$Base/api/auth.php" @{ action = 'verify-login-otp'; email = 'demo@client.com'; code = $cld.data.otp_debug_only }
$cvd = Json $cvr.raw
Step 'client verify -> role=client' ($cvd.ok -and $cvd.data.user.role -eq 'client') "role=$($cvd.data.user.role)"
Refresh-Csrf $C | Out-Null

$rs = Api $C "$Base/api/auth.php" @{ action = 'resend-login-otp'; email = 'demo@client.com' }
Step 'client resend-login-otp' ((Json $rs.raw).ok) $rs.raw.Substring(0, [Math]::Min(120, $rs.raw.Length))

$cset = Api $C "$Base/api/settings.php?action=list"
$csd = Json $cset.raw
Step 'client sees payment details' ($csd.ok -and $csd.data.payment.bank.account -eq '51710099563') "bank=$($csd.data.payment.bank.account) mobile=$($csd.data.payment.mobile.number)"

Write-Output '======== FORGOT PASSWORD ========'
$fpr = Api $C "$Base/api/auth.php" @{ action = 'forgot-password-init'; email = 'demo@client.com' }
$fprj = Json $fpr.raw
Step 'forgot_password accepted' ($fprj.ok) $fpr.raw.Substring(0, [Math]::Min(160, $fpr.raw.Length))

Write-Output ''
Write-Output "======== RESULT: $pass passed, $fail failed ========"

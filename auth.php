<?php
require_once __DIR__ . '/config/app.php';
$me = current_user();
if ($me) redirect($me['role'] === 'admin' ? 'admin' : 'client');
$pageTitle = 'Mbilinyi Tech Solutions — Client Portal | Sign In';
$page = 'auth';
/* Accept both the clean form (returnTo=client) and the legacy one
 * (returnTo=client.php) so old bookmarks still land on the right page. */
$returnTo = str_param('returnTo', '');
$returnTo = rtrim(preg_replace('/\.php$/', '', $returnTo), '/');
if (!in_array($returnTo, ['client', 'admin'], true)) $returnTo = '';
require __DIR__ . '/partials/head.php';
?>
<body class="antialiased">
<?php require __DIR__ . '/partials/header.php'; ?>

<!-- ============ AUTH VIEW ============ -->
<div id="view-auth" class="view-section active">
  <div class="max-w-5xl mx-auto px-4 py-12 grid md:grid-cols-2 gap-6 items-stretch">
    <div class="rounded-3xl p-[1.5px] bg-gradient-to-br from-cyan-400 to-violet-600">
      <div class="rounded-3xl bg-[#070D1F] p-8 h-full flex flex-col justify-between relative overflow-hidden">
        <div class="orb w-[300px] h-[300px] bg-violet-600/30 -top-20 -right-20"></div>
        <div class="relative">
          <div class="w-14 h-14 rounded-2xl grad-btn flex items-center justify-center font-display font-bold text-2xl">M</div>
          <h2 class="font-display font-bold text-3xl mt-5"><span data-en="Client" data-sw="Portal ya">Client</span> <span class="grad-text" data-en="Portal" data-sw="Mteja">Portal</span></h2>
          <p class="text-white/55 text-[14px] mt-2" data-en="One secure login for everything: submit requirements, receive official quotations, e-sign contracts, pay invoices & chat with support." data-sw="Kuingia moja salama kwa kila kitu: tuma mahitaji, pokea nukuu rasmi, saini mikataba kidijitali, lipa ankara na zungumza na msaada.">One secure login for everything: submit requirements, receive official quotations, e-sign contracts, pay invoices & chat with support.</p>
          <div class="grid gap-2.5 mt-6 text-[13px]">
            <div class="flex items-center gap-3 glass rounded-xl px-4 py-3"><i class="fa-solid fa-file-pen text-cyan-300"></i> <span data-en="Requirement forms with tracking IDs" data-sw="Fomu za mahitaji zenye namba za kufuatilia">Requirement forms with tracking IDs</span></div>
            <div class="flex items-center gap-3 glass rounded-xl px-4 py-3"><i class="fa-solid fa-file-invoice-dollar text-gold"></i> <span data-en="Official quotations & transparent pricing" data-sw="Nukuu rasmi na bei iliyo wazi">Official quotations & transparent pricing</span></div>
            <div class="flex items-center gap-3 glass rounded-xl px-4 py-3"><i class="fa-solid fa-file-contract text-violet-300"></i> <span data-en="BRELA contracts with e-signature" data-sw="Mikataba ya BRELA yenye sahihi ya kidijitali">BRELA contracts with e-signature</span></div>
            <div class="flex items-center gap-3 glass rounded-xl px-4 py-3"><i class="fa-solid fa-users-gear text-emerald-300"></i> <span data-en="Full account management" data-sw="Usimamizi kamili wa akaunti">Full account management</span></div>
          </div>
        </div>
        <div class="relative mt-6 rounded-2xl bg-cyan-500/[.06] border border-cyan-400/25 p-4 text-[12px] text-white/70">
          <div class="font-bold text-cyan-300 mb-1"><i class="fa-solid fa-wand-magic-sparkles mr-1"></i> <span data-en="ONE LOGIN — AUTO-DETECTED" data-sw="KUINGIA MOJA — HUTAMBULIWA KIOTOMATIKI">ONE LOGIN — AUTO-DETECTED</span></div>
          <span data-en="Sign in once with your email and password. Your requests, quotations, contracts and invoices all live in one secure portal." data-sw="Ingia mara moja kwa email na password yako. Maombi, nukuu, mikataba na ankara zako zote ziko kwenye portal moja salama.">Sign in once with your email and password. Your requests, quotations, contracts and invoices all live in one secure portal.</span>
        </div>
      </div>
    </div>
    <div class="glass rounded-3xl p-6 md:p-8">
      <div class="grid grid-cols-2 gap-2 p-1.5 rounded-2xl bg-white/[.04] border border-white/10">
        <button id="tabLogin" onclick="switchAuthTab('login')" class="py-3 rounded-xl font-bold text-sm grad-btn"><span data-en="Sign In" data-sw="Ingia">Sign In</span></button>
        <button id="tabRegister" onclick="switchAuthTab('register')" class="py-3 rounded-xl font-bold text-sm text-white/60"><span data-en="Create Account" data-sw="Fungua Akaunti">Create Account</span></button>
      </div>

      <!-- ---------- SIGN IN ---------- -->
      <div id="authLogin" class="mt-6">
        <div class="mb-5">
          <div class="font-display font-bold text-xl" data-en="Welcome back" data-sw="Karibu tena">Welcome back</div>
          <div class="text-white/50 text-[13px] mt-1" data-en="Sign in to track requests, quotations, contracts and invoices." data-sw="Ingia kufuatilia maombi, nukuu, mikataba na ankara zako.">Sign in to track requests, quotations, contracts and invoices.</div>
        </div>
        <label class="text-[12px] font-bold text-white/60" data-en="EMAIL ADDRESS" data-sw="BARUA PEPE">EMAIL ADDRESS</label>
        <div class="ifield mt-1.5"><i class="fa-solid fa-envelope"></i><input id="loginEmail" class="input input-i" placeholder="you@company.com"></div>
        <label class="text-[12px] font-bold text-white/60 mt-4 block" data-en="PASSWORD" data-sw="NENO LA SIRI">PASSWORD</label>
        <div class="ifield mt-1.5"><i class="fa-solid fa-lock"></i><input id="loginPass" type="password" class="input input-i !pr-12" placeholder="••••••••" onkeydown="if(event.key==='Enter')doLogin()"><button onclick="togglePw('loginPass',this)" class="absolute right-3 top-1/2 -translate-y-1/2 text-white/40 hover:text-cyan-300"><i class="fa-solid fa-eye"></i></button></div>
        <button onclick="doLogin()" id="btnDoLogin" class="grad-btn w-full mt-6 py-4 rounded-2xl font-bold"><i class="fa-solid fa-right-to-bracket mr-2"></i><span data-en="Sign In Securely" data-sw="Ingia Kwa Usalama">Sign In Securely</span></button>
        <div class="flex justify-between mt-4 text-[12px]">
          <a href="javascript:void(0)" onclick="openForgotPassword()" class="text-cyan-300 hover:text-cyan-200 font-bold"><span data-en="Forgot Password?" data-sw="Umesahau Neno la Siri?">Forgot Password?</span></a>
          <a href="javascript:void(0)" onclick="openResendCodeFlow()" class="text-white/50 hover:text-white/80"><span data-en="Need new login code?" data-sw="Nahitaji code mpya?">Need new login code?</span></a>
        </div>
        <div class="flex items-center justify-center gap-1.5 text-center text-[12px] text-white/40 mt-4"><i class="fa-solid fa-shield-halved text-emerald-300"></i><span data-en="Protected with encrypted sessions" data-sw="Inalindwa kwa sessions zilizosimbwa">Protected with encrypted sessions</span></div>
      </div>

      <!-- ---------- PASSWORD RESET ---------- -->
      <div id="authReset" class="mt-6 hidden">
        <div class="glass rounded-2xl p-4 border border-violet-400/30 bg-violet-500/[.04]">
          <div class="font-display font-bold text-lg text-violet-200"><i class="fa-solid fa-key mr-1"></i><span data-en="Reset Your Password" data-sw="Badili Neno la Siri">Reset Your Password</span></div>
          <p class="text-white/55 text-[12px] mt-1"><span data-en="Enter a new password (6+ characters) to restore access to your account." data-sw="Ingiza neno la siri jipya (6+ herufi) la kurejesha ufikiaji wa account yako.">Enter a new password (6+ characters) to restore access to your account.</span></p>
        </div>
        <label class="text-[12px] font-bold text-white/60 mt-4 block" data-en="NEW PASSWORD *" data-sw="NENO LA SIRI JIPYA *">NEW PASSWORD *</label>
        <div class="ifield mt-1.5"><i class="fa-solid fa-lock"></i><input id="resetNewPass" type="password" class="input input-i !pr-12" placeholder="Min 6 characters" oninput="pwStrength('resetNewPass','resetMeter')"><button onclick="togglePw('resetNewPass',this)" class="absolute right-3 top-1/2 -translate-y-1/2 text-white/40 hover:text-cyan-300"><i class="fa-solid fa-eye"></i></button></div>
        <div id="resetMeter" class="pw-meter mt-2"><span></span></div>
        <label class="text-[12px] font-bold text-white/60 mt-3 block" data-en="CONFIRM NEW PASSWORD *" data-sw="THIBITISHA NENO LA SIRI JIPYA *">CONFIRM NEW PASSWORD *</label>
        <div class="ifield mt-1.5"><i class="fa-solid fa-lock"></i><input id="resetConfirmPass" type="password" class="input input-i" placeholder="Repeat new password"></div>
        <button onclick="doResetPasswordFromUrl()" class="gold-btn w-full mt-6 py-4 rounded-2xl font-bold text-[#2a1500]"><i class="fa-solid fa-unlock mr-2"></i><span data-en="Update Password & Sign In" data-sw="Sasisha Neno la Siri na Uingie">Update Password & Sign In</span></button>
        <button onclick="showLoginTabOnly()" class="w-full mt-3 text-[12px] text-white/50 hover:text-white/80"><span data-en="← Back to Sign In" data-sw="← Rudi kwenye Kuingia">← Back to Sign In</span></button>
      </div>

      <!-- ---------- CREATE ACCOUNT ---------- -->
      <div id="authRegister" class="mt-6 hidden">
        <div class="mb-5">
          <div class="font-display font-bold text-xl" data-en="Create your account" data-sw="Fungua akaunti yako">Create your account</div>
          <div class="text-white/50 text-[13px] mt-1" data-en="Takes about a minute. No payment needed to get started." data-sw="Inachukua dakika moja. Hakuna malipo ya kuanza.">Takes about a minute. No payment needed to get started.</div>
        </div>
        <div class="grid sm:grid-cols-2 gap-3">
          <div><label class="text-[12px] font-bold text-white/60" data-en="FULL NAME *" data-sw="JINA KAMILI *">FULL NAME *</label><div class="ifield mt-1.5"><i class="fa-solid fa-user"></i><input id="regName" class="input input-i" placeholder="e.g. Amina Juma"></div></div>
          <div><label class="text-[12px] font-bold text-white/60" data-en="PHONE *" data-sw="SIMU *">PHONE *</label><div class="ifield mt-1.5"><i class="fa-solid fa-phone"></i><input id="regPhone" inputmode="tel" class="input input-i" placeholder="07XX XXX XXX"></div></div>
        </div>
        <label class="text-[12px] font-bold text-white/60 mt-3 block" data-en="EMAIL *" data-sw="BARUA PEPE *">EMAIL *</label>
        <div class="ifield mt-1.5"><i class="fa-solid fa-envelope"></i><input id="regEmail" type="email" class="input input-i" placeholder="you@company.com"></div>
        <label class="text-[12px] font-bold text-white/60 mt-3 block" data-en="COMPANY / ORGANIZATION" data-sw="KAMPUNI / TAASISI">COMPANY / ORGANIZATION</label>
        <div class="ifield mt-1.5"><i class="fa-solid fa-building"></i><input id="regCompany" class="input input-i" placeholder="Optional"></div>
        <div class="grid sm:grid-cols-2 gap-3 mt-3">
          <div><label class="text-[12px] font-bold text-white/60" data-en="PASSWORD *" data-sw="NENO LA SIRI *">PASSWORD *</label><div class="ifield mt-1.5"><i class="fa-solid fa-lock"></i><input id="regPass" type="password" class="input input-i !pr-12" placeholder="Min 6 chars" oninput="pwStrength('regPass','regMeter')"><button onclick="togglePw('regPass',this)" class="absolute right-3 top-1/2 -translate-y-1/2 text-white/40 hover:text-cyan-300"><i class="fa-solid fa-eye"></i></button></div><div id="regMeter" class="pw-meter mt-2"><span></span></div></div>
          <div><label class="text-[12px] font-bold text-white/60" data-en="CONFIRM *" data-sw="THIBITISHA *">CONFIRM *</label><div class="ifield mt-1.5"><i class="fa-solid fa-lock"></i><input id="regPass2" type="password" class="input input-i" placeholder="Repeat"></div></div>
        </div>
        <div class="grid sm:grid-cols-2 gap-3 mt-3">
          <div><label class="text-[12px] font-bold text-white/60"><span data-en="ACCOUNT PIN (4–6 digits)" data-sw="PIN YA ACCOUNT (tarakimu 4–6)">ACCOUNT PIN (4–6 digits)</span></label><div class="ifield mt-1.5"><i class="fa-solid fa-key"></i><input id="regPin" inputmode="numeric" maxlength="6" class="input input-i !tracking-[0.3em]" placeholder="1234"></div></div>
          <div><label class="text-[12px] font-bold text-white/60" data-en="CONFIRM PIN" data-sw="THIBITISHA PIN">CONFIRM PIN</label><div class="ifield mt-1.5"><i class="fa-solid fa-key"></i><input id="regPin2" inputmode="numeric" maxlength="6" class="input input-i !tracking-[0.3em]" placeholder="Repeat PIN"></div></div>
        </div>
        <div class="text-[11px] text-white/40 mt-2"><i class="fa-solid fa-circle-info mr-1 text-cyan-300"></i><span data-en="Used to verify it is really you at sign-in. If left blank, we use the last 4 digits of your phone." data-sw="Inatumika kuthibitisha ni wewe kweli unapoingia. Ukiacha wazi, tunatumia tarakimu 4 za mwisho za simu yako.">Used to verify it is really you at sign-in. If left blank, we use the last 4 digits of your phone.</span></div>
        <label class="flex items-start gap-2.5 mt-4 text-[12px] text-white/60 cursor-pointer"><input type="checkbox" id="regAgree" class="mt-0.5 w-4 h-4 accent-cyan-400"><span data-en="I agree to the Terms of Service and Privacy Policy." data-sw="Nakubali Masharti ya Huduma na Sera ya Faragha.">I agree to the Terms of Service and Privacy Policy.</span></label>
        <button onclick="doRegister()" class="gold-btn w-full mt-5 py-4 rounded-2xl font-bold text-[#2a1500]"><i class="fa-solid fa-user-plus mr-2"></i><span data-en="Create Client Account" data-sw="Fungua Account ya Mteja">Create Client Account</span></button>
      </div>

      <button onclick="showView('landing')" class="w-full mt-4 text-[13px] text-white/40 hover:text-white" data-en="← Back to website" data-sw="← Rudi kwenye wavuti">← Back to website</button>
    </div>
  </div>
</div>

<?php require __DIR__ . '/partials/footer.php'; ?>

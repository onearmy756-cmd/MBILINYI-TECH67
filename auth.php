<?php
require_once __DIR__ . '/config/app.php';
$me = current_user();
if ($me) redirect($me['role'] === 'admin' ? 'admin' : 'client');
$pageTitle = 'Mbilinyi Tech Solutions — Client & Admin Portal | Sign In';
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
          <h2 class="font-display font-bold text-3xl mt-5">Client & Admin <span class="grad-text">Portal</span></h2>
          <p class="text-white/55 text-[14px] mt-2">One secure login for everything: submit requirements, receive official quotations, e-sign contracts, pay invoices & chat with support.</p>
          <div class="grid gap-2.5 mt-6 text-[13px]">
            <div class="flex items-center gap-3 glass rounded-xl px-4 py-3"><i class="fa-solid fa-file-pen text-cyan-300"></i> Requirement forms with tracking IDs</div>
            <div class="flex items-center gap-3 glass rounded-xl px-4 py-3"><i class="fa-solid fa-file-invoice-dollar text-gold"></i> Admin quotations & transparent pricing</div>
            <div class="flex items-center gap-3 glass rounded-xl px-4 py-3"><i class="fa-solid fa-file-contract text-violet-300"></i> BRELA contracts with e-signature</div>
            <div class="flex items-center gap-3 glass rounded-xl px-4 py-3"><i class="fa-solid fa-users-gear text-emerald-300"></i> Full account management (admin)</div>
          </div>
        </div>
        <div class="relative mt-6 rounded-2xl bg-cyan-500/[.06] border border-cyan-400/25 p-4 text-[12px] text-white/70">
          <div class="font-bold text-cyan-300 mb-1"><i class="fa-solid fa-wand-magic-sparkles mr-1"></i> ONE LOGIN — AUTO-DETECTED • INGIA MOJA</div>
          <span data-en="Sign in once with your email & password — the system automatically detects whether you are Admin/CEO or Client and takes you to the right portal. No separate logins." data-sw="Ingia mara moja kwa email na password yako — mfumo hutambua kiotomatiki kama wewe ni Admin/CEO au Client na kukupeleka portal yako. Hakuna login tofauti.">Sign in once with your email & password — the system automatically detects whether you are Admin/CEO or Client and takes you to the right portal. No separate logins.</span>
        </div>
      </div>
    </div>
    <div class="glass rounded-3xl p-6 md:p-8">
      <div class="grid grid-cols-2 gap-2 p-1.5 rounded-2xl bg-white/[.04] border border-white/10">
        <button id="tabLogin" onclick="switchAuthTab('login')" class="py-3 rounded-xl font-bold text-sm grad-btn">Sign In</button>
        <button id="tabRegister" onclick="switchAuthTab('register')" class="py-3 rounded-xl font-bold text-sm text-white/60">Create Account</button>
      </div>
      <div id="authLogin" class="mt-6">
        <label class="text-[12px] font-bold text-white/60">EMAIL ADDRESS</label>
        <input id="loginEmail" class="input mt-1.5" placeholder="you@company.com">
        <label class="text-[12px] font-bold text-white/60 mt-4 block">PASSWORD</label>
        <div class="relative mt-1.5"><input id="loginPass" type="password" class="input pr-12" placeholder="••••••••" onkeydown="if(event.key==='Enter')doLogin()"><button onclick="togglePw('loginPass')" class="absolute right-3 top-1/2 -translate-y-1/2 text-white/40"><i class="fa-solid fa-eye"></i></button></div>
        <button onclick="doLogin()" id="btnDoLogin" class="grad-btn w-full mt-6 py-4 rounded-2xl font-bold"><i class="fa-solid fa-right-to-bracket mr-2"></i><span data-en="Sign In Securely" data-sw="Ingia Kwa Usalama">Sign In Securely</span></button>
        <div class="flex justify-between mt-4 text-[12px]">
          <a href="javascript:void(0)" onclick="openForgotPassword()" class="text-cyan-300 hover:text-cyan-200 font-bold"><span data-en="Forgot Password?" data-sw="Umesahau Neno la Siri?">Forgot Password?</span></a>
          <a href="javascript:void(0)" onclick="openResendCodeFlow()" class="text-white/50 hover:text-white/80"><span data-en="Need new login code?" data-sw="Nahitaji code mpya?">Need new login code?</span></a>
        </div>
        <div class="text-center text-[12px] text-white/40 mt-3">Protected with encrypted sessions • BRELA-verified company</div>
      </div>
      <div id="authReset" class="mt-6 hidden">
        <div class="glass rounded-2xl p-4 border border-violet-400/30 bg-violet-500/[.04]">
          <div class="font-display font-bold text-lg text-violet-200"><i class="fa-solid fa-key mr-1"></i><span data-en="Reset Your Password" data-sw="Badili Neno la Siri">Reset Your Password</span></div>
          <p class="text-white/55 text-[12px] mt-1"><span data-en="Enter a new password (6+ characters) to restore access to your account." data-sw="Ingiza neno la siri jipya (6+ herufi) la kurejesha ufikiaji wa account yako.">Enter a new password (6+ characters) to restore access to your account.</span></p>
        </div>
        <label class="text-[12px] font-bold text-white/60 mt-4 block">NEW PASSWORD *</label>
        <input id="resetNewPass" type="password" class="input mt-1.5" placeholder="Min 6 characters">
        <label class="text-[12px] font-bold text-white/60 mt-3 block">CONFIRM NEW PASSWORD *</label>
        <input id="resetConfirmPass" type="password" class="input mt-1.5" placeholder="Repeat new password">
        <button onclick="doResetPasswordFromUrl()" class="gold-btn w-full mt-6 py-4 rounded-2xl font-bold text-[#2a1500]"><i class="fa-solid fa-unlock mr-2"></i><span data-en="Update Password & Sign In" data-sw="Sasisha Neno la Siri na Uingie">Update Password & Sign In</span></button>
        <button onclick="showLoginTabOnly()" class="w-full mt-3 text-[12px] text-white/50 hover:text-white/80"><span data-en="← Back to Sign In" data-sw="← Rudi kwenye Kuingia">← Back to Sign In</span></button>
      </div>
      <div id="authRegister" class="mt-6 hidden">
        <div class="grid sm:grid-cols-2 gap-3">
          <div><label class="text-[12px] font-bold text-white/60">FULL NAME *</label><input id="regName" class="input mt-1.5" placeholder="e.g. Amina Juma"></div>
          <div><label class="text-[12px] font-bold text-white/60">PHONE *</label><input id="regPhone" class="input mt-1.5" placeholder="07XX XXX XXX"></div>
        </div>
        <label class="text-[12px] font-bold text-white/60 mt-3 block">EMAIL *</label><input id="regEmail" class="input mt-1.5" placeholder="you@company.com">
        <label class="text-[12px] font-bold text-white/60 mt-3 block">COMPANY / ORGANIZATION</label><input id="regCompany" class="input mt-1.5" placeholder="Optional">
        <div class="grid sm:grid-cols-2 gap-3 mt-3">
          <div><label class="text-[12px] font-bold text-white/60">PASSWORD *</label><input id="regPass" type="password" class="input mt-1.5" placeholder="Min 6 chars"></div>
          <div><label class="text-[12px] font-bold text-white/60">CONFIRM *</label><input id="regPass2" type="password" class="input mt-1.5" placeholder="Repeat"></div>
        </div>
        <button onclick="doRegister()" class="gold-btn w-full mt-6 py-4 rounded-2xl font-bold text-[#2a1500]"><i class="fa-solid fa-user-plus mr-2"></i>Create Client Account</button>
      </div>
      <button onclick="showView('landing')" class="w-full mt-4 text-[13px] text-white/40 hover:text-white">← Back to website</button>
    </div>
  </div>
</div>

<?php require __DIR__ . '/partials/footer.php'; ?>

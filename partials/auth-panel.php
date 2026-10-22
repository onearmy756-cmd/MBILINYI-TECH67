<?php
/**
 * Sign-in / sign-up card. Shared by auth.php and the homepage so the two can
 * never drift apart.
 *
 * @var bool $authShowBack  Show the "back to website" link (auth.php only —
 *                          on the homepage you are already on the website).
 */
$authShowBack = $authShowBack ?? false;
?>
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
    <div id="passkeyLoginWrap" class="hidden">
      <div class="flex items-center gap-3 my-4"><div class="h-px flex-1 bg-white/10"></div><span class="text-[11px] text-white/35" data-en="OR" data-sw="AU">OR</span><div class="h-px flex-1 bg-white/10"></div></div>
      <button onclick="doPasskeyLogin()" id="btnPasskeyLogin" class="w-full py-4 rounded-2xl font-bold border border-cyan-400/40 bg-cyan-500/10 text-cyan-200 hover:bg-cyan-500/20 transition"><i class="fa-solid fa-fingerprint mr-2"></i><span data-en="Sign in with a passkey" data-sw="Ingia kwa passkey">Sign in with a passkey</span></button>
      <div class="text-[11px] text-white/40 mt-2 text-center"><span data-en="Use your fingerprint, face or screen lock — no password, no code." data-sw="Tumia alama ya kidole, uso au lock ya simu — hakuna neno la siri wala code.">Use your fingerprint, face or screen lock — no password, no code.</span></div>
    </div>
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

  <?php if ($authShowBack): ?>
  <button onclick="showView('landing')" class="w-full mt-4 text-[13px] text-white/40 hover:text-white" data-en="← Back to website" data-sw="← Rudi kwenye wavuti">← Back to website</button>
  <?php endif; ?>
</div>

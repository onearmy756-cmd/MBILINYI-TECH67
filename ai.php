<?php
require_once __DIR__ . '/config/app.php';
$pageTitle = 'AI Fine-Tuning & Chatbots — LLMs Trained on Your Data | Mbilinyi Tech Solutions';
$page = 'landing';
require __DIR__ . '/partials/head.php';
?>
<body class="antialiased">
<?php require __DIR__ . '/partials/header.php'; ?>

<!-- ============ AI & FINE-TUNING PAGE ============ -->
<div id="view-landing" class="view-section active">
  <section id="ai" class="max-w-7xl mx-auto px-4 pt-8 pb-16">
    <div class="grid lg:grid-cols-2 gap-6">
      <div class="rounded-3xl p-[1.5px] bg-gradient-to-br from-cyan-400 via-blue-600 to-violet-600">
        <div class="rounded-3xl bg-[#070D1F] p-6 md:p-8 h-full">
          <div class="flex items-center gap-3"><div class="w-11 h-11 rounded-2xl grad-btn flex items-center justify-center"><i class="fa-solid fa-brain"></i></div><div><div class="font-display font-bold text-xl">AI Fine-Tuning Studio</div><div class="text-[12px] text-white/50">Custom LLMs trained on YOUR business data</div></div></div>
          <div class="grid sm:grid-cols-2 gap-3 mt-6 text-[13px]">
            <div class="glass rounded-2xl p-4"><i class="fa-solid fa-database text-cyan-300"></i><div class="font-bold mt-2">Domain Data Prep</div><div class="text-white/55 text-[12.5px]">We clean, label & structure your docs, chats & records for training.</div></div>
            <div class="glass rounded-2xl p-4"><i class="fa-solid fa-sliders text-violet-300"></i><div class="font-bold mt-2">LoRA / Full Fine-Tune</div><div class="text-white/55 text-[12.5px]">Open-source & commercial models tuned for accuracy & tone.</div></div>
            <div class="glass rounded-2xl p-4"><i class="fa-solid fa-gauge-high text-emerald-300"></i><div class="font-bold mt-2">Eval & Guardrails</div><div class="text-white/55 text-[12.5px]">Hallucination tests, safety filters & Swahili/English evals.</div></div>
            <div class="glass rounded-2xl p-4"><i class="fa-solid fa-plug text-gold"></i><div class="font-bold mt-2">Deploy + Chatbot</div><div class="text-white/55 text-[12.5px]">API + website & WhatsApp chatbot wired to your system.</div></div>
          </div>
          <div class="mt-5 flex flex-wrap gap-2 text-[11px] font-mono text-white/60"><span class="px-2.5 py-1 rounded-full bg-white/5 border border-white/10">Llama 3</span><span class="px-2.5 py-1 rounded-full bg-white/5 border border-white/10">Mistral</span><span class="px-2.5 py-1 rounded-full bg-white/5 border border-white/10">GPT-4o</span><span class="px-2.5 py-1 rounded-full bg-white/5 border border-white/10">Gemini</span><span class="px-2.5 py-1 rounded-full bg-white/5 border border-white/10">Whisper STT</span><span class="px-2.5 py-1 rounded-full bg-white/5 border border-white/10">RAG + Vector DB</span></div>
        </div>
      </div>
      <div class="glass rounded-3xl p-6 md:p-8">
        <div class="flex items-center justify-between"><div class="font-display font-bold text-xl"><i class="fa-solid fa-flask text-cyan-300 mr-2"></i>Live Demo: Fine-Tuned Assistant</div><span class="badge bg-emerald-500/15 text-emerald-300 border border-emerald-400/30">INTERACTIVE</span></div>
        <p class="text-white/55 text-[13px] mt-2">Select a business brain, type a customer question, and see how a fine-tuned model answers vs a generic one.</p>
        <label class="text-[12px] font-bold text-white/60 mt-4 block">CHOOSE FINE-TUNED BRAIN</label>
        <div class="grid grid-cols-3 gap-2 mt-2" id="aiBrains">
          <button data-brain="pharmacy" class="brain-btn tab-active px-3 py-2.5 rounded-xl border border-white/10 text-[12px] font-bold">Pharmacy 🇹🇿</button>
          <button data-brain="school" class="brain-btn px-3 py-2.5 rounded-xl border border-white/10 text-[12px] font-bold">School Fees</button>
          <button data-brain="hotel" class="brain-btn px-3 py-2.5 rounded-xl border border-white/10 text-[12px] font-bold">Hotel Booking</button>
        </div>
        <input id="aiDemoInput" class="input mt-3 font-mono text-[13px]" placeholder='Try: "Je, mna dawa ya presha? / What are fees for Form One?"'>
        <button onclick="runAiDemo()" class="grad-btn w-full mt-3 py-3 rounded-xl font-bold text-sm"><i class="fa-solid fa-wand-magic-sparkles mr-2"></i>Generate Fine-Tuned Answer</button>
        <div id="aiDemoOut" class="mt-4 grid gap-3">
          <div class="rounded-2xl border border-white/10 bg-white/[.02] p-4 text-[13px]"><div class="text-[11px] font-bold text-white/40 uppercase tracking-wider mb-1">Generic Model ❌</div><div class="text-white/55 italic">I don't have specific information about your business. Please check your records...</div></div>
          <div class="rounded-2xl border border-cyan-400/25 bg-cyan-500/[.06] p-4 text-[13px]"><div class="text-[11px] font-bold text-cyan-300 uppercase tracking-wider mb-1">Mbilinyi Fine-Tuned ✓</div><div id="aiDemoAnswer" class="text-white/85">Your tuned answer will appear here — trained on your price lists, policies & Swahili FAQs.</div></div>
        </div>
      </div>
    </div>
  </section>
</div>

<?php require __DIR__ . '/partials/footer.php'; ?>

<?php /** @var string $pageTitle */ ?>
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title><?= htmlspecialchars($pageTitle) ?></title>
<meta name="description" content="Mbilinyi Tech Solutions — BRELA Registered, TIN 192-147-522. Custom Software, System Analysis, IT Consultancy, Maintenance, Cybersecurity & AI Fine-Tuning. CEO Jackson Mbilinyi.">
<meta name="theme-color" content="#050914">
<link rel="manifest" href="manifest.json">
<link rel="apple-touch-icon" href="data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 180 180'><defs><linearGradient id='g' x1='0' y1='0' x2='1' y2='1'><stop offset='0' stop-color='%2306b6d4'/><stop offset='0.5' stop-color='%233b82f6'/><stop offset='1' stop-color='%238b5cf6'/></linearGradient></defs><rect width='180' height='180' rx='40' fill='url(%23g)'/><text x='90' y='120' font-family='Space Grotesk, Arial, sans-serif' font-size='90' font-weight='700' text-anchor='middle' fill='white'>M</text></svg>">
<meta name="apple-mobile-web-app-capable" content="yes">
<meta name="apple-mobile-web-app-status-bar-style" content="black-translucent">
<meta name="apple-mobile-web-app-title" content="Mbilinyi Tech">
<script src="https://cdn.tailwindcss.com"></script>
<script>
tailwind.config = {
  theme: {
    extend: {
      colors: {
        ink: '#050914',
        ink2: '#0A1226',
        card: '#0E1933',
        line: 'rgba(255,255,255,0.08)',
        cyanx: '#22d3ee',
        bluex: '#3b82f6',
        violetx: '#8b5cf6',
        gold: '#F5B301',
      },
      fontFamily: {
        display: ['Space Grotesk','sans-serif'],
        body: ['Inter','sans-serif'],
        mono: ['JetBrains Mono','monospace'],
      }
    }
  }
}
</script>
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=Space+Grotesk:wght@500;600;700&family=JetBrains+Mono:wght@400;600&display=swap" rel="stylesheet">
<link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.2/css/all.min.css">
<script src="https://cdn.jsdelivr.net/npm/chart.js@4.4.1/dist/chart.umd.min.js"></script>
<script src="https://cdnjs.cloudflare.com/ajax/libs/gsap/3.12.5/gsap.min.js"></script>
<script src="https://cdnjs.cloudflare.com/ajax/libs/gsap/3.12.5/ScrollTrigger.min.js"></script>
<style>
  *{scroll-behavior:smooth}
  body{background:#050914;color:#E6ECF5;font-family:Inter,sans-serif;overflow-x:hidden}
  ::-webkit-scrollbar{width:10px;height:10px}
  ::-webkit-scrollbar-track{background:#070D1F}
  ::-webkit-scrollbar-thumb{background:linear-gradient(#22d3ee,#8b5cf6);border-radius:99px}
  .glass{background:rgba(255,255,255,.04);backdrop-filter:blur(18px);-webkit-backdrop-filter:blur(18px);border:1px solid rgba(255,255,255,.08)}
  .glass-strong{background:rgba(14,25,51,.85);backdrop-filter:blur(22px);border:1px solid rgba(255,255,255,.1)}
  .grad-text{background:linear-gradient(90deg,#22d3ee,#60a5fa,#a78bfa,#f0abfc);-webkit-background-clip:text;background-clip:text;color:transparent}
  .grad-btn{background:linear-gradient(135deg,#06b6d4,#3b82f6 50%,#8b5cf6);box-shadow:0 10px 30px -8px rgba(59,130,246,.6)}
  .grad-btn:hover{transform:translateY(-2px);box-shadow:0 18px 40px -8px rgba(59,130,246,.7)}
  .gold-btn{background:linear-gradient(135deg,#F5B301,#ff7847);box-shadow:0 10px 30px -8px rgba(245,179,1,.5)}
  .grid-bg{background-image:linear-gradient(rgba(255,255,255,.04) 1px,transparent 1px),linear-gradient(90deg,rgba(255,255,255,.04) 1px,transparent 1px);background-size:44px 44px;mask-image:radial-gradient(ellipse 90% 70% at 50% 0%,black 60%,transparent 100%)}
  .orb{position:absolute;border-radius:50%;filter:blur(90px);opacity:.5;pointer-events:none}
  .marquee-track{display:flex;gap:14px;width:max-content;animation:marquee 28s linear infinite}
  @keyframes marquee{to{transform:translateX(-50%)}}
  @keyframes float{0%,100%{transform:translateY(0)}50%{transform:translateY(-14px)}}
  .float{animation:float 5s ease-in-out infinite}
  .float2{animation:float 7s ease-in-out infinite}
  .code-dots span{width:11px;height:11px;border-radius:50%;display:inline-block}
  .typing span{width:7px;height:7px;background:#22d3ee;border-radius:50%;display:inline-block;animation:blink 1s infinite}
  .typing span:nth-child(2){animation-delay:.2s}.typing span:nth-child(3){animation-delay:.4s}
  @keyframes blink{0%,100%{opacity:.2;transform:translateY(0)}50%{opacity:1;transform:translateY(-4px)}}
  .timeline-line{background:linear-gradient(180deg,#22d3ee,#8b5cf6,#22ff88)}
  .step-dot{box-shadow:0 0 0 6px rgba(34,211,238,.15)}
  .tab-active{background:linear-gradient(135deg,#06b6d4,#8b5cf6)!important;color:white!important;border-color:transparent!important}
  .sidebar-link.active{background:linear-gradient(135deg,rgba(6,182,212,.2),rgba(139,92,246,.2));border-left:3px solid #22d3ee;color:white}
  .input{background:rgba(255,255,255,.05);border:1px solid rgba(255,255,255,.1);border-radius:12px;padding:12px 14px;width:100%;color:white;outline:none;transition:.2s}
  .input:focus{border-color:#22d3ee;box-shadow:0 0 0 3px rgba(34,211,238,.15);background:rgba(255,255,255,.07)}
  .input::placeholder{color:#7C8AA5}
  select.input option{background:#0E1933}
  .badge{font-size:10px;letter-spacing:.12em;text-transform:uppercase;font-weight:700;padding:5px 10px;border-radius:99px}
  .status-Pending{background:rgba(245,179,1,.15);color:#F5B301;border:1px solid rgba(245,179,1,.3)}
  .status-UnderReview{background:rgba(59,130,246,.15);color:#60a5fa;border:1px solid rgba(59,130,246,.3)}
  .status-Quoted{background:rgba(139,92,246,.15);color:#a78bfa;border:1px solid rgba(139,92,246,.35)}
  .status-Approved{background:rgba(34,211,238,.15);color:#22d3ee;border:1px solid rgba(34,211,238,.35)}
  .status-InProgress{background:rgba(34,197,94,.15);color:#4ade80;border:1px solid rgba(34,197,94,.3)}
  .status-Completed{background:rgba(16,185,129,.2);color:#6ee7b7;border:1px solid rgba(16,185,129,.4)}
  .status-Rejected{background:rgba(239,68,68,.15);color:#fca5a5;border:1px solid rgba(239,68,68,.35)}
  .chat-bubble-ai{background:rgba(255,255,255,.06);border:1px solid rgba(255,255,255,.08);border-radius:4px 16px 16px 16px}
  .chat-bubble-me{background:linear-gradient(135deg,#0891b2,#6366f1);border-radius:16px 4px 16px 16px}
  .sig-pad{cursor:crosshair;touch-action:none}
  table.data th{font-size:11px;text-transform:uppercase;letter-spacing:.1em;color:#8EA0BF;text-align:left;padding:12px;font-weight:700;background:rgba(255,255,255,.03)}
  table.data td{padding:12px;border-top:1px solid rgba(255,255,255,.06);font-size:13.5px;vertical-align:middle}
  table.data tr:hover td{background:rgba(255,255,255,.02)}
  .wizard-step{display:none}.wizard-step.active{display:block;animation:fadeUp .4s ease}
  @keyframes fadeUp{from{opacity:0;transform:translateY(14px)}to{opacity:1;transform:translateY(0)}}
  .view-section{display:none}.view-section.active{display:block;animation:fadeUp .4s ease}
  .tech-card:hover{transform:translateY(-6px) scale(1.02)}
  .service-card{transition:.35s}.service-card:hover{transform:translateY(-8px);border-color:rgba(34,211,238,.4);box-shadow:0 24px 60px -18px rgba(34,211,238,.35)}
  #chatWindow{transition:.35s cubic-bezier(.2,.9,.3,1.2);transform-origin:bottom right}
  #chatWindow.hidden-chat{transform:scale(.6) translateY(20px);opacity:0;pointer-events:none}
  .toast-in{animation:toastIn .35s cubic-bezier(.2,.9,.3,1.2)}
  @keyframes toastIn{from{transform:translateX(100%);opacity:0}to{transform:translateX(0);opacity:1}}
  .print-only{display:none}
  @media print{
    body *{visibility:hidden}
    #printArea, #printArea *{visibility:visible}
    #printArea{position:absolute;left:0;top:0;width:100%;background:white;color:black;padding:24px}
    .print-only{display:block}
    .no-print{display:none!important}
  }
  .orbit-ring{border:1px dashed rgba(34,211,238,.25);border-radius:50%;position:absolute;animation:spin 24s linear infinite}
  @keyframes spin{to{transform:rotate(360deg)}}
  .shimmer{background:linear-gradient(110deg,transparent 30%,rgba(255,255,255,.12) 50%,transparent 70%);background-size:200% 100%;animation:shimmer 2.2s infinite}
  @keyframes shimmer{to{background-position:-200% 0}}
</style>
</head>

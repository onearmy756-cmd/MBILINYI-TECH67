/* Live calls inside the platform.
 *
 * Joins the signalling room at /ws/call/<room>, opens the camera and microphone,
 * and connects to each other participant with a peer connection. The media goes
 * straight between browsers; only the handshake passes through the server.
 *
 * The page provides: a container with [data-call-room] and [data-call-name],
 * and buttons with [data-call-action="mute|camera|share|hangup"].
 */
(function () {
  'use strict';

  var root = document.querySelector('[data-call-room]');
  if (!root) return;

  var room = root.getAttribute('data-call-room');
  var me = root.getAttribute('data-call-name') || 'guest';
  var myLabel = null;        // set from the server's `joined` event
  var myName = me;           // the display name, for tile captions
  var scheme = location.protocol === 'https:' ? 'wss' : 'ws';
  var socket;
  var localStream;
  var peers = {};            // peer label -> { pc, polite, name }
  var makingOffer = {};
  var makingOfferState = {};

  // Where to find the other side. A public STUN server gets two browsers behind
  // home routers talking. It cannot get through the symmetric NAT a mobile
  // carrier uses, so a TURN relay is needed for calls on a phone network — the
  // CEO sets those in Admin -> Settings and they arrive in window.__ICE__.
  // [data-call-ice] on the element wins, so a one-off room can override.
  var iceServers = [{ urls: 'stun:stun.l.google.com:19302' }];
  var fromPlatform = window.__ICE__;
  if (Object.prototype.toString.call(fromPlatform) === '[object Array]' && fromPlatform.length) {
    iceServers = fromPlatform;
  }
  try {
    var extra = JSON.parse(root.getAttribute('data-call-ice') || '[]');
    if (extra.length) iceServers = extra;
  } catch (e) {}
  var hasTurn = iceServers.some(function (s) {
    var u = typeof s === 'string' ? s : (s && s.urls) || '';
    return String(u).indexOf('turn') === 0 || String(u).indexOf('turns') === 0;
  });

  var grid = root.querySelector('[data-call-videos]') || root;
  var status = root.querySelector('[data-call-status]');

  function say(msg) {
    if (status) status.textContent = msg;
  }

  /* A label looks like "Asha#3f2a1b". Show only the name. */
  function nameOf(label) {
    var i = String(label).lastIndexOf('#');
    return i > 0 ? String(label).slice(0, i) : String(label);
  }

  function tile(key, stream, isSelf, caption) {
    var box = document.createElement('div');
    box.className = 'call-tile';
    box.setAttribute('data-peer', key);
    var video = document.createElement('video');
    video.autoplay = true;
    video.playsInline = true;
    video.muted = !!isSelf;
    video.srcObject = stream;
    var label = document.createElement('span');
    label.className = 'call-name';
    label.textContent = (caption || nameOf(key)) + (isSelf ? ' (you)' : '');
    box.appendChild(video);
    box.appendChild(label);
    grid.appendChild(box);
    return box;
  }

  function removeTile(name) {
    var box = grid.querySelector('[data-peer="' + name.replace(/"/g, '') + '"]');
    if (box) box.remove();
  }

  function send(type, payload) {
    if (socket && socket.readyState === 1) {
      socket.send(JSON.stringify({ type: type, payload: payload }));
    }
  }

  /* Set up the connection to one other participant.

     `initiate` says whether we are the side that sends the first offer. The
     person who joined later initiates; the people already in the room wait for
     that offer and answer it. Both sides offering at once is glare: each one
     drops the other's offer as a collision and the call connects to nobody,
     which is exactly what a two-person room did before this rule. */
  function connectionFor(name, initiate) {
    if (peers[name]) return peers[name];
    var pc = new RTCPeerConnection({ iceServers: iceServers });
    peers[name] = { pc: pc, initiate: !!initiate };
    makingOffer[name] = false;
    makingOfferState[name] = false;

    localStream.getTracks().forEach(function (t) {
      pc.addTrack(t, localStream);
    });

    pc.oniceconnectionstatechange = function () {
      var st = pc.iceConnectionState;
      if (st === 'failed') {
        say(hasTurn
          ? 'Imeshindwa kuunganisha na ' + nameOf(name) + '. Jaribu tena.'
          : 'Imeshindwa kuunganisha na ' + nameOf(name) + '. Mtandao huu unahitaji TURN serveri — mwambie CEO aiongeze kwenye Settings.');
      } else if (st === 'connected' || st === 'completed') {
        say('Umeunganishwa.');
      }
    };

    pc.onicecandidate = function (e) {
      if (e.candidate) send('ice', { to: name, candidate: e.candidate });
    };
    pc.ontrack = function (e) {
      var box = grid.querySelector('[data-peer="' + name.replace(/"/g, '') + '"]');
      if (box) {
        box.querySelector('video').srcObject = e.streams[0];
      } else {
        tile(name, e.streams[0], false, nameOf(name));
      }
    };
    pc.onnegotiationneeded = function () {
      // Adding our tracks fires this on both sides. Only the initiating side
      // turns it into an offer; the other side waits for the offer to arrive.
      if (!peers[name] || !peers[name].initiate) return;
      makingOfferState[name] = true;
      pc.createOffer()
        .then(function (o) { return pc.setLocalDescription(o); })
        .then(function () { send('offer', { to: name, sdp: pc.localDescription }); })
        .catch(function () {})
        .then(function () { makingOfferState[name] = false; });
    };
    return peers[name];
  }

  function handleOffer(name, sdp) {
    // Whoever sent the offer is the initiating side; we only ever answer.
    var entry = connectionFor(name, false);
    var pc = entry.pc;
    // Perfect negotiation: a collision resolves without either side stalling.
    var collision = makingOfferState[name] || pc.signalingState !== 'stable';
    if (collision) return; // the other side will answer ours instead
    pc.setRemoteDescription(new RTCSessionDescription(sdp))
      .then(function () { return pc.createAnswer(); })
      .then(function (a) { return pc.setLocalDescription(a); })
      .then(function () { send('answer', { to: name, sdp: pc.localDescription }); })
      .catch(function () {});
  }

  function handleAnswer(name, sdp) {
    var entry = peers[name];
    if (!entry) return;
    entry.pc.setRemoteDescription(new RTCSessionDescription(sdp)).catch(function () {});
  }

  function handleIce(name, candidate) {
    var entry = peers[name];
    if (!entry || !candidate) return;
    entry.pc.addIceCandidate(new RTCIceCandidate(candidate)).catch(function () {});
  }

  /* A payload is an object when it was relayed, and a JSON string when the
     server wrote the event itself. Accept both. */
  function payloadObject(msg) {
    var p = msg && msg.payload;
    if (typeof p === 'string') { try { return JSON.parse(p) || {}; } catch (e) { return {}; } }
    return p && typeof p === 'object' ? p : {};
  }

  function onMessage(text) {
    var msg;
    try { msg = JSON.parse(text); } catch (e) { return; }
    var from = msg.from;
    if (msg.type === 'joined') {
      var info = payloadObject(msg);
      if (info.you && info.you.label) myLabel = info.you.label;
      if (info.you && info.you.name) myName = info.you.name;
      var already = info.peers || [];
      say(already.length
        ? 'Umeunganishwa. Wapo ' + already.length + ' kwenye chumba.'
        : 'Umeunganishwa kwenye chumba. Wengine wanaweza kujiunga sasa.');
      // The newcomer offers to the people already here; they answer. This is
      // what stops both sides offering at once.
      already.forEach(function (p) { connectionFor(p, true); });
      return;
    }
    if (msg.type === 'peer-joined' && myLabel && from !== myLabel) {
      say(nameOf(from) + ' amejiunga.');
      // The newcomer waits; the people already here greet by offering.
      connectionFor(from, false);
      return;
    }
    if (msg.type === 'peer-left' && from !== myLabel) {
      say(nameOf(from) + ' ameondoka.');
      if (peers[from]) { peers[from].pc.close(); delete peers[from]; }
      removeTile(from);
      return;
    }
    if (from === myLabel) return;
    // Ignore a signal addressed to another participant.
    var to = payloadObject(msg).to;
    if (to && myLabel && to !== myLabel) return;
    if (msg.type === 'offer') return handleOffer(from, msg.payload.sdp);
    if (msg.type === 'answer') return handleAnswer(from, msg.payload.sdp);
    if (msg.type === 'ice') return handleIce(from, msg.payload.candidate);
  }

var left = false;

function open() {
    if (left) return;
    if (!hasTurn) {
      say('Chumba kimefunguliwa. Kumbuka: hakuna TURN serveri iliyowekwa, hivyo simu inaweza kushindwa kwenye mtandao wa simu.');
    }
    socket = new WebSocket(scheme + '://' + location.host + '/ws/call/' + encodeURIComponent(room) + '?name=' + encodeURIComponent(me));
    socket.onmessage = function (e) { onMessage(e.data); };
    socket.onclose = function () {
      if (left) return;
      say('Mawasiliano yamekatika. Inajaribu kuunganisha tena…');
      setTimeout(open, 2000);
    };
    socket.onerror = function () { say('Imeshindwa kuunganisha. Angalia mtandao.'); };
  }

  function wireButtons() {
    root.querySelectorAll('[data-call-action]').forEach(function (b) {
      b.addEventListener('click', function () {
        var a = b.getAttribute('data-call-action');
        if (a === 'mute') {
          localStream.getAudioTracks().forEach(function (t) { t.enabled = !t.enabled; });
          b.classList.toggle('off');
          send('mute', { on: !localStream.getAudioTracks()[0].enabled });
        } else if (a === 'camera') {
          localStream.getVideoTracks().forEach(function (t) { t.enabled = !t.enabled; });
          b.classList.toggle('off');
          send('camera', { on: !localStream.getVideoTracks()[0].enabled });
        } else if (a === 'share') {
          share(b);
        } else if (a === 'hangup') {
          if (left) return;
          // Tell the room first, while the socket is still open.
          send('leave', {});
          left = true;
          Object.keys(peers).forEach(function (n) { peers[n].pc.close(); });
          peers = {};
          if (localStream) localStream.getTracks().forEach(function (t) { t.stop(); });
          if (socket) socket.close();
          say('Umeondoka kwenye chumba.');
        }
      });
    });
  }

  function share(button) {
    if (!navigator.mediaDevices.getDisplayMedia) return;
    navigator.mediaDevices.getDisplayMedia({ video: true }).then(function (display) {
      var track = display.getVideoTracks()[0];
      Object.keys(peers).forEach(function (n) {
        var sender = peers[n].pc.getSenders().find(function (s) { return s.track && s.track.kind === 'video'; });
        if (sender) sender.replaceTrack(track);
      });
      send('share', { on: true });
      track.onended = function () {
        var cam = localStream.getVideoTracks()[0];
        Object.keys(peers).forEach(function (n) {
          var sender = peers[n].pc.getSenders().find(function (s) { return s.track && s.track.kind === 'video'; });
          if (sender) sender.replaceTrack(cam);
        });
        send('share', { on: false });
        button.classList.remove('off');
      };
      button.classList.add('off');
    }).catch(function () {});
  }

  navigator.mediaDevices.getUserMedia({ video: true, audio: true }).then(function (stream) {
    localStream = stream;
    tile('self', stream, true, myName);
    wireButtons();
    open();
  }).catch(function () {
    // No camera: still allow voice, which is what a client on a phone wants.
    navigator.mediaDevices.getUserMedia({ audio: true }).then(function (stream) {
      localStream = stream;
      say('Kamera haipatikani; sauti pekee inatumika.');
      wireButtons();
      open();
    }).catch(function () {
      say('Kamera na maikrofoni hazipatikani. Ruhusu kwenye kivinjari kisha jaribu tena.');
    });
  });
})();

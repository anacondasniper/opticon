const statusEl = document.getElementById("status");
const localVideo = document.getElementById("local");
const remoteVideo = document.getElementById("remote");
const log = (m) => { statusEl.textContent = m; console.log(m); };

const myId = Math.random().toString(36).slice(2, 8);

const pc = new RTCPeerConnection({
	iceServers: [{ urls: "stun:stun.l.google.com:19302" }],
});

const pendingCandidates = [];
let ws;
let started = false;

pc.ontrack = (e) => { remoteVideo.srcObject = e.streams[0]; log("remote video arrived \u{1F389}"); };

pc.onicecandidate = (e) => { if (e.candidate) send({ type: "ice", candidate: e.candidate }); };

function send(obj) { ws.send(JSON.stringify({ ...obj, from: myId })); }

async function flushCandidates() {
	while(pendingCandidates.length) await pc.addIceCandidate(pendingCandidates.shift());
}

async function maybeStartCall(otherId) {
	if (started) return;
	started = true;
	if (myId > otherId) {
		await pc.setLocalDescription(await pc.createOffer());
		send({ type: "offer", sdp: pc.localDescription });
		log("I'm the caller \u2192 sent offer.");
	} else {
		log("waiting for an offer\u2026");
	}
}

function setupSignaling() {
	const proto = location.protocol === "https:" ? "wss:" : "ws:";
	ws = new WebSocket(`${proto}//${location.host}/ws`);
	ws.onopen = () => { log(`connected as ${myId}`); send({ type: "ready"}); };
	ws.onmessage = async (e) => {
		const msg = JSON.parse(e.data);
		if (msg.from == myId) return;

		switch (msg.type) {
			case "ready":
				send({ type: "ready-ack" });
				maybeStartCall(msg.from);
				break;
			case "ready-ack":
				maybeStartCall(msg.from);
				break;
			case "offer":
				await pc.setRemoteDescription(msg.sdp);
				await flushCandidates();
				await pc.setLocalDescription(await pc.createAnswer());
				send({ type: "answer", sdp: pc.localDescription });
				log("got offer \u2192 sent answer");
				break;
			case "answer":
				await pc.setRemoteDescription(msg.sdp);
				await flushCandidates();
				log("got offer \u2192 connected");
				break;
			case "ice":
				if (msg.candidate) {
					if (pc.remoteDescription) await pc.addIceCandidate(msg.candidate);
					else pendingCandidates.push(msg.candidate);
				}
				break;
		}
	};
}

async function main() {
	const stream = await navigator.mediaDevices.getUserMedia({ video: true, audio: false });
	localVideo.srcObject = stream;
	stream.getTracks().forEach((t) => pc.addTrack(t, stream));
	log("camera ready \u2014 open this page in a second tab");
	setupSignaling();
}

main().catch((err) => log("error: " + err.message));

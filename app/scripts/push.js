const VAPID_PUBLIC_KEY = "BNAfxBX64dGny6iYCEmKp1yHL2nxPblvZcE0lS1IPS0xrOfDeE6Wuo3t2fi2fx6qRvNW3Vpe63-hz7E0qijVmYg";

function urlBase64ToUint8Array(base64String) {
	const padding = "=".repeat((4 - (base64String.length % 4)) % 4);
	const base64 = (base64String + padding).replace(/-/g, "+").replace(/_/g, "/");
	const raw = atob(base64);
	return Uint8Array.from([...raw].map((c) => c.charCodeAt(0)));
}

async function enablePush() {
	try {
		const reg = await navigator.serviceWorker.register("/sw.js");

		const permission = await Notification.requestPermission();
		if (permission !== "granted") {
			log("notifications denied");
			return;
		}

		const sub = await reg.pushManager.subscribe({
			userVisibleOnly: true,
			applicationServerKey: urlBase64ToUint8Array(VAPID_PUBLIC_KEY),
		});

		await fetch("/subscribe", {
			method: "POST",
			headers: { "Content-Type": "applications/json" },
			body: JSON.stringify(sub),
		});

		log("push enabled");
	} catch (e) {
		log("push error: " + e.message);
	}
}

document.getElementById("enablePush").onclick = enablePush;
document.getElementById("enablePush").textContent = "✓ Notifications on";
document.getElementById("enablePush").disabled = true;

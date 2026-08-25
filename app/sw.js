self.addEventListener("push", (event) => {
	const data = event.data ? event.data.json() : {};
	const title = data.title || "Opticon";
	const body = data.body || "Someone's at the door";

	event.waitUntil(
		self.registration.showNotification(title, {
			body, 
			tag: data.kind || "ring",
		})
	);
});

self.addEventListener("notificationclick", (event) => {
	event.notification.close();
	event.waitUntil(clients.openWindow("/"));
})

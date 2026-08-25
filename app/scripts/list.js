async function loadDoorbells() {
	const container = document.querySelector(".doorbell-cards");
	container.innerHTML = "";

	try {
		const res = await fetch("/doorbells");
		const doorbells = await res.json();

		for (const db of doorbells) {
			const card = document.createElement("button");
			card.className = "doorbell-card";
			card.dataset.id = db.id;
			card.innerHTML = `
				<div class="doorbell-icon">🔔</div>
				<div class="doorbell-info">
					<div class="doorbell-name">${db.name}</div>
					<div class="doorbell-status">Online</div>
				</div>
				<div class="doorbell-chevron">›</div>
			`;
			card.addEventListener("click", () => openDoorbell(db.id, db.name));
			container.appendChild(card);
		}
	} catch (e) {
		container.innerHTML = `<p style="color:var(--text-dim)">Couldn't load doorbells</p>`;
	}
}

function openDoorbell(id, name) {
	document.getElementById("list").style.display = "none";
	document.getElementById("app").style.display = "block";
	main();
}

const loginError = document.getElementById("loginError");

document.getElementById("loginForm").addEventListener("submit", async (e) => {
	e.preventDefault();
	const username = document.getElementById("username").value;
	const password = document.getElementById("password").value;

	try {
		const res = await fetch("/login", {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ username, password }),
		});

		if (res.ok) {
			document.getElementById("login").style.display = "none";
			document.getElementById("list").style.display = "block";
			loadDoorbells();
		} else {
			loginError.textContent = "Invalid username or password";
		}
	} catch (e) {
		loginError.textContent = "Login failed: " + e.message;
	}
});

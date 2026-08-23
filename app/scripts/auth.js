const loginBtn = document.getElementById("loginBtn");
const loginError = document.getElementById("loginError");

loginBtn.onclick = async () => {
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
			document.getElementById("app").style.display = "block";
			main();
		} else {
			loginError.textContent = "Invalid username or password";
		}
	} catch (e) {
		loginError.textContent = "Login failed: " + e.message;
	}
};

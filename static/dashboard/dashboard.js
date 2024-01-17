class Dashboard {
	constructor() {
		Auth.request("/api/users/me").then(response => {
			let user = JSON.parse(response);
			document.getElementById("test").innerText = "Logged in as " + user.display_name;
			console.log(user);
		});
	}
}

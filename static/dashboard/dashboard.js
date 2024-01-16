class Dashboard {
	constructor() {
		Auth.refresh_token();
		document.getElementById("test").innerText = "Logged in as " + Auth.get_cookie("username");
	}
}

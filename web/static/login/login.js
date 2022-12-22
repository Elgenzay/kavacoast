class LoginPage {
	constructor() {
		let login = sessionStorage.getItem("login");
		if (login) {
			try {
				console.log("has login");
				let d = JSON.parse(login);
				let data = {
					"username": d.u,
					"password": atob(d.p)
				};
				LoginPage.post('/api/auth', JSON.stringify(data)).then(function (e) {
					if (LoginPage.process_login_response(e)) {
						window.location.href = "/dashboard";
					} else {
						LoginPage.error(e, false);
					}
				}, function (e) {
					LoginPage.error(e, false);
				});
			} catch (e) {
				LoginPage.error(e, false);
			}
		} else {
			document.getElementById("content").style.visibility = "visible";
		}

		document.getElementById("form").addEventListener("submit", function (event) {
			event.preventDefault();
			let data = {
				"username": document.getElementById("username").value,
				"password": document.getElementById("password").value
			};
			LoginPage.post('/api/auth', JSON.stringify(data)).then(function (e) {
				if (LoginPage.process_login_response(e)) {
					sessionStorage.setItem("login", JSON.stringify({
						"u": data.username,
						"p": btoa(data.password)
					}));
					window.location.href = "/dashboard";
				}
			}, function (e) {
				LoginPage.error("Unexpected network error.");
			});

		});
	}

	static process_login_response(e) {
		if (e.target.status < 200 || e.target.status >= 300) {
			try {
				let response = JSON.parse(e.target.responseText);
				if ("error" in response) {
					LoginPage.error(response["error"]);
				} else {
					LoginPage.error("Unexpected response error");
				}
			} catch (f) {
				LoginPage.error("Unexpected error: " + e.target.status);
			}
		} else {
			console.log("success");
			let response_obj = JSON.parse(e.target.responseText);
			console.log(response_obj);
			if (response_obj["error"]) {
				LoginPage.error(response_obj["error"]);
			} else if (e.target.status == 200) {
				return true;
			} else {
				LoginPage.error("Unexpected error");
			}
		}
		return false;
	}

	static error(desc, display = true) {
		document.getElementById("content").style.visibility = "visible";
		console.error(desc);
		sessionStorage.clear();
		if (!display) {
			return;
		}
		document.getElementById("username").style.border = "2px solid red";
		document.getElementById("password").style.border = "2px solid red";
		document.getElementById("error").innerText = desc;
	}

	static post(url, body) {
		return new Promise(function (resolve, reject) {
			var xhr = new XMLHttpRequest();
			xhr.open("POST", url);
			xhr.setRequestHeader('Content-type', 'application/json');
			xhr.onload = resolve;
			xhr.onerror = reject;
			xhr.send(body);
		});
	}
}

class Dashboard {

	static HOME_PAGE = "settings";

	static PAGES = {
		"loading": {
			"function": "open_loading",
			"no_url": true,
		},
		"error": {
			"elem": "page-error",
			"function": "open_error",
			"no_url": true,
		},
		"admin": {
			"label": "Administration",
			"endpoint": "/api/page/admin",
			"function": "open_admin"
		},
		"settings": {
			"label": "Settings",
			"endpoint": "/api/page/settings",
			"function": "open_settings"
		}
	};

	constructor() {
		this.pages = document.getElementsByClassName("page");
		this.open_loading();

		Auth.request("/api/page/dashboard").then(r => {
			let response = JSON.parse(r);

			for (let elem of document.getElementsByClassName("displayname")) {
				elem.innerText = response.display_name;
			}

			for (let k in Dashboard.PAGES) {
				let page_obj = Dashboard.PAGES[k];

				if (!page_obj.label) {
					continue;
				}

				if (k == "admin" && !response.is_admin) {
					continue;
				}

				let nav_elem = document.createElement("div");
				nav_elem.classList.add("nav-button");
				nav_elem.setAttribute("data-page", k);
				nav_elem.setAttribute("onclick", "dashboard.open_page('" + k + "')");
				nav_elem.innerText = page_obj.label;

				for (let list_elem of document.getElementsByClassName("nav-button-list")) {
					list_elem.appendChild(nav_elem.cloneNode(true));
				}
			}

			let page = new URLSearchParams(window.location.search).get("p");

			this.open_page(page ? page : Dashboard.HOME_PAGE);
		}).catch(e => {
			console.error(e);
			this.open_error();
		});
	}

	show_page(page) {
		for (let page_elem of this.pages) {
			if (page_elem.getAttribute("data-page") === page) {
				page_elem.classList.remove("hidden");
			} else {
				page_elem.classList.add("hidden");
			}
		}
	}

	set_page_url(page) {
		if (history.pushState) {
			let newurl = new URL(window.location.href);
			newurl.searchParams.set('p', page);
			window.history.pushState({ path: newurl.href }, '', newurl.href);
		}
	}

	open_loading() {
		this.show_page("loading");
	}

	open_error() {
		this.show_page("error");
	}

	open_settings(data) {
		this.current_username = data.username;
		this.current_displayname = data.display_name;

		this.update_username_button = document.getElementById("settings-change-username-button");
		this.update_displayname_button = document.getElementById("settings-change-displayname-button");
		this.update_password_button = document.getElementById("settings-change-password-button");

		this.update_username_error = document.getElementById("settings-change-username-error");
		this.update_displayname_error = document.getElementById("settings-change-displayname-error");
		this.update_password_error = document.getElementById("settings-change-password-error");

		document.getElementById("settings-change-displayname-input").value = data.display_name;
		document.getElementById("settings-change-username-input").value = data.username;

		for (let elem of document.getElementsByClassName("discord-username")) {
			elem.innerText = data.discord_username;
		}

		this.show_page("settings");
	}

	open_admin() {
		this.show_page("admin");
	}

	open_page(page) {
		if (this.current_page === page) {
			return;
		}

		let page_obj = Dashboard.PAGES[page];

		if (!page_obj) {
			this.open_page(Dashboard.HOME_PAGE);
			return;
		}

		this.current_page = page;

		let data;
		let endpoint = page_obj.endpoint;

		if (!page_obj.no_url) {
			this.set_page_url(page);
		}

		for (let elem of document.getElementsByClassName("nav-button")) {
			if (elem.getAttribute("data-page") === page) {
				elem.classList.add("selected");
			} else {
				elem.classList.remove("selected");
			}
		}

		if (endpoint) {
			this.open_loading();

			Auth.request(endpoint).then(response => {
				try {
					data = JSON.parse(response);
					this[page_obj.function](data);
				} catch (e) {
					console.error(e);
					this.open_error();
					return;
				}
			}).catch(e => {
				try {
					if (JSON.parse(e.message).error === "Forbidden") {
						this.open_page(Dashboard.HOME_PAGE);
						return;
					}
				} catch (e) {
					console.error(e);
				}
				console.error(e);
				this.open_error();
				return;
			});
		} else {
			this[page_obj.function]();
		}
	}

	settings_username_input(elem) {
		this.settings_new_username = elem.value;

		elem.value = Auth.format_username(elem.value);

		if (this.settings_new_username !== this.current_username || this.settings_new_username === "") {
			this.update_username_button.disabled = false;
		} else {
			this.update_username_button.disabled = true;
		}
	}

	settings_displayname_input(elem) {
		this.settings_new_displayname = elem.value;

		if (this.settings_new_displayname !== this.current_displayname || this.settings_new_displayname === "") {
			this.update_displayname_button.disabled = false;
		} else {
			this.update_displayname_button.disabled = true;
		}
	}

	settings_current_password_input(elem) {
		this.settings_current_password = elem.value;
		this.settings_toggle_password_update_button();
	}

	settings_new_password_input(elem) {
		this.settings_new_password = elem.value;
		this.settings_toggle_password_update_button();
	}

	settings_confirm_password_input(elem) {
		this.settings_confirm_new_password = elem.value;
		this.settings_toggle_password_update_button();
	}

	settings_toggle_password_update_button() {
		if (!this.settings_current_password || !this.settings_new_password || this.settings_new_password !== this.settings_confirm_new_password) {
			this.update_password_button.disabled = true;
		} else {
			this.update_password_button.disabled = false;
		}
	}

	settings_password_update() {
		this.update_password_button.disabled = true;

		Auth.request("/api/users/me/change_password", {
			"old_password": this.settings_current_password,
			"new_password": this.settings_new_password
		}, "POST").then(r => {
			try {
				let response = JSON.parse(r);
				if (response.success) {
					window.location.reload();
				}
			} catch (e) {
				this.display_error(e, this.update_password_error);
				this.update_password_button.disabled = false;
			}
		}).catch(e => {
			this.display_error(e, this.update_password_error);
			this.update_password_button.disabled = false;
		});
	}

	settings_username_update() {
		this.update_username_button.disabled = true;

		Auth.request("/api/users/me", {
			"username": this.settings_new_username
		}, "PATCH").then(r => {
			try {
				let response = JSON.parse(r);
				if (response.success) {
					window.location.reload();
				}
			} catch (e) {
				this.display_error(e, this.update_username_error);
				this.update_username_button.disabled = false;
			}
		}).catch(e => {
			this.display_error(e, this.update_username_error);
			this.update_username_button.disabled = false;
		});
	}

	settings_displayname_update() {
		this.update_displayname_button.disabled = true;

		Auth.request("/api/users/me", {
			"display_name": this.settings_new_displayname
		}, "PATCH").then(r => {
			try {
				let response = JSON.parse(r);
				if (response.success) {
					window.location.reload();
				}
			} catch (e) {
				this.display_error(e, this.update_displayname_error);
				this.update_displayname_button.disabled = false;
			}
		}).catch(e => {
			this.display_error(e, this.update_displayname_error);
			this.update_displayname_button.disabled = false;
		});
	}



	display_error(error, elem) {
		console.error(error);
		try {
			let err_msg = JSON.parse(error.message).error;
			elem.innerText = err_msg;
		} catch {
			elem.innerText = "Internal server error";
		}
	}
}

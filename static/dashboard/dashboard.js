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
}

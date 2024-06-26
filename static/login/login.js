class Login {
	constructor() {
		this.username_elem = document.getElementById("username");
		this.password_elem = document.getElementById("password");
		this.error_elem = document.getElementById("error");
		this.submit_elem = document.getElementById("submit");

		if (Auth.get_cookie("refreshToken")) {
			window.location.href = "/dashboard";
		}

		const queryParams = new URLSearchParams(window.location.search);
		if (queryParams.has("expired")) {
			this.error_elem.innerText = "Session expired or invalid. Please log in again.";
		}

		document.getElementById("forgot-password-button").addEventListener("click", function (event) {
			event.preventDefault();
			Login.forgot_password();
		});
	}

	input(id) {
		if (id === "username") {
			this.username_elem.value = Auth.format_username(this.username_elem.value);
		}

		let password = this.password_elem.value;
		let username = this.username_elem.value;

		let button_enabled = username !== "" && password !== "";
		this.submit_elem.disabled = !button_enabled;

		if (this.error_elem.innerText === "Invalid credentials") {
			this.error_elem.innerText = "";
		}
	}

	submit() {
		const generic_err_msg = "Internal server error";

		this.submit_elem.disabled = true;

		Request.post("/api/auth/token", {
			"grant_type": "password",
			"username": this.username_elem.value,
			"password": this.password_elem.value
		}).then(response => {
			this.submit_elem.disabled = false;

			try {
				let jwt = JSON.parse(response);
				Auth.store_tokens(jwt);
				Auth.set_cookie("username", this.username_elem.value);
				window.location.href = "/dashboard";
			} catch (e) {
				this.error_elem.innerText = generic_err_msg;
			}
		}).catch(error => {
			this.submit_elem.disabled = false;

			if (error.message) {
				try {
					let error_obj = JSON.parse(error.message);
					this.error_elem.innerText = error_obj.error;
				} catch (e) {
					this.error_elem.innerText = generic_err_msg;
				}
			} else {
				this.error_elem.innerText = generic_err_msg;
			}
		});
	}

	static forgot_password() {
		let info_wrapper = document.getElementById("forgot-password-info-wrapper");
		if (info_wrapper.style.display === "none") {
			info_wrapper.style.display = "flex";
		} else {
			info_wrapper.style.display = "none";
		}
	}

	handle_enter(event) {
		if (event.key === 'Enter') {
			this.submit();
		}
	}
}

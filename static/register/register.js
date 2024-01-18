class Register {
	constructor() {
		this.username_elem = document.getElementById("username");
		this.displayname_elem = document.getElementById("displayname");
		this.password_elem = document.getElementById("password");
		this.confirmpassword_elem = document.getElementById("confirmpassword");
		this.error_elem = document.getElementById("error");
		this.submit_elem = document.getElementById("submit");

		if (Auth.get_cookie("refreshToken")) {
			window.location.href = "/dashboard";
		}

		let params = new URLSearchParams(window.location.search);
		let registration_key = params.get('k');

		let no_key = document.getElementById("no-key");
		let bad_key = document.getElementById("bad-key");
		let info_page = document.getElementById("info-page");

		function key_err() {
			info_page.style.display = "flex";
			document.getElementById("key-err").style.display = "initial";
		}

		if (registration_key) {
			this.registration_key = registration_key;
			Request.post("/api/check_registration_key", {
				"registration_key": this.registration_key,
			}).then(response => {
				try {
					let response_obj = JSON.parse(response);
					if (response_obj.success) {
						document.getElementById("registration-page").style.display = "flex";
					} else {
						key_err();
					}
				} catch (e) {
					key_err();
				}
			}).catch(error => {
				if (error.message = "Invalid registration key") {
					info_page.style.display = "flex";
					bad_key.style.display = "initial";
				} else {
					key_err();
				}
			});

		} else {
			no_key.style.display = "initial";
			info_page.style.display = "flex";
		}
	}

	input(id) {
		switch (id) {
			case "username":
				this.username_elem.value = Auth.format_username(this.username_elem.value);
				break;
			case "displayname":
				this.displayname_elem.value = this.displayname_elem.value.replace(/^\s+/, "");
				break;
		}

		let username = this.username_elem.value;
		let displayname = this.displayname_elem.value;
		let password = this.password_elem.value;
		let confirmpassword = this.confirmpassword_elem.value;

		let button_enabled = username !== "" && displayname !== "" && password !== "" && confirmpassword !== "";

		const password_mismatch_msg = "Passwords do not match";

		if (confirmpassword !== "" && password !== confirmpassword) {
			if (document.activeElement !== this.confirmpassword_elem) {
				error.innerText = password_mismatch_msg;
			}

			button_enabled = false;
		} else {
			if (error.innerText === password_mismatch_msg) {
				error.innerText = "";
			}
		}

		this.submit_elem.disabled = !button_enabled;
	}

	submit() {
		const generic_err_msg = "Internal server error";

		Request.post("/api/register_user", {
			"username": this.username_elem.value,
			"display_name": this.displayname_elem.value,
			"password": this.password_elem.value,
			"registration_key": this.registration_key,
		}).then(response => {
			try {
				let response_obj = JSON.parse(response);
				Auth.store_tokens(response_obj);
				Auth.set_cookie("username", this.username_elem.value);
				window.location.href = "/dashboard";
			} catch (e) {
				this.error_elem.innerText = generic_err_msg;
			}
		}).catch(error => {
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

}

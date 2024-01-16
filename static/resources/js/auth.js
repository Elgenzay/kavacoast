class Auth {
	static store_tokens(jwt) {
		try {
			let { access_token, refresh_token, expires_in, x_refresh_token_expires_in } = jwt;
			Auth.set_cookie('accessToken', access_token, expires_in);
			Auth.set_cookie('refreshToken', refresh_token, x_refresh_token_expires_in);
		} catch (error) {
			console.error('Error storing tokens:', error);
		}
	}

	static set_cookie(name, value, expiry_seconds) {
		let cookie = `${name}=${encodeURIComponent(value)}; path=/`;

		if (expiry_seconds) {
			let date = new Date();
			date.setTime(date.getTime() + (expiry_seconds * 1000)); // Seconds to milliseconds
			cookie += `; expires=${date.toUTCString()}; Secure`;
		}

		document.cookie = cookie;
	}

	static clear_cookie(name) {
		document.cookie = `${name}=; expires=Thu, 01 Jan 1970 00:00:00 GMT; path=/; Secure`;
	}

	static get_cookie(name) {
		let value = `; ${document.cookie}`;
		let parts = value.split(`; ${name}=`);

		if (parts.length === 2) {
			return parts.pop().split(';').shift();
		}

		return undefined;
	}

	static clear_tokens() {
		Auth.clear_cookie("accessToken");
		Auth.clear_cookie("refreshToken");
	}

	static refresh_token() {
		let refresh_token = Auth.get_cookie("refreshToken");
		let username = Auth.get_cookie("username");

		if (!refresh_token) {
			Auth.logout("expired");
		}

		Request.post("/api/auth/token", {
			"grant_type": "refresh_token",
			"username": username,
			"refresh_token": refresh_token
		}).then(response => {
			try {
				let response_obj = JSON.parse(response);
				Auth.store_tokens(response_obj);
			} catch (error) {
				Auth.logout("expired");
			}
		}).catch(e => {
			Auth.logout("expired");
		});
	}

	static logout(reason) {
		Auth.clear_tokens();
		if (reason) {
			location.href = `/login?${reason}`;
		} else {
			location.href = "/login";
		}
	}

	static format_username(name) {
		return name
			.replace(/[^a-z0-9]/gi, '_')
			.toLowerCase()
	}
}

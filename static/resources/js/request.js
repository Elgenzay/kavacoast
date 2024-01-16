class Request {
	static get(url, headers = {}) {
		return Request.send(url, null, "GET", headers);
	}

	static post(url, body, headers = {}) {
		if (!headers["Content-Type"]) {
			headers["Content-Type"] = "application/json";
		}
		return Request.send(url, body, "POST", headers);
	}

	static send(url, body, method = "GET", headers = {}) {
		return new Promise(function (resolve, reject) {
			var xhr = new XMLHttpRequest();
			xhr.open(method, url);

			for (const [key, value] of Object.entries(headers)) {
				xhr.setRequestHeader(key, value);
			}

			xhr.onload = function () {
				if (xhr.status >= 200 && xhr.status < 300) {
					resolve(xhr.response);
				} else {
					reject(new Error(xhr.response));
				}
			};

			xhr.onerror = function () {
				reject(new Error("Network error"));
			};

			if (body) {
				xhr.send(JSON.stringify(body));
			} else {
				xhr.send();
			}
		});
	}
}

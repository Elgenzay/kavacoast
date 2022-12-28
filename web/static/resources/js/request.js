class Request {

	static get(url, body) {
		return Request.send(url, body, "GET");
	}

	static post(url, body) {
		return Request.send(url, body, "POST");
	}

	static send(url, body, method = "GET") {
		return new Promise(function (resolve, reject) {
			var xhr = new XMLHttpRequest();
			xhr.open(method, url);
			xhr.setRequestHeader('Content-type', 'application/json');
			xhr.onload = resolve;
			xhr.onerror = reject;
			if (body) {
				xhr.send(body);
			} else {
				xhr.send();
			}
		});
	}
}

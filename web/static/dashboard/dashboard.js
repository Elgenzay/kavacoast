class Dashboard {
	constructor() {
		if (!this.is_valid_login()) {
			Dashboard.logout();
			return;
		}
		document.getElementById("content").style.visibility = "visible";
		Request.get("/resources/json/PublicData.json").then(function (e) {
			if (e.target.status == 200) {
				Dashboard.processpubdata(JSON.parse(e.target.response));
			}
		}, function (e) { });
	}

	static processpubdata(pubdata) {
		let days = [
			"sun",
			"mon",
			"tue",
			"wed",
			"thu",
			"fri",
			"sat",
		];
		window.scheduledata = {};
		let tables = document.getElementById("tables");
		for (let week of ["week1", "week2"]) {
			window.scheduledata[week] = {};
			let table_label = document.createElement("div");
			table_label.innerText = week == 1 ? "This week" : "Next week";
			tables.appendChild(table_label);
			let table = document.createElement("table");
			let labels = document.createElement("tr");
			labels.appendChild(document.createElement("th"));
			for (let day of days) {
				window.scheduledata[week][day] = {};
				window.scheduledata[week][day].locations = [];
				let label = document.createElement("th");
				label.innerText = day;
				labels.appendChild(label);
			}
			table.appendChild(labels);
			for (let loc in pubdata.locations) {
				let row = document.createElement("tr");
				let loc_label = document.createElement("td");
				loc_label.innerText = pubdata.locations[loc].friendly_name;
				row.appendChild(loc_label);
				for (let day of days) {
					let location_data = Dashboard.get_by_name(window.scheduledata[week][day].locations, pubdata.locations[loc].name);
					location_data.shifts = [];
					let cell = document.createElement("td");
					let dropdown = document.createElement("select");
					let off_opt = document.createElement("option");
					off_opt.setAttribute("value", "");
					off_opt.innerText = "Off";
					dropdown.appendChild(off_opt);
					for (let shift in pubdata.shifts) {
						let shift_data = Dashboard.get_by_name(location_data.shifts, pubdata.shifts[shift].name);
						shift_data["bartender"] = "";
						let option = document.createElement("option");
						option.setAttribute("value", pubdata.shifts[shift].name);
						option.innerText = pubdata.shifts[shift].friendly_name;
						dropdown.appendChild(option);
					}
					dropdown.setAttribute("onchange", "Dashboard.update('" + [week, day, pubdata.locations[loc].name].join("','") + "',this.value)");
					cell.appendChild(dropdown);
					row.appendChild(cell);
				}
				table.appendChild(row);
			}
			table.style.border = "1px solid white";
			tables.appendChild(table);
		}
		console.log(window.scheduledata);
	}

	static get_by_name(obj, name) {
		for (let i in obj) {
			if (obj[i].name == name) {
				return obj[i];
			}
		}
		obj.push({ "name": name });
		return obj[obj.length - 1];
	}

	static update(week, day, location, shift) {
		let location_data = Dashboard.get_by_name(window.scheduledata[week][day].locations, location);
		for (let s in location_data.shifts) {
			if (location_data.shifts[s].bartender == window.username) {
				location_data.shifts[s].bartender = "";
			}
		}
		var shift = Dashboard.get_by_name(location_data.shifts, shift);
		shift.bartender = window.username;
	}

	is_valid_login() {
		let login = sessionStorage.getItem("login");
		if (!login) {
			return false;
		}
		return Request.post("/api/auth", login).then(function (e) {
			if (e.target.status == 200) {
				let p = JSON.parse(login);
				document.getElementById("title").innerText = p.username;
				document.title = "Dashboard: " + p.username;
				window.username = p.username;
				window.password = p.password;
				return true;
			}
		}, function (e) {
			return false
		});
	}

	static logout() {
		sessionStorage.clear();
		window.location.href = "/login";
	}

	static save() {
		let request = JSON.stringify(
			{
				"verify": {
					"username": window.username,
					"password": window.password
				},
				"schedule": window.scheduledata
			}
		);
		Request.post("/api/schedule", request).then(function (e) {
			if (e.target.status == 200) {
				console.log("success");
				return true;
			}
		}, function (e) {
			console.error("Error: " + e.target.status);
			console.error("e.target.response");
			return false;
		});
	}
}



class Dashboard {
	constructor() {
		if (!this.is_valid_login()) {
			Dashboard.logout();
			return;
		}
	}

	static processpubdata(pubdata) {
		let tables = document.getElementById("tables");
		for (let week in window.scheduledata) {
			let table_label = document.createElement("div");
			table_label.innerText = week == "week1" ? "This week" : "Next week";
			tables.appendChild(table_label);
			let table = document.createElement("table");
			let labels = document.createElement("tr");
			labels.appendChild(document.createElement("th"));
			for (let day in window.scheduledata[week]) {
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
				for (let day in window.scheduledata[week]) {
					let location_data = Dashboard.get_by_name(window.scheduledata[week][day].locations, pubdata.locations[loc].name);
					let cell = document.createElement("td");
					let dropdown = document.createElement("select");
					let off_opt = document.createElement("option");
					off_opt.setAttribute("value", "");
					off_opt.innerText = "Off";
					dropdown.appendChild(off_opt);
					let dropdown_value = "";
					for (let shift in pubdata.shifts) {
						let shift_data = Dashboard.get_by_name(location_data.shifts, pubdata.shifts[shift].name);
						let option = document.createElement("option");
						option.setAttribute("value", pubdata.shifts[shift].name);
						option.innerText = pubdata.shifts[shift].friendly_name;
						if (shift_data["bartender"] != "") {
							if (shift_data["bartender"] != window.username) {
								option.innerText = pubdata.shifts[shift].friendly_name + " (" + shift_data["bartender"] + ")";
								option.disabled = true;
							} else {
								dropdown_value = pubdata.shifts[shift].name;
							}
						}
						dropdown.appendChild(option);
					}
					dropdown.value = dropdown_value;
					dropdown.setAttribute("onchange", "Dashboard.update('" + [week, day, pubdata.locations[loc].name].join("','") + "',this.value)");
					cell.appendChild(dropdown);
					row.appendChild(cell);
				}
				table.appendChild(row);
			}
			table.style.border = "1px solid white";
			tables.appendChild(table);
		}
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
				Dashboard.get_schedule();
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

	static save_schedule() {
		document.getElementById("content").style.visibility = "hidden";
		let request = JSON.stringify(
			{
				"verify": {
					"username": window.username,
					"password": window.password
				},
				"schedule": window.scheduledata
			}
		);
		Request.send("/api/schedule_update", request, "PUT").then(function (e) {
			if (e.target.status == 200) {
				window.location.reload();
				return true;
			}
		}, function (e) {
			console.error("Error: " + e.target.status);
			console.error(e.target.response);
			return false;
		});
	}

	static get_schedule(overwrite = false) {
		let request = JSON.stringify({
			"username": window.username,
			"password": window.password
		});
		Request.post("/api/schedule_get", request).then(function (e) {
			if (e.target.status == 200) {
				document.getElementById("content").style.visibility = "visible";
				Request.get("/resources/json/PublicData.json").then(function (e) {
					if (e.target.status == 200) {
						Dashboard.processpubdata(JSON.parse(e.target.response));
					}
				}, function (e) { });
				if (window.scheduledata_initialstr) {
					if (window.scheduledata_initialstr !== e.target.response) {
						window.alert("Data has changed since page load. Someone else may have modified the schedule. Try again.");
						window.location.reload();
					} else if (overwrite) {
						Dashboard.save_schedule();
					}
				} else {
					window.scheduledata = JSON.parse(e.target.response);
					window.scheduledata_initialstr = e.target.response;
				}
				return true;
			}
		}, function (e) {
			console.error("Error: " + e.target.status);
			console.error(e.target.response);
			return false;
		});
	}
}



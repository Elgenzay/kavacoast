class Settings {
    open(data) {
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

        Dashboard.show_page("settings");
    }

    username_input(elem) {
        this.new_username = elem.value;

        elem.value = Auth.format_username(elem.value);

        if (this.new_username !== this.current_username || this.new_username === "") {
            this.update_username_button.disabled = false;
        } else {
            this.update_username_button.disabled = true;
        }
    }

    displayname_input(elem) {
        this.new_displayname = elem.value;

        if (this.new_displayname !== this.current_displayname || this.new_displayname === "") {
            this.update_displayname_button.disabled = false;
        } else {
            this.update_displayname_button.disabled = true;
        }
    }

    current_password_input(elem) {
        this.current_password = elem.value;
        this.toggle_password_update_button();
    }

    new_password_input(elem) {
        this.new_password = elem.value;
        this.toggle_password_update_button();
    }

    confirm_password_input(elem) {
        this.confirm_new_password = elem.value;
        this.toggle_password_update_button();
    }

    toggle_password_update_button() {
        if (!this.current_password || !this.new_password || this.new_password !== this.confirm_new_password) {
            this.update_password_button.disabled = true;
        } else {
            this.update_password_button.disabled = false;
        }
    }

    password_update() {
        this.update_password_button.disabled = true;

        Auth.request("/api/users/me/change_password", {
            "old_password": this.current_password,
            "new_password": this.new_password
        }, "POST").then(r => {
            try {
                let response = JSON.parse(r);
                if (response.success) {
                    window.location.reload();
                }
            } catch (e) {
                Dashboard.display_error(e, this.update_password_error);
                this.update_password_button.disabled = false;
            }
        }).catch(e => {
            Dashboard.display_error(e, this.update_password_error);
            this.update_password_button.disabled = false;
        });
    }

    username_update() {
        this.update_username_button.disabled = true;

        Auth.request("/api/users/me", {
            "username": this.new_username
        }, "PATCH").then(r => {
            try {
                let response = JSON.parse(r);
                if (response.success) {
                    window.location.reload();
                }
            } catch (e) {
                Dashboard.display_error(e, this.update_username_error);
                this.update_username_button.disabled = false;
            }
        }).catch(e => {
            Dashboard.display_error(e, this.update_username_error);
            this.update_username_button.disabled = false;
        });
    }

    displayname_update() {
        this.update_displayname_button.disabled = true;

        Auth.request("/api/users/me", {
            "display_name": this.new_displayname
        }, "PATCH").then(r => {
            try {
                let response = JSON.parse(r);
                if (response.success) {
                    window.location.reload();
                }
            } catch (e) {
                Dashboard.display_error(e, this.update_displayname_error);
                this.update_displayname_button.disabled = false;
            }
        }).catch(e => {
            Dashboard.display_error(e, this.update_displayname_error);
            this.update_displayname_button.disabled = false;
        });
    }
}
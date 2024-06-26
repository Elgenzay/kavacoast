class Settings {
    constructor(parent, data) {
        parent.show_page("settings", () => {
            this.current_username = data.username;
            this.current_displayname = data.display_name;

            this.update_username_button = document.getElementById("settings-change-username-button");
            this.update_displayname_button = document.getElementById("settings-change-displayname-button");
            this.update_password_button = document.getElementById("settings-change-password-button");
            this.create_referral_button = document.getElementById("settings-create-referral-button");

            this.update_username_error = document.getElementById("settings-change-username-error");
            this.update_displayname_error = document.getElementById("settings-change-displayname-error");
            this.update_password_error = document.getElementById("settings-change-password-error");
            this.referral_error = document.getElementById("referral-error");
            this.referrals_container = document.getElementById("referral-list-container");

            this.referral_revoke_buttons = {};

            document.getElementById("settings-change-displayname-input").value = data.display_name;
            document.getElementById("settings-change-username-input").value = data.username;

            for (let elem of document.getElementsByClassName("discord-username")) {
                if (data.discord_username) {
                    elem.innerText = data.discord_username;
                } else {
                    elem.innerText = "N/A";
                }
            }

            this.update_referral_list(data.referrals);
        });
    }

    referral_element(registration_key) {
        let referral_elem = document.createElement("div");
        referral_elem.className = "section";
        let input = document.createElement("input");
        input.type = "text";
        input.value = `https://kavacoast.com/register?k=${registration_key}`;
        input.readOnly = true;
        referral_elem.appendChild(input);

        let img = document.createElement("img");
        img.src = "/resources/i/copy.png";
        img.className = "copy-button";
        img.setAttribute("onclick", "copy(this.previousElementSibling)");

        referral_elem.appendChild(img);

        let button = document.createElement("button");
        button.innerText = "Revoke";
        button.onclick = () => this.revoke_referral(registration_key);
        referral_elem.appendChild(button);

        this.referral_revoke_buttons[registration_key] = button;

        return referral_elem;
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
            this.update_password_button.disabled = false;

            try {
                let response = JSON.parse(r);
                if (response.success) {
                    window.location.reload();
                }
            } catch (e) {
                Dashboard.display_error(e, this.update_password_error);
            }
        }).catch(e => {
            Dashboard.display_error(e, this.update_password_error);
        });
    }

    username_update() {
        this.update_username_button.disabled = true;

        Auth.request("/api/users/me", {
            "username": this.new_username
        }, "PATCH").then(r => {
            this.update_username_button.disabled = false;

            try {
                let response = JSON.parse(r);
                if (response.success) {
                    window.location.reload();
                }
            } catch (e) {
                Dashboard.display_error(e, this.update_username_error);
            }
        }).catch(e => {
            Dashboard.display_error(e, this.update_username_error);
        });
    }

    displayname_update() {
        this.update_displayname_button.disabled = true;

        Auth.request("/api/users/me", {
            "display_name": this.new_displayname
        }, "PATCH").then(r => {
            this.update_displayname_button.disabled = false;

            try {
                let response = JSON.parse(r);
                if (response.success) {
                    window.location.reload();
                }
            } catch (e) {
                Dashboard.display_error(e, this.update_displayname_error);
            }
        }).catch(e => {
            Dashboard.display_error(e, this.update_displayname_error);
        });
    }

    create_referral() {
        this.create_referral_button.disabled = true;

        Auth.request("/api/users/me/referrals", {}, "POST").then(r => {
            this.create_referral_button.disabled = false;

            try {
                let response = JSON.parse(r);

                if (response.key) {
                    this.refresh_referrals();
                } else {
                    throw new Error("Unexpected response from server.");
                }

            } catch (e) {
                Dashboard.display_error(e, this.update_displayname_error);
            }
        }).catch(e => {
            Dashboard.display_error(e, this.referral_error);
        });
    }

    revoke_referral(registration_key) {
        this.referral_revoke_buttons[registration_key].disabled = true;

        Auth.request("/api/users/me/referrals", { "key": registration_key }, "DELETE").then(r => {
            this.referral_revoke_buttons[registration_key].disabled = false;

            try {
                let response = JSON.parse(r);

                if (response.success) {
                    this.refresh_referrals();
                } else {
                    throw new Error("Unexpected response from server.");
                }

            } catch (e) {
                Dashboard.display_error(e, this.update_displayname_error);
            }
        }).catch(e => {
            Dashboard.display_error(e, this.referral_error);
        });
    }

    refresh_referrals() {
        Auth.request("/api/users/me/referrals").then(r => {
            let response = JSON.parse(r);
            this.update_referral_list(response);
        }).catch(e => {
            console.error(e);
        });
    }

    update_referral_list(ref_urls) {
        this.referrals_container.innerHTML = "";

        for (let ref_url of ref_urls) {
            let referral_elem = this.referral_element(ref_url);
            this.referrals_container.appendChild(referral_elem);
        }
    }

}

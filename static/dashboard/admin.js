class Admin {
    constructor(parent, data) {
        parent.show_page("admin", () => {
            console.log(data);
        });
    }
}
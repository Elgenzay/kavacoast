class Pool {
    constructor(parent, data) {
        parent.show_page("pool", () => {
            console.log(data);
        });
    }
}
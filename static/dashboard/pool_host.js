class PoolHost {
    constructor(parent, data) {
        parent.show_page("pool_host", () => {
            console.log(data);
        });
    }
}
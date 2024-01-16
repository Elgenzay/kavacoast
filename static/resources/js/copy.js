function copy() {
	let link_elem = document.getElementById("direct_invite_link");
	link_elem.select();
	if (navigator.clipboard) {
		navigator.clipboard.writeText(link_elem.value)
	} else { // browser not compatible
		document.execCommand("copy");
	}
}

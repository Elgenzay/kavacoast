function copy(input, tooltip) {
	input.select();

	if (navigator.clipboard) {
		navigator.clipboard.writeText(input.value)
	} else { // browser not compatible
		document.execCommand("copy");
	}

	if (tooltip) {
		tooltip.classList.add("copied");
	}
}

function copy_blur(elem) {
	elem.classList.remove("copied");
}

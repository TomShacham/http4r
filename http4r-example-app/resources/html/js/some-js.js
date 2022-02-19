document.addEventListener("DOMContentLoaded", function () {
    collapsible()
});

function collapsible() {
    document.querySelectorAll(".collapsible").forEach(el => {
        el.addEventListener("click", function () {
            if (el.classList.contains("collapsed")) {
                el.classList.remove("collapsed");
            } else {
                el.classList.add("collapsed");
            }
        });
    })
}


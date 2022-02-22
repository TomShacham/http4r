document.addEventListener("DOMContentLoaded", function () {
    collapsible()
});

function collapsible() {
    document.querySelectorAll(".collapsible > span").forEach(el => {
        el.addEventListener("click", function (e) {
            const parent = el.parentElement;
            if (parent.classList.contains("collapsed")) {
                parent.classList.remove("collapsed");
                el.innerText = (el.innerText.slice(0, el.innerText.length - 2)) + "⬇️";
            } else {
                parent.classList.add("collapsed");
                el.innerText = (el.innerText.slice(0, el.innerText.length-2)) + "➡️"
            }
        });
    })
}


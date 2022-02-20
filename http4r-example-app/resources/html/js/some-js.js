document.addEventListener("DOMContentLoaded", function () {
    collapsible()
});

function collapsible() {
    document.querySelectorAll(".collapsible").forEach(el => {
        el.addEventListener("click", function (e) {
            console.log(e.pageY, el.offsetTop)
            if (e.pageY < el.offsetTop + (window.innerWidth/8)  && e.pageY > el.offsetTop + (window.innerWidth/50)) {
                if (el.classList.contains("collapsed")) {
                    el.classList.remove("collapsed");
                } else {
                    el.classList.add("collapsed");
                }
            }
        });
    })
}


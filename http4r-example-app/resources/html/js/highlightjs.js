function tokenise(el) {
    let innerText = el.innerText;

    const lastPass = innerText
        .split(" ")
        .map(it => keyWord(it))
        .join(" ");

    const functionArguments = new RegExp("\\(([^)]+)\\)", "g").exec(innerText)[1]
        .split(/,\s*/);

    el.innerHTML = lastPass;
}

function keyWord(text) {
    const keywords = ["as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where", "while", "abstract", "become", "box", "do", "final", "macro", "override", "priv", "typeof", "unsized", "virtual", "yield"]

    return text.split("\n").map(t => decorate(t)).join("\n");

    function decorate(text) {
        function sub(text) {
            function extracted(it, token) {
                return `${it.slice(0, it.indexOf(token) + token.length)}<div style="color: #8250df; display: inline;">${escapeHtml(it.slice(it.indexOf(token) + token.length))}</div>`;
            }

            function decorate_sub(it) {
                if (it.indexOf(".") > -1) {
                    return extracted(it, ".");
                } else if (it.indexOf("::") > -1) {
                    return extracted(it, "::");
                } else if (it.indexOf(", ") > -1) {
                    return extracted(it, ", ");
                } else if (it.indexOf(",") > -1) {
                    return extracted(it, ",");
                } else return `<div style="color: #8250df; display: inline;">${escapeHtml(it)}</div>`
            }

            return text.split("(").map(foo => {
                if (foo && foo.includes("(")) {
                    return decorate_sub(foo.split("(")[0]) + decorate_sub(foo.split("(")[1])
                } else {
                    return decorate_sub(foo);
                }
            });
        }

        if (keywords.includes(text)) {
            return `<div style="color: coral; display: inline">${escapeHtml(text)}</div>`;
        } else if (text.includes("(")) {
            return sub(text).join("(")
        } else {
            return escapeHtml(text);
        }
    }


    function escapeHtml(unsafe) {
        return unsafe
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;")
            .replace(/"/g, "&quot;")
            .replace(/'/g, "&#039;");
    }
}


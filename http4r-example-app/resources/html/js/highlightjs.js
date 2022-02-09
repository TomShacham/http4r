const tokens = {
    WHITESPACE: {value: " ", colour: "none", type: "WHITESPACE"},
    NEW_LINE: {value: "\n", colour: "none", type: "NEW_LINE"},
    COMMA: {value: ",", colour: "none", type: "COMMA"},
    OPEN_TYPE_PARAMETERS: {value: "<", colour: "none", type: "OPEN_TYPE_PARAMETERS"},
    CLOSE_TYPE_PARAMETERS: {value: ">", colour: "none", type: "CLOSE_TYPE_PARAMETERS"},
    OPEN_FUNCTION_CALL: {value: "(", colour: "none", type: "OPEN_FUNCTION_CALL"},
    CLOSE_FUNCTION_CALL: {value: ")", colour: "none", type: "CLOSE_FUNCTION_CALL"},
    OPEN_UNIT: {value: "(", colour: "none", type: "OPEN_UNIT"},
    CLOSE_UNIT: {value: ")", colour: "none", type: "CLOSE_UNIT"},
    OPEN_SCOPE: {value: "{", colour: "none", type: "OPEN_SCOPE"},
    CLOSE_SCOPE: {value: "}", colour: "none", type: "CLOSE_SCOPE"},
    METHOD_OR_ATTRIBUTE_CALL: {value: ".", colour: "unknown", type: "METHOD_OR_ATTRIBUTE_CALL"},
    ATTRIBUTE: {value: ".", colour: "purple", type: "ATTRIBUTE"},
    METHOD_CALL: {value: ".", colour: "yellow", type: "METHOD_CALL"},
    CLASS_METHOD_CALL_OR_TYPE_DECLARATION: {value: ":", colour: "none", type: "CLASS_METHOD_CALL_OR_TYPE_DECLARATION"},
    CLASS_METHOD_CALL: {value: "::", colour: "none", type: "CLASS_METHOD_CALL"},
    TYPE_DECLARATION: {value: ":", colour: "none", type: "TYPE_DECLARATION"},
    END_STATEMENT: {value: ";", colour: "none", type: "END_STATEMENT"},
    REFERENCE: {value: "&", colour: "none", type: "REFERENCE"},
    ARROW: {value: "->", colour: "none", type: "ARROW"},
    VARIABLE_OR_STRUCT_NAME: (char) => ({value: char, colour: "none", type: "VARIABLE_OR_STRUCT_NAME"}),
    KEYWORD: (kw) => ({value: kw, colour: "orange", type: "KEYWORD"}),
}

function colourCode(el) {
    let tokens = tokenise(el);
    keyWords(tokens);
}

function tokenise(el) {
    let innerText = el.innerText;
    let prev = [];

    for (const char of innerText.split("")) {
        let lastToken = prev[prev.length - 1];
        let lastLastToken = prev[prev.length - 2];
        if (char === ")" && lastToken.value === "(" && (lastLastToken.value === " " || lastLastToken.value === "\n")) {
            prev[prev.length-1] = tokens.OPEN_UNIT;
            prev.push(tokens.CLOSE_UNIT);
        } else if (char === ">" && lastToken.value === "-") {
            prev[prev.length-1] = tokens.ARROW;
        } else if (char === ":" && lastToken.type === tokens.CLASS_METHOD_CALL_OR_TYPE_DECLARATION.type) {
            prev[prev.length - 1] = tokens.CLASS_METHOD_CALL
            prev.push(tokens.CLASS_METHOD_CALL)
        } else if (char === ":") {
            prev.push(tokens.TYPE_DECLARATION)
        } else if ((char === " " || char === "," || char === ")") && lastToken === tokens.METHOD_OR_ATTRIBUTE_CALL) {
            changeTo("attribute", prev);
            insertTokenFrom(char)
        } else if (char === "(" && lastToken === tokens.METHOD_OR_ATTRIBUTE_CALL) {
            changeTo("method", prev);
            insertTokenFrom(char)
        } else {
            insertTokenFrom(char);
        }
    }

    return prev;

    function insertTokenFrom(char) {
        let token = Object.keys(tokens).find(it => tokens[it].value === char);
        if (token === undefined) {
            prev.push(tokens.VARIABLE_OR_STRUCT_NAME(char))
        } else {
            prev.push(tokens[token]);
        }
    }

    function changeTo(type, prev) {
        for (let i = prev.length - 1; i >= 0; i--) {
            if (prev[i] === tokens.METHOD_OR_ATTRIBUTE_CALL && type === "method") prev[i] = tokens.METHOD_CALL
            if (prev[i] === tokens.METHOD_OR_ATTRIBUTE_CALL && type === "attribute") prev[i] = tokens.ATTRIBUTE
        }
    }

}

function keyWords(tokens) {
    const keywords = ["as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where", "while", "abstract", "become", "box", "do", "final", "macro", "override", "priv", "typeof", "unsized", "virtual", "yield"]

    let prev = [];
    // go into state::VAR when first encountered
    // start pushing into vec
    // then replace prev.length tokens with one key word token
    // if it matches a keyword.
    for (const token of tokens) {
        let lastToken = prev[prev.length - 1];
        let lastTokenIsAName = lastToken !== undefined && lastToken.type === "VARIABLE_OR_STRUCT_NAME";
        if (lastTokenIsAName && token.type === "VARIABLE_OR_STRUCT_NAME") {
            prev.push(token);
        } else if (lastTokenIsAName) {
            for (let i = prev.length-1; i >= 0 ; i--) {
                prev[i]
            }
        }
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


function writeHtml(it, token) {
    return `${it.slice(0, it.indexOf(token) + token.length)}<div style="color: #8250df; display: inline;">${escapeHtml(it.slice(it.indexOf(token) + token.length))}</div>`;
}
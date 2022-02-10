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
    ATTRIBUTE: (word) => ({value: word, colour: "purple", type: "ATTRIBUTE"}),
    METHOD_CALL: (word) => ({value: word, colour: "yellow", type: "METHOD_CALL"}),
    CLASS_METHOD_CALL_OR_TYPE_DECLARATION: {value: ":", colour: "none", type: "CLASS_METHOD_CALL_OR_TYPE_DECLARATION"},
    OPEN_TYPE_EXPLANATION: {value: "::<", colour: "none", type: "OPEN_TYPE_EXPLANATION"},
    CLOSE_TYPE_EXPLANATION: {value: ">", colour: "none", type: "CLOSE_TYPE_EXPLANATION"},
    CLASS_METHOD_CALL: {value: "::", colour: "none", type: "CLASS_METHOD_CALL"},
    OPEN_TYPE_DECLARATION: {value: ":", colour: "none", type: "OPEN_TYPE_DECLARATION"},
    CLOSE_TYPE_DECLARATION: (char) => ({value: char, colour: "none", type: "CLOSE_TYPE_DECLARATION"}),
    TYPE: (word) => ({value: word, colour: "none", type: "TYPE"}),
    END_STATEMENT: {value: ";", colour: "orange", type: "END_STATEMENT"},
    REFERENCE: {value: "&", colour: "none", type: "REFERENCE"},
    ARROW: {value: "->", colour: "none", type: "ARROW"},
    OPEN_STRING: {value: "\"", colour: "green", type: "OPEN_STRING"},
    CLOSE_STRING: {value: "\"", colour: "green", type: "CLOSE_STRING"},
    OPEN_CHAR: {value: "'", colour: "green", type: "OPEN_CHAR"},
    CLOSE_CHAR: {value: "'", colour: "green", type: "CLOSE_CHAR"},
    ESCAPE: {value: "\\", colour: "orange", type: "ESCAPE"},
    CHAR: (char) => ({value: char, colour: "green", type: "CHAR"}),
    TYPE_PARAMETER: (char) => ({value: char, colour: "cyan", type: "TYPE_PARAMETER"}),
    STRING: (char) => ({value: char, colour: "green", type: "STRING"}),
    VAR_OR_STRUCT_DECLARATION: {value: "none", colour: "none", type: "VAR_OR_STRUCT_DECLARATION"},
    NAME: (word) => ({value: word, colour: "none", type: "NAME"}),
    KEYWORD: (kw) => ({value: kw, colour: "orange", type: "KEYWORD"}),
}

function tokenise(el) {
    let innerText = el.innerText;
    let prev = [];
    let state = [];
    let buffer = [];

    for (const char of innerText.split("")) {
        let lastToken = prev[prev.length - 1];
        let lastLastToken = prev[prev.length - 2];
        let currentState = state[state.length - 1];

        if (char === ")" && lastToken.value === "(" && (lastLastToken.value === " " || lastLastToken.value === "\n")) {
            prev[prev.length-1] = tokens.OPEN_UNIT;
            prev.push(tokens.CLOSE_UNIT);
        } else if (char === "(" && currentState === tokens.METHOD_OR_ATTRIBUTE_CALL || currentState === tokens.CLASS_METHOD_CALL_OR_TYPE_DECLARATION) {
            prev.push(tokens.OPEN_FUNCTION_CALL)
            state.pop();
        } else if (char === "\"" && lastToken.type !== tokens.STRING.type) {
            prev.push(tokens.OPEN_STRING)
        } else if (char === "\"" && lastToken.type !== tokens.ESCAPE.type) {
            prev.push(tokens.CLOSE_STRING)
        } else if (lastToken !== undefined && (lastToken.type === tokens.STRING.type || lastToken.type === tokens.OPEN_STRING.type)) {
            prev.push(tokens.STRING(char))
        } else if (char === "'" && lastToken !== undefined && lastToken.type !== tokens.STRING.type) {
            prev.push(tokens.OPEN_CHAR)
        } else if (char === "'" && lastToken !== undefined && (lastToken.type !== tokens.STRING.type || lastToken.type !== tokens.CHAR.type) ) {
            prev.push(tokens.CLOSE_CHAR)
        } else if (lastToken !== undefined && lastToken.type === tokens.OPEN_CHAR.type) {
            prev.push(tokens.CHAR(char))
        } else if (char === "." && currentState !== tokens.METHOD_OR_ATTRIBUTE_CALL.type)  {
            prev.push(tokens.METHOD_OR_ATTRIBUTE_CALL);
            state.push(tokens.METHOD_OR_ATTRIBUTE_CALL.type);
        } else if (char === ">" && lastToken !== undefined && lastToken.value === "-") {
            prev[prev.length-1] = tokens.ARROW;
        } else if (char === "<" && lastToken !== undefined && lastToken.type === tokens.CLASS_METHOD_CALL.type) {
            prev[prev.length - 1] = tokens.OPEN_TYPE_EXPLANATION;
            state.push(tokens.OPEN_TYPE_EXPLANATION.type);
        } else if (char === ">" && currentState === tokens.OPEN_TYPE_EXPLANATION.type) {
            prev.push(tokens.CLOSE_TYPE_EXPLANATION);
            state.pop();
        } else if (char === "<") {
            prev.push(tokens.OPEN_TYPE_PARAMETERS);
            state.push(tokens.OPEN_TYPE_PARAMETERS.type)
        } else if (char === ">" && currentState === tokens.OPEN_TYPE_PARAMETERS.type) {
            prev.push(tokens.CLOSE_TYPE_PARAMETERS);
            state.pop();
        } else if (currentState === tokens.OPEN_TYPE_PARAMETERS.type && (char !== " " || char !== ",")) {
            prev.push(tokens.TYPE_PARAMETER(char));
        } else if (char === ":" && lastToken !== undefined && lastToken.type === tokens.CLASS_METHOD_CALL_OR_TYPE_DECLARATION.type) {
            prev[prev.length - 1] = tokens.CLASS_METHOD_CALL;
            state.push(tokens.CLASS_METHOD_CALL.type);
        } else if (char === ":") {
            prev.push(tokens.OPEN_TYPE_DECLARATION)
            state.push(tokens.OPEN_TYPE_DECLARATION.type)
        } else if (char === " " && lastToken !== undefined && lastToken.type === tokens.OPEN_TYPE_DECLARATION.type) {
            prev.push(tokens.WHITESPACE)
        } else if ((char !== " " && char !== "," && char !== ")") && currentState === tokens.OPEN_TYPE_DECLARATION.type) {
            prev.push(tokens.TYPE(char))
        } else if (currentState === tokens.OPEN_TYPE_DECLARATION.type) {
            prev.push(tokens.CLOSE_TYPE_DECLARATION(char));
            state.pop();
        } else if (char === " " && lastToken !== undefined && lastToken.type === tokens.OPEN_TYPE_DECLARATION) {
            prev.push(tokens.WHITESPACE)
        } else if (char === ";") {
            prev.push(tokens.END_STATEMENT)
        } else if (char === "&") {
            prev.push(tokens.REFERENCE)
        } else if (char === "." && currentState === tokens.METHOD_OR_ATTRIBUTE_CALL.type) {
            let word = buffer.join("")
            prev.push(tokens.ATTRIBUTE(word))
            buffer = [];
            prev.push(tokens.METHOD_OR_ATTRIBUTE_CALL);
        } else if ((char === " " || char === "," || char === ")") && currentState === tokens.METHOD_OR_ATTRIBUTE_CALL.type) {
            state.pop();
            let word = buffer.join("")
            prev.push(tokens.ATTRIBUTE(word))
            buffer = [];
            insertTokenFrom(char)
        } else if (char === "(" && currentState === tokens.METHOD_OR_ATTRIBUTE_CALL.type) {
            state.pop();
            let word = buffer.join("")
            prev.push(tokens.METHOD_CALL(word))
            buffer = [];
            prev.push(tokens.OPEN_FUNCTION_CALL)
        } else if (currentState === tokens.METHOD_OR_ATTRIBUTE_CALL.type) {
            buffer.push(char)
        } else if (char === " " && currentState === tokens.VAR_OR_STRUCT_DECLARATION.type) {
            state.pop();
            let word = buffer.join("")
            if (isKeyWord(word)) {
                prev.push(tokens.KEYWORD(word))
            } else {
                prev.push(tokens.NAME(word))
            }
            buffer = [];
            prev.push(tokens.WHITESPACE);
        } else if (currentState === tokens.VAR_OR_STRUCT_DECLARATION.type) {
            buffer.push(char)
        } else {
            let token = Object.keys(tokens).find(it => tokens[it].value === char);
            if (token === undefined) {
                state.push(tokens.VAR_OR_STRUCT_DECLARATION.type);
                buffer.push(char);
            } else {
                prev.push(tokens[token]);
            }
        }
    }

    // todo() handle =, ==, !=, &&, ||

    return prev;

    function insertTokenFrom(char) {
        let token = Object.keys(tokens).find(it => tokens[it].value === char);
        if (token === undefined) {
            prev.push(tokens.NAME(char))
        } else {
            prev.push(tokens[token]);
        }
    }

}



function isKeyWord(word) {
    const keywords = ["as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where", "while", "abstract", "become", "box", "do", "final", "macro", "override", "priv", "self", "typeof", "unsized", "virtual", "yield"]
    return keywords.includes(word);
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
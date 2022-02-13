document.addEventListener("DOMContentLoaded", function () {
    colourCode();
});

function colourCode() {
    document.querySelectorAll("code").forEach(el => {
        let tokens1 = tokenise(el);
        console.log(tokens1);
        el.innerHTML = fold(tokens1);
    })
}

function fold(tokens) {
    let types = [];
    let buffer = [];
    let out = [];
    let prev = undefined;

    for (let token of tokens) {
        if (token.type === "TYPE_PARAMETER") {
            types.push(token.value);
        }
        let isActuallyAType = (token.type === "NAME" || token.type === "TYPE") && types.includes(token.value);

        if (isActuallyAType) {
            writeBuffer(buffer, "GROUPED");
            out.push(tokenToHtml({value: token.value, colour: "darkcyan", type: "TYPE_PARAMETER"}));
        } else if (token.type.includes("STRING") && (prev !== undefined && !prev.type.includes("STRING"))) {
            writeBuffer(buffer, "GROUPED")
            buffer = [token];
        } else if (token.type.includes("STRING") && (prev !== undefined && prev.type.includes("STRING"))) {
            buffer.push(token);
        } else if (!token.type.includes("STRING") && (prev !== undefined && prev.type.includes("STRING"))) {
            writeBuffer(buffer, "STRING");
            buffer.push(token);
        } else if (token.colour !== "none") {
            writeBuffer(buffer, "GROUPED")
            out.push(tokenToHtml(token));
        } else if (token.colour === "none") {
            buffer.push(token);
        }
        prev = token;
    }

    writeBuffer(buffer, "GROUPED")

    function writeBuffer(buf, type) {
        let colour = type === "STRING" ? "#171" : "none";
        if (buf.length > 0) {
            out.push(tokenToHtml({value: buf.map(t => t.value).join(""), colour: colour, type: type}));
        }
        buffer = [];
    }

    return out.join("");
}

function tokenToHtml(token) {
    return `<div class="${token.type}">${escapeHtml(token.value)}</div>`;
}

const tokens = {
    WHITESPACE: {value: " ", colour: "none", type: "WHITESPACE"},
    COMMENT: {value: "//", colour: "#777", type: "COMMENT"},
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
    ATTRIBUTE: (word) => ({value: word, colour: "#8250df", type: "ATTRIBUTE"}),
    METHOD_CALL: (word) => ({value: word, colour: "#ca5", type: "METHOD_CALL"}),
    CLASS_METHOD_CALL_OR_TYPE_DECLARATION: {value: ":", colour: "none", type: "CLASS_METHOD_CALL_OR_TYPE_DECLARATION"},
    OPEN_TYPE_EXPLANATION: {value: "::<", colour: "none", type: "OPEN_TYPE_EXPLANATION"},
    CLOSE_TYPE_EXPLANATION: {value: ">", colour: "none", type: "CLOSE_TYPE_EXPLANATION"},
    CLASS_METHOD_CALL: {value: "::", colour: "none", type: "CLASS_METHOD_CALL"},
    OPEN_TYPE_DECLARATION: {value: ":", colour: "none", type: "OPEN_TYPE_DECLARATION"},
    CLOSE_TYPE_DECLARATION: (char) => ({value: char, colour: "none", type: "CLOSE_TYPE_DECLARATION"}),
    TYPE: (word) => ({value: word, colour: "none", type: "TYPE"}),
    END_STATEMENT: {value: ";", colour: "#c75", type: "END_STATEMENT"},
    REFERENCE: {value: "&", colour: "none", type: "REFERENCE"},
    ARROW: {value: "->", colour: "none", type: "ARROW"},
    OPEN_STRING: {value: "\"", colour: "#171", type: "OPEN_STRING"},
    CLOSE_STRING: {value: "\"", colour: "#171", type: "CLOSE_STRING"},
    OPEN_CHAR: {value: "'", colour: "#171", type: "OPEN_CHAR"},
    CLOSE_CHAR: {value: "'", colour: "#171", type: "CLOSE_CHAR"},
    ESCAPE: {value: "\\", colour: "#c75", type: "ESCAPE"},
    CHAR: (char) => ({value: char, colour: "#171", type: "CHAR"}),
    TYPE_PARAMETER: (char) => ({value: char, colour: "darkcyan", type: "TYPE_PARAMETER"}),
    STRING: (char) => ({value: char, colour: "#171", type: "STRING"}),
    COMMENT_STRING: (str) => ({value: str, colour: "#777", type: "COMMENT"}),
    VAR_OR_STRUCT_DECLARATION: {value: "none", colour: "none", type: "VAR_OR_STRUCT_DECLARATION"},
    NAME: (word) => ({value: word, colour: "none", type: "NAME"}),
    KEYWORD: (kw) => ({value: kw, colour: "#d75", type: "KEYWORD"}),
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

        if (char === "/" && lastToken && (lastToken.value === " " || lastToken.value === "\n")) {
            prev.push(tokens.COMMENT);
            state.push(tokens.COMMENT.type);
        } else if (char === "/" && lastToken && lastToken.type === "COMMENT") {
            continue
        } else if (char === "\n" && currentState === tokens.COMMENT.type) {
            writeBuffer(buffer, tokens.COMMENT_STRING)
            state.pop();
            prev.push(tokens.NEW_LINE);
        } else if (currentState === tokens.COMMENT.type) {
            buffer.push(char);
        } else if (char === ")" && lastToken !== undefined && lastToken.value === "(" && (lastLastToken !== undefined && (lastLastToken.value === " " || lastLastToken.value === "\n"))) {
            prev[prev.length - 1] = tokens.OPEN_UNIT;
            prev.push(tokens.CLOSE_UNIT);
        } else if (char === "(") {
            let firstCharOfLastToken = lastToken.value.charAt(0);
            if (lastToken && (firstCharOfLastToken === firstCharOfLastToken.toUpperCase()) && (firstCharOfLastToken !== ":" && firstCharOfLastToken !== ".")) {
                writeBuffer(buffer, tokens.ATTRIBUTE)
            }else {
                writeBuffer(buffer, tokens.METHOD_CALL)
            }
            prev.push(tokens.OPEN_FUNCTION_CALL)
            state.pop();
        } else if (char === "\"" && lastToken !== undefined && lastToken.type !== "STRING") {
            writeBuffer(buffer, tokens.NAME);
            prev.push(tokens.OPEN_STRING)
        } else if (char === "\"" && lastToken !== undefined && lastToken.value !== tokens.ESCAPE.value) {
            prev.push(tokens.CLOSE_STRING)
        } else if (lastToken !== undefined && (lastToken.type === "STRING" || lastToken.type === tokens.OPEN_STRING.type)) {
            prev.push(tokens.STRING(char))
        } else if (char === "'" && lastToken !== undefined && lastToken.type !== tokens.STRING.type) {
            prev.push(tokens.OPEN_CHAR)
        } else if (char === "'" && lastToken !== undefined && (lastToken.type !== tokens.STRING.type || lastToken.type !== tokens.CHAR.type)) {
            prev.push(tokens.CLOSE_CHAR)
        } else if (lastToken !== undefined && lastToken.type === tokens.OPEN_CHAR.type) {
            prev.push(tokens.CHAR(char))
        } else if (char === "." && currentState !== tokens.METHOD_OR_ATTRIBUTE_CALL.type) {
            writeBuffer(buffer, tokens.NAME)
            prev.push(tokens.METHOD_OR_ATTRIBUTE_CALL);
            state.push(tokens.METHOD_OR_ATTRIBUTE_CALL.type);
        } else if (char === ">" && lastToken !== undefined && lastToken.value === "-") {
            prev[prev.length - 1] = tokens.ARROW;
        } else if (char === "<" && lastToken !== undefined && lastToken.type === tokens.CLASS_METHOD_CALL.type) {
            prev[prev.length - 1] = tokens.OPEN_TYPE_EXPLANATION;
            state.push(tokens.OPEN_TYPE_EXPLANATION.type);
        } else if (char === ">" && currentState === tokens.OPEN_TYPE_EXPLANATION.type) {
            prev.push(tokens.CLOSE_TYPE_EXPLANATION);
            state.pop();
        } else if (char === "<") {
            if (isKeyWord(buffer.join(""))) {
                writeBuffer(buffer, tokens.KEYWORD)
            } else if (lastLastToken && lastLastToken.value !== "for") {
                writeBuffer(buffer, tokens.METHOD_CALL)
            } else {
                writeBuffer(buffer, tokens.NAME)
            }
            prev.push(tokens.OPEN_TYPE_PARAMETERS);
            state.push(tokens.OPEN_TYPE_PARAMETERS.type)
        } else if (char === ">" && currentState === tokens.OPEN_TYPE_PARAMETERS.type) {
            prev.push(tokens.CLOSE_TYPE_PARAMETERS);
            state.pop();
        } else if (currentState === tokens.OPEN_TYPE_PARAMETERS.type && (char !== " " || char !== ",")) {
            prev.push(tokens.TYPE_PARAMETER(char));
        } else if (char === ":" && lastToken !== undefined && lastToken.type === tokens.OPEN_TYPE_DECLARATION.type) {
            prev[prev.length - 1] = tokens.CLASS_METHOD_CALL;
            state.pop()
            state.push(tokens.CLASS_METHOD_CALL.type);
        } else if (char === ":") {
            writeBuffer(buffer, tokens.NAME);
            prev.push(tokens.OPEN_TYPE_DECLARATION)
            state.push(tokens.OPEN_TYPE_DECLARATION.type)
        } else if (char === " " && lastToken !== undefined && lastToken.type === tokens.OPEN_TYPE_DECLARATION.type) {
            prev.push(tokens.WHITESPACE)
        } else if ((char !== " " && char !== "," && char !== ")") && currentState === tokens.OPEN_TYPE_DECLARATION.type) {
            buffer.push(char);
        } else if (currentState === tokens.OPEN_TYPE_DECLARATION.type) {
            writeBuffer(buffer, tokens.TYPE);
            prev.push(tokens.CLOSE_TYPE_DECLARATION(char));
            state.pop();
        } else if (char === " " && lastToken !== undefined && lastToken.type === tokens.OPEN_TYPE_DECLARATION) {
            prev.push(tokens.WHITESPACE)
        } else if (char === ";") {
            writeBuffer(buffer, tokens.NAME);
            prev.push(tokens.END_STATEMENT)
        } else if (char === "&") {
            prev.push(tokens.REFERENCE)
        } else if (char === "." && currentState === tokens.METHOD_OR_ATTRIBUTE_CALL.type) {
            writeBuffer(buffer, tokens.ATTRIBUTE)
            prev.push(tokens.METHOD_OR_ATTRIBUTE_CALL);
        } else if ((char === " " || char === "," || char === ")") && currentState === tokens.METHOD_OR_ATTRIBUTE_CALL.type) {
            state.pop();
            writeBuffer(buffer, tokens.ATTRIBUTE)
            insertTokenFrom(char)
        } else if ((char === "(") && currentState === tokens.METHOD_OR_ATTRIBUTE_CALL.type) {
            state.pop();
            writeBuffer(buffer, tokens.METHOD_CALL)
            prev.push(tokens.OPEN_FUNCTION_CALL)
        } else if (currentState === tokens.METHOD_OR_ATTRIBUTE_CALL.type) {
            buffer.push(char)
        } else if ((char === " " || char === "(" || char === "," || char === ":" || char === "\n" || char === ".") && currentState === tokens.VAR_OR_STRUCT_DECLARATION.type) {
            state.pop();
            writeBuffer(buffer, keyWordOrName)
            insertTokenFrom(char);
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

    return prev;

    function writeBuffer(buf, f) {
        let word = buf.join("");
        let items = f(word);
        prev.push(items);
        buffer = [];
    }

    function keyWordOrName(word) {
        if (isKeyWord(word)) {
            return tokens.KEYWORD(word);
        } else {
            return tokens.NAME(word);
        }
    }

    function insertTokenFrom(char) {
        let token = Object.keys(tokens).find(it => tokens[it].value === char);
        // if it's not in our list of tokens then it's a name like a var or struct
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

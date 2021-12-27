import init, {serve, jsRequest} from "../pkg/http4r_core.js";
async function run() {
    await init();
    let request = jsRequest("GET", "/", "body", "X-Tom: Cool beans; Cache-Control: private, max-age:0");
    let response = await serve(request);
    const jsResponse = {
        status: response.status(),
        body: response.body(),
        headers: response.headers().split("; ").map(string => string.split(": "))
    }
    console.log(jsResponse);
}

run();
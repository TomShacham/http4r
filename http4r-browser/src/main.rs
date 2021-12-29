use http4r_browser::router::Router;
use http4r_core::server::Server;

fn main(){
    let mut server = Server::new(8080);
    server.start(|| { Ok(Router {}) }, None);
}

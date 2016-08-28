extern crate iron;

use self::iron::prelude::*;
use self::iron::status;

pub fn start_server() {

    fn hello_world(req: &mut Request) -> IronResult<Response> {
        println!("URL: {}", req.url);
        Ok(Response::with((status::Ok, "Hello World!")))
    }

    Iron::new(hello_world).http("localhost:80").unwrap();
    println!("On 80");
}